#[cfg(test)]
mod tests {
    use std::fs::File;
    use crate::{indexer::{Indexer, OutOption}, models::{Query, CSVType}};

    #[test]
    fn it_works() {
        println!("1");
        let mut indexer = Indexer::new();
        indexer.add_table_fast(
            "table1", 
            File::open("test.csv").expect("Failed to open")
        );
        indexer.add_table_fast(
            "table2", 
            "Test\n1\n2".as_bytes()
        );
        //let stdin = std::io::stdin();
        //indexer.add_table_fast(
            //"table3", 
            //stdin.lock()
        //);
        println!("Without queries");
        indexer.index(Query::empty("table1"), OutOption::Term);
        let mut result = String::new();
        indexer.index(Query::empty("table2"), OutOption::Value(&mut result));
        indexer.index_raw("SELECT a,b,c FROM table1 WHERE a = 0.1 JOIN ee", OutOption::Term);
        indexer.index_raw("SELECT * FROM table3 WHERE a = 0.1", OutOption::File(std::fs::File::create("out.csv").expect("Failed to open")));
    }
}

