# A toy payments processing engine

`cargo run -- -h` to see help

`cargo run -- $PATH_TO_CSV` to run engine over csv data

`cargot test` to run tests

## Design:

- doesn't support cuncurrent execution for multiple csv's
- every transaction is encapsulated into its data type to decouple processing logic from the engine
- funds are stored as rational numbers
- dispute is only possible for deposit/withdrawn transaction
- only deposit/withdrawn transactions are stored in memory, with status flag, telling if the tx is being disputed or charged back

## Improvements:

- add error handling (skipped for the demo)
- add handling for invalid events since current logic ignores them (with some warnings for observability)
- test coverage is minimal, but it shows areas that should increase code coverage (transactions logic, parsing logic)
