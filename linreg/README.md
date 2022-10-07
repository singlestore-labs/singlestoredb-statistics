# <img src="https://github.com/singlestore-labs/singlestore-python/blob/main/resources/singlestore-logo.png" height="60" valign="middle"/> SingleStoreDB Regression Analysis

This project contains Rust code and SQL statements to perform basic linear regression modeling.

## Steps

The basic steps to extend SingleStoreDB with the capabilities in this package are as follows:

1. Compile the Rust crate to Wasm
2. Push the functions in the `categorical.wasm` module to the database
3. Run the SQL commands in `categorical.sql` to create aggregate functions and stored procedures.
4. Run SQL queries that use the stored procedures.

### Compiling

```
  /* for debug build */
  cargo wasi build

  /* for optimized build */
  cargo wasi build --release
```
### Push UDFs to the database
Suppose the `pushwasm` executable has been copied to the base directory of your repo and the connection string for the SingleStoreDB database (`mysql://username:password@dbhostname:3306/mydatabase`) is stored in environment variable `SINGLESTOREDB_CONNSTRING`. The following command, for example, creates the user-defined functions mlr_init in the database based on the release build:

```
./pushwasm --force $SINGLESTOREDB_CONNSTRING \
   --wit ./linreg.wit ./target/wasm32-wasi/release/linreg.wasm \
   mlr_init
```
See the file `pushwasm.txt` for the commands for all functions in this module.

### Create aggregate functions (UDA) and helper UDFs in the database

Run the file `linreg.sql` in SingleStoreDB to create the UDAs and helper UDFs.

See  `schema.sql` for an example.

## Resources

* [SingleStore](https://singlestore.com)
* [Documentation](https://docs.singlestore.com)
* [Twitter](https://twitter.com/SingleStoreDevs)
* [SingleStore forums](https://www.singlestore.com/forum)


