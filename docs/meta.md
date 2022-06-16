### TODO

* [ ] Make OPERATE order consistent
	- To work lv and rv's order doesn't matter

* [ ] Join functionality
* [ ] <OR> variant for predicate
- Currently all predicate are AND variant
* [ ] Evaluation process would be useful
* [ ] Count, average, sum
	* [ ] This is technically a sql function support
* [ ] Index method to query with indexes
	e.g. ) SELECT IN(2)

### DONE

**0.5.1**

* [x] Multi-line query construction

Prior 0.5.0

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
* [x] Transpose
* [x] Offset, Limit
* [x] Module separation

0.5.0

* [x] Reexport dcsv reader option
* [x] Made constructor's error handable
* [x] Remove empty method for query because it is useless for most cases
* [x] Ditch bigut flags : it's lightweight, but cindex can be more lighter
* [x] Custom header would be good
* [x] Fix Dcsv port bugs
	* [x] FET : Removed double quotes termination
	* [x] BUG : Currently select column is broken for headers
	* [x] BUG : Hmap doesn't work
	* [x] FET : Make * detected among select args
	* [x] FET : Wildcard can be applied in between because why not?
* [x] Support quotes in query so that user can utilize whitespace
