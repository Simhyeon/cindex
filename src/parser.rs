use crate::models::{Query, Predicate, Operator, Separator};

pub struct Parser { 
    cursor: ParseCursor,
    state: ParseState,
}

#[derive(Default)]
pub struct ParseState {
    table_name: String,
    raw_column_names: Option<String>,
    where_args: Vec<String>,
    joined: Option<Vec<String>>,
}

#[derive(PartialEq)]
pub enum ParseCursor {
    None,
    From,
    Select,
    Where,
    Join,
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

    pub fn parse(&mut self, query: &str) -> Query {
        let mut split = query.split(' ');

        // SELECT columns FROM TABLE WHERE arguments
        while let Some(word) = split.next() {
            // If no cursor change, then
            if !self.set_cursor(word) {
                self.update_state(word);
            }
        }

        let table_name = std::mem::replace(&mut self.state.table_name,String::new());
        let columns = std::mem::replace(&mut self.state.raw_column_names,None);
        let predicates = self.get_predicates();
        let joined = std::mem::replace(&mut self.state.joined,None);

        // Split 
        let columns = if let Some(raw) = columns {
            Some(raw.split(',').map(|v| v.trim().to_owned()).collect())
        } else {
            None
        };

        Query::new(table_name,columns,predicates, joined)
    }

    fn set_cursor(&mut self, arg: &str) -> bool {
        let candidate = match arg.to_lowercase().as_str() {
            "select" => ParseCursor::Select, 
            "where" => ParseCursor::Where,
            "from" => ParseCursor::From,
            "join" => ParseCursor::Join,
            _ => ParseCursor::None,
        };

        if candidate != self.cursor && candidate != ParseCursor::None {
            self.cursor = candidate;
            true
        } else {
            false
        }
    }

    fn update_state(&mut self, arg: &str) {
        match self.cursor {
            ParseCursor::From => {self.state.table_name = arg.to_owned();}
            ParseCursor::Select => {
                if self.state.raw_column_names == None {
                    self.state.raw_column_names.replace(String::new());
                }
                // This is safe to use unwrap 
                self.state.raw_column_names.as_mut().unwrap().push_str(arg);
            }
            ParseCursor::Where => {self.state.where_args.push(arg.to_owned());}
            ParseCursor::Join => {
                if self.state.joined == None {
                    self.state.joined.replace(vec![]);
                }
                // This is safe to use unwrap 
                self.state.joined.as_mut().unwrap().push(arg.to_owned());
                self.cursor = ParseCursor::None;
            }
            _ => {}
        }
    }

    // Inner parse predicate arguments
    fn get_predicates(&self) -> Option<Vec<Predicate>> {
        let mut predicates = vec![];
        let mut p = Predicate::build();
        let mut w_cursor = WhereCursor::Left;

        for token in &self.state.where_args {
            if let Some(sep) = self.find_separator(token) {
                if !p.column.is_empty() {predicates.push(p);}

                // Reset predicate and where cursor
                // for next iteration
                p = Predicate::build();
                w_cursor = WhereCursor::Left;

                p.set_separator(sep); 
            }

            match w_cursor {
                WhereCursor::Left => {
                    p.set_column(token);
                    w_cursor = WhereCursor::Operator;
                },
                WhereCursor::Operator => {
                    p.set_operator(Operator::from_token(token));
                    w_cursor = WhereCursor::Right;
                },
                WhereCursor::Right => {
                    p.add_arg(token);
                },
            } 
        }

        // Add a lastly created predicated into vector
        if !p.column.is_empty() {predicates.push(p);}

        Some(predicates)
    }

    fn find_separator(&self, token: &str) -> Option<Separator> {
        match token {
            "&&" => Some(Separator::And),
            "||" => Some(Separator::Or),
            _ => None
        }
    }
}