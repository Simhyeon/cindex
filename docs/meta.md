### TODO

* [ ] Support quotes in query so that user can utilize whitespace

* [ ] To think about it... I think "operate" function was inherently failure.
Since it always require the query syntax to expect column name first.
	- 'SELECT * FROM table WHERE col-name = hello' works but,
	- 'SELECT * FROM table WHERE hello = col-name' doesn't work
Is this ok? Hmm... I think it might be ok. The reasons are
	- People write the first way anyway.
	- Cindex is not SQL, thus query syntax can differ, because why not.

* [ ] Join functionality
* [ ] <OR> variant for predicate
- Currently all predicate are AND variant
* [ ] Evaluation process would be useful
* [x] Transpose
* [x] Offset, Limit
* [x] Module separation
* [ ] Count, average, sum
	* [ ] This is technically a sql function support

### DONE

* [x] In and between support for raw query
* [x] Like support
* [x] Disable rayon feature
* [x] Header also select columns
* [x] Find usage of header\_types
* [x] Error handling
* [x] CSVData type checking
* [x] Order by
* [x] Possibly windows "\r\n" option
* [x] COLUMN mapping
* [x] Droptable
* [x] Contains
* [x] Set a default order by option for ergonomics
* [x] Supplment syntax
* [x] More ergonomic print header syntax
* [x] Support quote rule
	- Kinda..., It simply removes all double quotes on real operation
* [x] Use dcsv cate for better csv spec and better maintainability
* [x] Ditch thiserror + indexmap
* [x] Make filter non panicking
