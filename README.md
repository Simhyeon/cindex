# Cindex, a csv indexer

Cindex is a easy to use csv indexer with a simple SQL-like query support.

Cindex is not intended for heavy database indexing but for simple in-memory
querying. Use other databases interaction layer if you're using big chunks of
csv files.

Binary to be added soon.

# Features

**Read csv file as string table**

No mmap, no bufstream no nothing. If 'read' is ok, then table content is ok
through the whole indexing.

**Index csv table with SQL-like query**

Sometimes you want raw way to index table and get raw value from it. Just pipe
csv table to other programs and maybe with an optional csv header.

**Human friendly csv reading**

Writing csv values by hand is not so trivial yet there are some cases when you
have to. Missing values can be allowed with cindex, even missing columns can be
allowed with specific FLAG syntax. Missing comma, is not allowed though.

# Usage

```toml
[dependencies]
cindex = "*" # Use the latest version if possible

# Use "default-features = false" if you don't need rayon iteration enhancement.
```

```rust
use std::fs::File;
use cindex::{Indexer, CsvType, Predicate, Query, OutOption, Operator};

let mut indexer = Indexer::new();

// Add table without types
// Default types for every columns are "Text"
indexer
    .add_table_fast(
        "table1",
        File::open("test.csv").expect("Failed to open a file"),
    )
    .expect("Failed to add table");

// Add table with column types
indexer
    .add_table(
        "table2",
        vec![CsvType::Text, CsvType::Text],
        "id,address
abc,111-2222"
            .as_bytes(),
    )
    .expect("Failed to add table");

// Add table from stdin
let stdin = std::io::stdin();
indexer
    .add_table_fast("table3", stdin.lock())
    .expect("Failed to add table");

// Indexing

// Create query object and print output to terminal
let query = Query::from_str("SELECT a,b,c FROM table1 WHERE a = 10");
indexer
    .index(query, OutOption::Term)
    .expect("Failed to index a table");

// Use raw query and yield output to a file
indexer
    .index_raw(
        "SELECT * FROM table3 WHERE id = 10",
        OutOption::File(std::fs::File::create("out.csv").expect("Failed to create a file")),
    )
    .expect("Failed to index a table");

// Use builder pattern to construct query and index a table
let query = Query::empty("table2")
    .columns(vec!["id", "address"])
    .predicate(Predicate::new("id", Operator::Equal).args(vec!["10"]))
    .predicate(
        Predicate::build()
            .column("address")
            .operator(Operator::NotEqual)
            .raw_args("111-2222"),
    );

let mut acc = String::new();
indexer
    .index(query, OutOption::Value(&mut acc))
    .expect("Failed to index a table");

// Always use unix newline for formatting
indexer.always_use_unix_newline(true);
```
# Query syntax

Cindex's query syntax is similar to SQL but has some small differences.

```SQL
/* Select everythig from given table*/
SELECT * FROM table1

/* Select everything from given table and order by column with descending
order*/
SELECT * FROM table1 ORDER BY col1 DESC

/* Same with previous commands but map header result */
SELECT * FROM table1 ORDER BY col1 DESC HMAP new_h,new_h2,new_h3

/* -> Previous query result header with underbar substituted with whitespaces
new h,new h2,new h3
<-- Content goes here -->
 */

/* You can use OFFSET and LIMIT syntax to control how much lines to print*/
/* Keep in mind that this doesn't early return from indexing, but it works as
   final_records[offset..offset+limit] */
/* e.g. next line gets records[1..3] */
SELECT * FROM table1 OFFSET 1 LIMIT 2

/* Select given columns from table where column's value is equal to given
condition and also other column's value matches regex expression */
SELECT col1,col2 FROM table1 WHERE col1 = 10 AND col2 LIKE ^start

/* There is a flag syntax which changes query behaviour*/
SELECT * FROM table_name FLAG PHD SUP

/* In this case each flag does next operation
  - PHD (PRINT-HEADER) : Print header in result output
  - SUP (SUPPLEMENT)   : Enable selection of non-existent column with empty values
  - TP  (Transpose)    : Transpose(Invert) csv value as of linalg
 */
```

Supported WHERE operations are

```
 >= 
 >
 <=
 <
 =
 !=
 IN
 BETWEEN
 LIKE ( with regeular expression )
```

# TODO

* [ ] DateTime csv type
* [ ] Multi where caluse support
* [ ] Join table
