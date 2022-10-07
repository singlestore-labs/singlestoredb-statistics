

/*---------------------------------------------------------------*/
/* Helper UDF to translate the return struct from mlr_term to JSON
/*---------------------------------------------------------------*/
delimiter //
create or replace function mlr_term_json(
    state record (
        nvars bigint(20) not null,
        xpx array(double not null)) not null)
    returns json not null as
begin
    return json_array_unpack_f64(`mlr_term`(state));
end //
delimiter ;


delimiter //
create or replace function mlr_term_json2(
    state record (
        nvars bigint(20) not null,
        xpx array(double not null)) not null)
    returns json not null as
begin
    return json_array_unpack_f64(`mlr_terml`(state));
end //
delimiter ;


create or replace aggregate mlr(double not null, blob not null) 
    returns json not null
    with state record (
        nvars bigint(20) not null,
        xpx  array(double not null)) not null
    initialize with `mlr_init`
    iterate    with `mlr_iter`
    merge      with `mlr_merge`
    terminate  with mlr_term_json;


create or replace aggregate mlrl(double not null, blob not null) 
    returns json not null
    with state record (
        nvars bigint(20) not null,
        xpx  array(double not null)) not null
    initialize with `mlr_init`
    iterate    with `mlr_iter`
    merge      with `mlr_merge`
    terminate  with mlr_term_json2;

