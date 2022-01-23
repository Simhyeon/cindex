#[cfg(test)]
mod tests {
    use std::fs::File;
    use crate::{indexer::{Indexer, OutOption}, models::{Query, CsvType, Predicate}};
    use crate::CIndexResult;
    use crate::models::Operator;

    #[test]
    fn it_works() -> CIndexResult<()> {

        let mut indexer = Indexer::new();

        // Add table without types
        // Default types for every columns are "Text"
        indexer.add_table_fast(
            "table1", 
            File::open("test.csv").expect("Failed to open")
        )?;

        // Indexing

        // Create query object and print output to terminal
        let query = Query::from_str("SELECT * FROM table1 ORDER BY a DESC WHERE c IN 111 333 224 HMAP 첫_첫,둘_둘,셋_셋")?;
        println!("{:#?}", query);
        indexer.index(query, OutOption::Term)?;

        Ok(())
    }
}

