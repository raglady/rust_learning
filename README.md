# USER CRUD
The purpose of this project is to learn rust programming language.

## How to configure it ?
There is two config file: `local.yml` and `test.yml`
The first is for local running and the last is for test.

## Supported database driver
Supported database driver are: `sqlite`, `mysql`, and `postgresql`

## How to run ?
After configuring the app, you can run using:
```
cargo run
```

## How to access to swagger ?
You can get the swagger by opening the url:
```
http://localhost:8000/api/docs/
```
Don't forget the `/` in the end of the url


## How to test ?
To launch simple test, you can run:
```
cargo test
```

If you would like to run the coverage test, you need tarpaulin or grcov.

For this test I use tarpaulin.

Install tarpaulin:
```
cargo install cargo-tarpaulin
```

Run the test with html visualisation:
```
cargo tarpaulin --out html --engine llvm
```
(Optional) You can set the job numbers if needed:
```
cargo tarpaulin --out html --engine llvm -j 4
```

## Issues
When running coverage test, some function are called but not in the current code,
and these function are not included in coverage test.
For example: `api/src/error.rs` line 13.
