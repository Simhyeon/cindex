#[cfg(test)]
mod tests {
    use crate::indexer::{Indexer, OutOption};
    use crate::CIndexResult;
    use std::fs::File;

    #[test]
    fn test_function() -> CIndexResult<()> {
        let mut indexer = Indexer::new();

        // Add table without types
        // Default types for every columns are "Text"
        indexer.add_table_fast(
            "table1",
            "a,b,c
1,2,3"
                .as_bytes(),
        )?;

        // Indexing
        //indexer.index_raw("SELECT * FROM table1 FLAG PHD", OutOption::Term)?;
        indexer.index_raw(
            "SELECT a,b,d,e FROM table1 HMAP first,second,third,fourth WHERE a = 1 FLAG PHD SUP",
            OutOption::Term,
        )?;

        Ok(())
    }
}
