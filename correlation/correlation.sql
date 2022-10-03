
/* 
use test;

select species, corr2d(sepal_width,sepal_length) from iris group by species;
select species, corr2(sepal_width,sepal_length) from iris group by species;
select corrmat(vec_pack_f64([sepal_length, sepal_width, petal_width])) from test.iris;
select corrmat(vec_pack_f64([sepal_length, sepal_width, petal_length, petal_width])) from test.iris;
*/

/* Create a table for the Iris data set  from a CSV file */
create table if not exists 
    iris(sepal_length double, sepal_width double, petal_length double, petal_width double, species varchar(40));
delete from iris;
load data local infile './data/iris.csv'
     fields terminated by ','
     NULL defined by ''
     IGNORE 1 lines
     replace into table test.iris;



/*---------------------------------------------------------------*/
/* Helper UDF to translate the return struct from corr2_term to JSON
/*---------------------------------------------------------------*/
delimiter //
create or replace function corr_term_json(
    state record (
        wrk  array(double not null),
        n    array(bigint(20) not null)) not null)
    returns json not null as
begin
    return to_json(`corr2_term`(state));
end //
delimiter ;


/*---------------------------------------------------------------*/
/* This version returns a JSON object that describes the relation-
/* ship between the two variables in detail, including simple linear
/* regression statistics.
/*---------------------------------------------------------------*/
create or replace aggregate corr2(double NOT NULL, double NOT NULL)
    returns json not null
    with state record (
        wrk  array(double not null),
        n    array(bigint(20) not null)
    ) not null
    initialize with `corr2_init`
    iterate with `corr2_iter`
    merge with `corr2_merge`
    terminate with corr_term_json;

/*---------------------------------------------------------------*/
/* A simpler version that returns just the correlation coefficient
/*---------------------------------------------------------------*/
create or replace aggregate corr2d(double NOT NULL, double NOT NULL)
    returns double not null
    with state record (
        wrk  array(double not null),
        n    array(bigint(20) not null)
    ) not null
    initialize with `corr2_init`
    iterate    with `corr2_iter`
    merge      with `corr2_merge`
    terminate  with `corr2_termd`;


/*---------------------------------------------------------------*/
/* Helper UDF to translate the return blob from corrmat_term     */
/* into a JSON double array.                                      */
/*---------------------------------------------------------------*/
delimiter //
create or replace function corrmat_term_json (
    state record (
        nvar bigint(20) not null,
        sx  array(double not null),
        sy  array(double not null),
        sxx array(double not null),
        syy array(double not null),
        sxy array(double not null),
        nxy array(double not null)) not null)
    returns json not null as
begin
    return json_array_unpack_f64(`corrmat_term`(state)); 
end //
delimiter ;


create or replace aggregate corrmat(blob not null) 
    returns json not null
    with state record (
        nvar bigint(20) not null,
        sx  array(double not null),
        sy  array(double not null),
        sxx array(double not null),
        syy array(double not null),
        sxy array(double not null),
        nxy array(double not null)
     ) not null
    initialize with `corrmat_init`
    iterate    with `corrmat_iter`
    merge      with `corrmat_merge`
    terminate  with corrmat_term_json;

