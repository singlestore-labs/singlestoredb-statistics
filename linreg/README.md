# <img src="https://github.com/singlestore-labs/singlestore-python/blob/main/resources/singlestore-logo.png" height="60" valign="middle"/> SingleStoreDB Regression Analysis

This project contains Rust code and SQL statements to perform basic linear regression modeling and analysis of variance for fixed-effects models.

## Steps

The basic steps to extend SingleStoreDB with the capabilities in this package are as follows:

1. Compile the Rust crate to Wasm
2. Push the functions in the `linreg.wasm` module to the database
3. Run the SQL commands in `linreg.sql` to create aggregate functions and stored procedures.
4. Run SQL queries that use the UDAs and/or stored procedures.

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

### Run analyses through SQL queries

See  `schema.sql` for regression examples using the `slr()` function for simple linear regression, the `mlr()` function for multiple linear regression with short output (just the regression coefficients), and the `mlrl()` function for multiple linear regression with detailed output. 

```
select mlr(oxygen,vec_pack_f64([Age, Weight, RunTime, RestPulse, RunPulse, MaxPulse])) from fitness;

/* Long output, small model                                     */
select mlrl(oxygen,vec_pack_f64([Age, Weight])) from fitness;

/* Short output: just the regression coefficients               */
select mlr(oxygen,vec_pack_f64([Age, Weight])) from fitness;

/* Simple linear regression using the slr() and mlr() functions */
select slr(oxygen,Age) from fitness;
select slr(oxygen,Age) from fitness group by agegroup;

select mlr(oxygen,vec_pack_f64([Age])) from fitness; 
select mlrl(oxygen,vec_pack_f64([Age])) from fitness; 
```

To run an analysis of variance, use the `aov()` stored procedure (created in `linreg.sql`).
For example, the analysis of variance for Survival with factors SibSp and Pclass in the titanic data set is invoked with

```
echo aov('titanic','Survived',['SibSp','Pclass']);
```


## Resources

* [SingleStore](https://singlestore.com)
* [Documentation](https://docs.singlestore.com)
* [Twitter](https://twitter.com/SingleStoreDevs)
* [SingleStore forums](https://www.singlestore.com/forum)


