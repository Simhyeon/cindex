use crate::{
    models::{Operator, OrderType, Predicate, Query, QueryFlags, Separator},
    CIndexResult, CIndexError,
};

pub struct Parser {
    cursor: ParseCursor,
    state: ParseState,
}

#[derive(Default)]
pub struct ParseState {
    table_name: String,
    raw_column_names: Option<String>,
    raw_column_map: Option<String>,
    where_args: Vec<String>,
    joined: Option<Vec<String>>,
    order_by: Vec<String>,
    flags: QueryFlags,
    range: (usize,usize),
}

#[derive(PartialEq)]
pub enum ParseCursor {
    None,
    From,
    Select,
    Where,
    Join,
    OrderBy(bool),
    Limit,
    Offset,
    Hmap,
    Flag,
}

pub enum WhereCursor {
    Left,
    Operator,
    Right,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            cursor: ParseCursor::None,
            state: ParseState::default(),
        }
    }

    pub fn parse(&mut self, query: &str) -> CIndexResult<Query> {
        let mut split = query.split_whitespace();

        // SELECT columns FROM TABLE WHERE arguments
        while let Some(word) = split.next() {
            // If no cursor change, then
            if !self.set_cursor(word) {
                self.update_state(word)?;
            }
        }

        let table_name = std::mem::replace(&mut self.state.table_name, String::new());
        let columns = std::mem::replace(&mut self.state.raw_column_names, None);
        let predicates = self.get_predicates();
        let joined = std::mem::replace(&mut self.state.joined, None);
        let order_type = match self.state.order_by.len() {
            2 => OrderType::from_str(&self.state.order_by[1], &self.state.order_by[0])?,
            1 => OrderType::from_str(&self.state.order_by[1], "ASEC")?, // Default ordering is ASEC
            len if len > 2 => {
                OrderType::from_str(&self.state.order_by[1], &self.state.order_by[0])?
            }
            _ => OrderType::None,
        };

        let column_map = std::mem::replace(&mut self.state.raw_column_map, None)
            .map(|s| s.split(',').map(|s| s.to_owned()).collect::<Vec<String>>());

        // Split
        let columns = if let Some(raw) = columns {
            Some(raw.split(',').map(|v| v.trim().to_owned()).collect())
        } else {
            None
        };

        Ok(Query::new(
            table_name,
            columns,
            predicates,
            joined,
            order_type,
            column_map,
            self.state.flags,
            self.state.range,
        ))
    }

    fn set_cursor(&mut self, arg: &str) -> bool {
        let candidate = match arg.to_lowercase().as_str() {
            "select" => ParseCursor::Select,
            "where" => ParseCursor::Where,
            "from" => ParseCursor::From,
            "join" => ParseCursor::Join,
            "hmap" => ParseCursor::Hmap,
            "order" => ParseCursor::OrderBy(false),
            "by" => {
                if self.cursor == ParseCursor::OrderBy(false) {
                    ParseCursor::OrderBy(true)
                } else {
                    ParseCursor::None
                }
            }
            "limit" => {
                ParseCursor::Limit
            }
            "offset" => {
                ParseCursor::Offset
            }
            "flag" => ParseCursor::Flag,
            _ => ParseCursor::None,
        };

        if candidate != self.cursor && candidate != ParseCursor::None {
            self.cursor = candidate;
            true
        } else {
            false
        }
    }

    fn update_state(&mut self, arg: &str) -> CIndexResult<()> {
        let arg = self.without_dquotes(arg);
        match self.cursor {
            ParseCursor::From => {
                self.state.table_name = arg.to_owned();
            }
            ParseCursor::Select => {
                if self.state.raw_column_names == None {
                    self.state.raw_column_names.replace(String::new());
                }
                // This is safe to use unwrap
                self.state
                    .raw_column_names
                    .as_mut()
                    .unwrap()
                    .push_str(&format!("{} ", arg));
            }
            ParseCursor::Where => {
                self.state.where_args.push(arg.to_owned());
            }
            ParseCursor::Join => {
                if self.state.joined == None {
                    self.state.joined.replace(vec![]);
                }
                // This is safe to use unwrap
                self.state.joined.as_mut().unwrap().push(arg.to_owned());
                self.cursor = ParseCursor::None;
            }
            ParseCursor::OrderBy(true) => {
                self.state.order_by.push(arg.to_owned());
            }
            ParseCursor::Hmap => {
                if self.state.raw_column_map == None {
                    self.state.raw_column_map.replace(String::new());
                }
                let arg = arg.replace("_", " ");
                self.state.raw_column_map.as_mut().unwrap().push_str(&arg);
            }
            ParseCursor::Flag => {
                self.state.flags.set_str(&arg)?;
            }
            ParseCursor::Offset => {
                self.state.range.0 = arg.parse()
                    .map_err(|_| CIndexError::InvalidTableName(format!("Limit's argument should be usize value")))?;
            }
            ParseCursor::Limit => {
                self.state.range.1 = arg.parse()
                    .map_err(|_| CIndexError::InvalidTableName(format!("Limit's argument should be usize value")))?;
            }
            ParseCursor::None | ParseCursor::OrderBy(false) => (),
        }
        Ok(())
    }

    // Inner parse predicate arguments
    fn get_predicates(&self) -> Option<Vec<Predicate>> {
        let mut predicates = vec![];
        let mut p = Predicate::build();
        let mut w_cursor = WhereCursor::Left;

        for token in &self.state.where_args {
            if let Some(sep) = self.find_separator(token) {
                if !p.column.is_empty() {
                    predicates.push(p);
                }

                // Reset predicate and where cursor
                // for next iteration
                p = Predicate::build();
                w_cursor = WhereCursor::Left;

                p.set_separator(sep);
                continue;
            }

            match w_cursor {
                WhereCursor::Left => {
                    p.set_column(token);
                    w_cursor = WhereCursor::Operator;
                }
                WhereCursor::Operator => {
                    p.set_operator(Operator::from_token(token));
                    w_cursor = WhereCursor::Right;
                }
                WhereCursor::Right => {
                    p.add_arg(token);
                }
            }
        }

        // Add a lastly created predicated into vector
        if !p.column.is_empty() {
            predicates.push(p);
        }

        Some(predicates)
    }

    fn find_separator(&self, token: &str) -> Option<Separator> {
        match token {
            "AND" => Some(Separator::And),
            "OR" => Some(Separator::Or),
            _ => None,
        }
    }

    fn without_dquotes(&self, src: &str) -> String {
        src.replace('"', "")
    }
}
