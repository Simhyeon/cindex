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

        // Add table with column types
        indexer.add_table(
            "table2", 
            vec![CsvType::Text, CsvType::Text],
            "id,address
10,110-2222".as_bytes()
        )?;

        // Add table from stdin
        let stdin = std::io::stdin();
        //indexer.add_table_fast(
            //"table3", 
            //stdin.lock()
        //);

        // Indexing

        // Create query object and print output to terminal
        let query = Query::from_str("SELECT a,b,c FROM table1 WHERE a = 10");
        indexer.index(query, OutOption::Term)?;

        // Use raw query and yield output to a file
        indexer.index_raw("SELECT * FROM table2 WHERE id = 10", OutOption::File(std::fs::File::create("out.csv")?))?;

        // Use builder pattern to construct query and index
        let query = Query::empty("table2")
            .columns(vec!["id", "address"])
            .predicate(
                Predicate::new("id", Operator::Equal)
                .args(vec!["10"])
            )
            .predicate(
                Predicate::build()
                .column("address")
                .operator(Operator::NotEqual)
                .raw_args("111-2222")
            );

        let mut acc = String::new();
        indexer.index(query, OutOption::Value(&mut acc))?;

        Ok(())
    }
}

