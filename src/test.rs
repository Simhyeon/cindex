#[cfg(test)]
mod tests {
    use crate::models::Operator;
    use crate::CIndexResult;
    use crate::{
        indexer::{Indexer, OutOption},
        models::{CsvType, Predicate, Query},
    };
    use std::fs::File;

    #[test]
    fn test_function() -> CIndexResult<()> {
        let mut indexer = Indexer::new();

        // Add table without types
        // Default types for every columns are "Text"
        indexer.add_table_fast("table1", File::open("test.csv").expect("Failed to open"))?;

        // Indexing

        // Create query object and print output to terminal
        let query = Query::from_str(
            r#"SELECT c,b,a,d,e FROM table1 ORDER BY a DESC WHERE c IN 111 333 224 FLAG SUP PHD"#,
        )?;
        indexer.index(query, OutOption::Term)?;
        indexer.index(
            Query::from_str("SELECT * FROM table1 ORDER BY a ASEC FLAG PHD")?,
            OutOption::Term,
        )?;

        Ok(())
    }
}
