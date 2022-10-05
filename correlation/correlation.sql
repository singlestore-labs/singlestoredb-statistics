
/* 
use test;

select species, corr2d(sepal_width,sepal_length) from iris group by species;
+------------+----------------------------------+
| species    | corr2d(sepal_width,sepal_length) |
+------------+----------------------------------+
| virginica  |              0.45722781639411025 |
| versicolor |               0.5259107172828018 |
| setosa     |                0.746780373263978 |
+------------+----------------------------------+

select species, corr2(sepal_width,sepal_length) from iris group by species;
+------------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| species    | corr2(sepal_width,sepal_length)                                                                                                                                                                    |
+------------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+
| virginica  | {"b0":3.9068364663866495,"b1":0.9015344766689136,"corr":0.45722781639411025,"n":50,"nmiss":0,"r2":0.20905727608452618,"sse":15.6707900003937,"x_avg":2.9739999999999998,"y_avg":6.587999999999998} |
| versicolor | {"b0":3.53973471502595,"b1":0.865077720207238,"corr":0.5259107172828018,"n":50,"nmiss":0,"r2":0.27658208255291106,"sse":9.444365595855574,"x_avg":2.7700000000000005,"y_avg":5.936}                |
| setosa     | {"b0":2.6446596755600273,"b1":0.6908543956816768,"corr":0.746780373263978,"n":50,"nmiss":0,"r2":0.5576809258922863,"sse":2.692926986982545,"x_avg":3.4180000000000006,"y_avg":5.005999999999999}   |
+------------+----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------+

select corrmat(vec_pack_f64([sepal_length, sepal_width, petal_width])) from test.iris;
|+-----------------------------------------------------------------------+
| [1,-0.10936924995067457,1,0.81795363336917759,-0.35654408961382239,1] |
+-----------------------------------------------------------------------+

select corrmat(vec_pack_f64([sepal_length, sepal_width, petal_length, petal_width])) from test.iris;
+--------------------------------------------------------------------------------------------------------------------------------------+
| [1,-0.10936924995067457,1,0.87175415730488537,-0.42051609640118814,1,0.81795363336917759,-0.35654408961382239,0.96275709705096546,1] |
+--------------------------------------------------------------------------------------------------------------------------------------+

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

