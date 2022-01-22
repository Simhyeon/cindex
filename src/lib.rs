/// # Cindex, a csv indexer
/// 
/// Cindex is a easy to use csv indexer with SQL-like simple query support.
/// 
/// Cindex is not intended for heavy database indexing but for simple in-memory
/// querying. Use other databases interaction layer if you're using big chunks of
/// csv files.
/// 
/// # Usage
/// 
/// ```toml
/// [dependencies]
/// cindex = "*" # Use the latest version if possible
/// 
/// # Use "default-features = false" if you don't need rayon iteration enhancement.
/// ```
/// 
/// ```no_run
/// use std::fs::File;
/// use cindex::{Indexer, CsvType, Predicate, Query, OutOption, Operator};
/// 
/// let mut indexer = Indexer::new();
/// 
/// // Add table without types
/// // Default types for every columns are "Text"
/// indexer.add_table_fast(
///     "table1", 
///     File::open("test.csv").expect("Failed to open a file")
/// ).expect("Failed to add table");
/// 
/// // Add table with column types
/// indexer.add_table(
///     "table2", 
///     vec![CsvType::Text, CsvType::Text],
///     "id,address
/// 10,110-2222".as_bytes()
/// ).expect("Failed to add table");
/// 
/// // Add table from stdin
/// let stdin = std::io::stdin();
/// indexer.add_table_fast(
///     "table3", 
///     stdin.lock()
/// ).expect("Failed to add table");
/// 
/// // Indexing
/// 
/// // Create query object and print output to terminal
/// let query = Query::from_str("SELECT a,b,c FROM table1 WHERE a = 10").expect("Failed to create query");
/// indexer.index(query, OutOption::Term).expect("Failed to index a table");
/// 
/// // Use raw query and yield output to a file
/// indexer.index_raw(
///     "SELECT * FROM table3 WHERE id = 10", 
///     OutOption::File(std::fs::File::create("out.csv").expect("Failed to create a file"))
/// ).expect("Failed to index a table");
/// 
/// // Use builder pattern to construct query and index a table
/// let query = Query::empty("table2")
///     .columns(vec!["id", "address"])
///     .predicate(Predicate::new("id", Operator::Equal)
///         .args(vec!["10"])
///     )
///     .predicate(Predicate::build()
///         .column("address")
///         .operator(Operator::NotEqual)
///         .raw_args("111-2222")
///     );
/// 
/// let mut acc = String::new();
/// indexer.index(query, OutOption::Value(&mut acc)).expect("Failed to index a table");
///
/// // Disable header print if you want
/// indexer.set_print_header(false);
///
/// // Always use unix newline for formatting
/// indexer.always_use_unix_newline(true);
/// ```
///
/// # Query
///
/// Cindex's query syntax is similar to SQL but has some small differences.
/// 
/// ```SQL
/// /* Select everythig from given table*/
/// SELECT * FROM table1
/// 
/// /* Select everything from given table and order by column with descending
/// order*/
/// SELECT * FROM table1 ORDER BY col1 DESC
///
/// /* Same with previous commands but map headers into custom values */
/// SELECT * FROM table1 ORDER BY col1 DESC HMAP new_h,new_h2,new_h3
/// 
/// /* -> Previous query result header with underbar substituted with whitespaces
/// new h,new h2,new h3
/// <-- Content goes here -->
///  */
/// 
/// /* Select given columns from table where column's value is equal to given
/// condition and also other column's value matches regex expression */
/// SELECT col1,col2 FROM table1 WHERE col1 = 10 AND col2 LIKE ^start
/// ```
///
/// ```markdown
///  >= 
///  >
///  <=
///  <
///  =
///  !=
///  IN ( enumerate )
///  BETWEEN (inclusive range of min & max)
///  LIKE ( with regeular expression )
/// ```

mod indexer;
#[cfg(test)]
mod test;
pub mod models;
mod parser;
mod error;
mod consts;

pub use indexer::{Indexer, OutOption};
pub use models::{CsvType, Predicate, Operator, Query};
pub use error::{CIndexError, CIndexResult};
