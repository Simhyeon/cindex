#[cfg(test)]
mod tests {
    use crate::indexer::{Indexer, OutOption};
    use crate::ReaderOption;
    use crate::{CIndexResult, Operator, Predicate, Query};
    use std::fs::File;
    use std::io::BufReader;

    #[test]
    fn test_function() -> CIndexResult<()> {
        let mut indexer = Indexer::new();

        // Table 1
        // "LatD", "LatM", "LatS", "NS", "LonD", "LonM", "LonS", "EW", "City", "State"
        //indexer.add_table_fast("t1", File::open("/home/simon/misc/csv_samples/cities.csv")?)?;
        //indexer.index_raw(r#"SELECT "LatD",' "LonM"',' "City"',' "State"' FROM t1 FLAG PHD"#, OutOption::Term)?;

        // Table 2
        // John,Doe,120 jefferson st.,Riverside, NJ, 08075
        //indexer.add_table(
        //"t2",
        //BufReader::new(File::open("/home/simon/misc/csv_samples/addresses.csv")?),
        //)?;
        //indexer.index_raw(
        //r#"SELECT John,Doe,'120 jefferson st.','fuck yeah' FROM t2 FLAG PHD TP SUP"#,
        //OutOption::Term,
        //)?;

        // Table 3
        // HHH
        //indexer.add_table(
        //"t3",
        //BufReader::new(File::open("/home/simon/misc/csv_samples/example.csv")?),
        //)?;
        //indexer.index_raw(r#"SELECT * FROM t3 FLAG PHD"#, OutOption::Term)?;

        // Table 4
        //
        let mut reader_option = ReaderOption::new();
        reader_option.ignore_empty_row = true;
        reader_option.trim = true;
        reader_option.consume_dquote = true;
        indexer.add_table_with_option(
            "t4",
            BufReader::new(File::open("/home/simon/misc/csv_samples/biostats.csv")?),
            reader_option,
        )?;
        indexer.add_table(
            "t1",
            "id,first name,last name,address
1,John,doe,AA 1234
2,Janet,doner,BB 4566
3,Hevay,jojo,CC 8790"
                .as_bytes(),
        )?;
        indexer.index_raw(
            r#"SELECT 'first name'
            'last name' 
            FROM 
            t1 
            FLAG
            PHD"#,
            OutOption::Term,
        )?;
        //indexer.index_raw(
        //r#"SELECT * FROM t4 ORDER BY 'Height (in)' desc OFFSET 1 FLAG PHD WHERE 'Height (in)' = 66"#,
        //OutOption::Term,
        //)?;

        Ok(())
    }
}
