# 0.5.2-rc1

Upgraded regex version

# 0.5.1 Hotfix

- BUG : Query panicked on multiline source
- CHG : Improved error handling inside query building.

# 0.5.0

- CHG : Changed method names
- CHG : Removed empty method for query
- CHG : Table creation's failure now propagates properly
- CHG : Ditches dependencies without losing feature
- FET : More table creation options + re-export reader option
- FET : Removed double quotes termination
- FET : Asterisk can come after fisrt select argument
- FET : Enabled quotes in query statment which enables whitespace input
- BUG : Select column didn't affect headers
- BUG : Hmap didn't work
- BUG : Order by with single argument panicked... 

# 0.4.1

- Intern : Ditched several dependencies in favor of [dcsv](https://crates.io/crate/dcsv)
crate.
- Change : Disabled add\_table for dcsv compatibility. Automatic limiter setting will be
added later.
- Ergono : Improved "like" syntax performance with precompiled regex
