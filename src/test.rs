#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use crate::{indexer::{Indexer, ReadOption}, models::{Query, Predicate}};

    #[test]
    fn it_works() {
        let mut indexer = Indexer::new();
        indexer.add_table(
            "table1", 
            ReadOption::File(PathBuf::from("test.csv"))
        );
        println!("Without queries");
        // Non query test
        indexer.index("table1", None);
        // Query test
        println!("With queries");
        let query = Query::new(Some(vec![Predicate::new("a", crate::models::Operation::BiggerOrEqual).raw_args("1")]), None);
        indexer.index("table1", Some(query));
        println!("----------");
    }
}

