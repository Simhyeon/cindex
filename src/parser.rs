use crate::query::{Operator, Predicate, Query, QueryFlags, Separator};
use crate::{models::OrderType, CIndexError, CIndexResult};

pub struct Parser {
    cursor: ParseCursor,
    state: ParseState,
}

#[derive(Default)]
pub struct ParseState {
    table_name: String,
    raw_column_names: String,
    raw_column_map: Option<String>,
    where_args: Vec<String>,
    joined: Option<Vec<String>>,
    order_by: Vec<String>,
    flags: QueryFlags,
    range: (usize, usize),
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
        let split = tokens_with_quote(query).into_iter();

        // SELECT columns FROM TABLE WHERE arguments
        for word in split {
            let word = word.trim();
            if word.is_empty() {
                continue;
            }
            // If no cursor change, then
            if !self.set_cursor(word) {
                self.update_state(word)?;
            }
        }

        let table_name = std::mem::take(&mut self.state.table_name);
        let columns = std::mem::take(&mut self.state.raw_column_names);
        let predicates = self.get_predicates()?;
        let joined = std::mem::replace(&mut self.state.joined, None);
        let order_type = match self.state.order_by.len() {
            1 => OrderType::from_str("ASEC", &self.state.order_by[0])?, // Default ordering is ASEC
            2 => OrderType::from_str(&self.state.order_by[1], &self.state.order_by[0])?,
            len if len > 2 => {
                OrderType::from_str(&self.state.order_by[1], &self.state.order_by[0])?
            }
            _ => OrderType::None,
        };

        let column_map = std::mem::replace(&mut self.state.raw_column_map, None)
            .map(|s| s.split(',').map(|s| s.to_owned()).collect::<Vec<String>>());

        // Split
        let columns = columns.split(',').map(|s| s.to_owned()).collect::<Vec<_>>();

        Ok(Query::new(
            table_name,
            columns,
            predicates,
            joined,
            order_type,
            column_map,
            std::mem::take(&mut self.state.flags),
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
            "limit" => ParseCursor::Limit,
            "offset" => ParseCursor::Offset,
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
        match self.cursor {
            ParseCursor::From => {
                self.state.table_name = arg.to_owned();
            }
            ParseCursor::Select => {
                // This is safe to use unwrap
                self.state.raw_column_names = arg.to_owned();
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
                // This is safe to use unwrap
                self.state.raw_column_map.as_mut().unwrap().push_str(arg);
            }
            ParseCursor::Flag => {
                self.state.flags.set(arg)?;
            }
            ParseCursor::Offset => {
                self.state.range.0 = arg.parse().map_err(|_| {
                    CIndexError::InvalidTableName(
                        "Limit's argument should be usize value".to_owned(),
                    )
                })?;
            }
            ParseCursor::Limit => {
                self.state.range.1 = arg.parse().map_err(|_| {
                    CIndexError::InvalidTableName(
                        "Limit's argument should be usize value".to_owned(),
                    )
                })?;
            }
            ParseCursor::None | ParseCursor::OrderBy(false) => (),
        }
        Ok(())
    }

    // Inner parse predicate arguments
    fn get_predicates(&mut self) -> CIndexResult<Option<Vec<Predicate>>> {
        let mut predicates = vec![];
        let mut p = Predicate::build();
        let mut w_cursor = WhereCursor::Left;

        for token in &self.state.where_args {
            if let Some(sep) = self.find_separator(token) {
                Self::predicate_chore(&mut p)?;

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
                    p.set_operator(Operator::from_token(token)?);
                    w_cursor = WhereCursor::Right;
                }
                WhereCursor::Right => {
                    p.add_arg(token);
                }
            }
        }

        // Add a lastly created predicated into vector
        if !p.column.is_empty() {
            Self::predicate_chore(&mut p)?;
            predicates.push(p);
        }

        Ok(Some(predicates))
    }

    /// Apply chores for predicate
    fn predicate_chore(predicate: &mut Predicate) -> CIndexResult<()> {
        // Precompile regex
        if Operator::Like == predicate.operation {
            predicate.set_matcher(
                &predicate
                    .arguments
                    .last()
                    .ok_or_else(|| {
                        CIndexError::InvalidQueryStatement(
                            "Like requires argument to be a valid query".to_owned(),
                        )
                    })?
                    .clone(),
            )?;
        }

        if predicate.arguments.is_empty() {
            return Err(CIndexError::InvalidQueryStatement(format!(
                "Query has insufficient arguments : {:?}",
                predicate
            )));
        }

        Ok(())
    }

    fn find_separator(&self, token: &str) -> Option<Separator> {
        match token {
            "AND" => Some(Separator::And),
            "OR" => Some(Separator::Or),
            _ => None,
        }
    }
}

/// Parse string with quote-able tokens
///
/// This strips surround quote.
fn tokens_with_quote(source: &str) -> Vec<String> {
    let mut tokens = vec![];
    let mut on_quote = false;
    let mut previous = ' ';
    let mut chunk = String::new();
    let iter = source.chars().peekable();
    for ch in iter {
        match ch {
            // Escape character should not bed added
            '\\' => {
                if previous == '\\' {
                    previous = ' '; // Reset previous
                } else {
                    on_quote = !on_quote;
                    previous = ch;
                    continue;
                }
            }
            '\'' => {
                // Add literal quote if previous was escape character
                if previous == '\\' {
                    previous = ' '; // Reset previous
                } else {
                    on_quote = !on_quote;
                    previous = ch;
                    continue;
                }
            }
            ' ' => {
                if !on_quote {
                    // If previous is also blank. skip
                    if previous == ' ' {
                        continue;
                    }
                    let flushed = std::mem::take(&mut chunk);
                    tokens.push(flushed);
                    previous = ch;
                    continue;
                }
            }
            _ => previous = ch,
        }
        chunk.push(ch);
    }
    if !chunk.is_empty() {
        tokens.push(chunk);
    }
    tokens
}
