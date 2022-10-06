# <img src="https://github.com/singlestore-labs/singlestore-python/blob/main/resources/singlestore-logo.png" height="60" valign="middle"/> SingleStoreDB Categorical Analysis

This project contains Rust code and SQL code to perform basic statistical analysis on categorical data. The first module will compute the Chi-square test of independence between two categorical variable. It returns the Chi-square test statistic, its degrees of freedom, and the p-value for the hypothesis of independence.

The data for these analyses comes in two forms: (i) as raw data that needs to be classified according to two variables, (ii) as cross-classified data where each row of the table contains the number of observations that share the same level of the two categorical variables. Data organized according to (ii) is sometimes called grouped data or data with frequency weight; acccordingly, data organized as in (i) is called raw data or ungrouped data. You perform the Chi-square test for ungrouped data by calling the `chisq( )` stored procedure. For example

```
    echo chisq('titanic','Survived','Pclass');  
```
performs a Chi-square test of independence between the variables Survival and Passenger class for the titanic passenger data.

You perform the analysis for grouped data by calling the `chisq_grouped( )` procedure. For example, 

```
create table if not exists test.employee_sat (
    EmpClass text, Opinion text, Nij bigint(20)
);
delete from test.employee_sat;
insert into test.employee_sat(EmpClass, Opinion, Nij) values 
("Staff"        , "Favor"       , 30),
("Staff"        , "Do not Favor", 15),
("Staff"        , "Undecided"   , 15),
("Faculty"      , "Favor"       , 40),
("Faculty"      , "Do not Favor", 50),
("Faculty"      , "Undecided"   , 10),
("Administrator", "Favor"       , 10),
("Administrator", "Do not Favor", 25),
("Administrator", "Undecided"   ,  5)
;

echo chisq_grouped('employee_sat','EmpClass','Opinion',"Nij")

```
caluclate the Chi-square test of independence between Employee Classification and Employee Opinion. The number of employees that had a particular opinion in a particular employee group are captured in the Nij column.'

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
Suppose the `pushwasm` executable has been copied to the base directory of your repo and the connection string for the SingleStoreDB database (`mysql://username:password@dbhostname:3306/mydatabase`) is stored in environment variable `SINGLESTOREDB_CONNSTRING`. The following commands, for example, create the user-defined functions chisq_init, chisq_iter, chisq_merge, and chisq_term in the database based on a release build:

```bash
./pushwasm --force $SINGLESTOREDB_CONNSTRING \
   --wit ./categorical.wit ./target/wasm32-wasi/release/categorical.wasm chisq_init

./pushwasm --force $SINGLESTOREDB_CONNSTRING \
   --wit ./categorical.wit ./target/wasm32-wasi/release/categorical.wasm chisq_iter

./pushwasm --force $SINGLESTOREDB_CONNSTRING \
   --wit ./categorical.wit ./target/wasm32-wasi/release/categorical.wasm chisq_merge

./pushwasm --force $SINGLESTOREDB_CONNSTRING \
   --wit ./categorical.wit ./target/wasm32-wasi/release/categorical.wasm chisq_term
```
The file `pushwasm.txt` contains all `pushwasm` commands that for this module.

### Create aggregate functions (UDA) and stored procedures in the database

Run the file `categorical.sql` in SingleStoreDB. 

## Resources

* [SingleStore](https://singlestore.com)
* [Documentation](https://docs.singlestore.com)
* [Twitter](https://twitter.com/SingleStoreDevs)
* [SingleStore forums](https://www.singlestore.com/forum)
