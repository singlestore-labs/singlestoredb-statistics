
# <img src="https://github.com/singlestore-labs/singlestore-python/blob/main/resources/singlestore-logo.png" height="60" valign="middle"/> SingleStoreDB Correlation Analysis

This project contains Rust code that computes the Pearson product-moment correlation coefficient between
two numeric variables or the matrix of all pairwise Pearson product-moment correlation coefficients
between a set ( >=2 ) of numeric variables.

The Rust code can be compiled to Wasm (WebAssembly) and registered with the SingleStoreDB database as
user-defined aggregate (UDA) functions. The SQL statements to create the functions in the database is provided.

## Setup

1. [Sign up](https://www.singlestore.com/try-free/) for a free SingleStore license. This allows you
   to run up to 4 nodes up to 32 gigs each for free. Grab your license key from
   [SingleStore portal](https://portal.singlestore.com/?utm_medium=osm&utm_source=github) and set it as an environment
   variable.

   ```bash
   export SINGLESTORE_LICENSE="singlestore license"
   ```

2. If you want to develop on SingleStoreDB, check out the [singlestoredb-dev-image](https://github.com/singlestore-labs/singlestoredb-dev-image) repo

3. Install the [SingleStoreDB Wasm Toolkit](https://github.com/singlestore-labs/singlestore-wasm-toolkit). This provides the WRIT utility to help test Wasm functions locally without the need to create a separate driver program. It also provide pushwasm, a utility that allows you to easily import your locally-built Wasm function into SingleStoreDB as a UDF or TVF.

## Steps
The basic steps to extend SingleStoreDB with the capabilities in this package are as follows:

1. Compile the Rust crate to Wasm
2. Push the functions in the `correlation.wasm` module to the database
3. Create aggregate functions (UDAs) in the database
4. Run a SQL query that calls one of the UDAs.

## Compiling

Run:
```
  /* for debug build */
  cargo wasi build
  /* for optimized build */
  cargo wasi build --release
```

## Push UDFs to the database
Suppose the `pushwasm` executable has been copied to the base directory of your repo and the connection string for the SingleStoreDB database (`mysql://username:password@dbhostname:3306/mydatabase`) is stored in environment variable `SINGLESTOREDB_CONNSTRING`. The following command, for example, creates the user-defined functions corr2_init and corr_iter in the database:

```bash
./pushwasm --force $SINGLESTOREDB_CONNSTRING \
   --wit ./correlation.wit ./target/wasm32-wasi/release/correlation.wasm \
   corr2_init

./pushwasm --force $SINGLESTOREDB_CONNSTRING \
   --wit ./correlation.wit ./target/wasm32-wasi/release/correlation.wasm \
   corr2_iter
```
See the file `pushwasm.txt` for a list of `pushwasm` commands that install all necessary functions.

## Create aggregate functions (UDA) in the database

Check the file `correlation.sql` for the CREATE FUNCTION / AGGREGATE calls. For pairwise correlations there are two aggregate functions depending on whether you want just the correlation coefficient or a more elaborate result that describes the relationship between the two variables in detail--the result is returned as a JSON array in this case.

## SQL examples

Compute the correlation between `sepal_width` and `sepal_length`, return only the correlation coefficient.
```
select species, corr2d(sepal_width,sepal_length) from iris;
```

Compute the correlation between `sepal_width` and `sepal_length` for each iris species and return the results in a JSON array.
```
select species, corr2(sepal_width,sepal_length) from iris group by species;
```

Compute the (lower triangular portion) of the matrix of correlation coefficients between variables `sepal_length`, `sepal_width`, and `petal_width`. This returns a JSON double array of lenght 6.
```
select corrmat(vec_pack_f64([sepal_length, sepal_width, petal_width])) from iris;

```

## Resources

* [SingleStore](https://singlestore.com)
* [Documentation](https://docs.singlestore.com)
* [Twitter](https://twitter.com/SingleStoreDevs)
* [SingleStore forums](https://www.singlestore.com/forum)
