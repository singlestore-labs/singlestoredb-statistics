

/*---------------------------------------------------------------*/
/* Helper UDF to translate the return struct from mlr_term to JSON
/*---------------------------------------------------------------*/
delimiter //
create or replace function mlr_term_json (
    state record (
        nvars bigint(20) not null,
        xpx array(double not null)) not null)
    returns json not null as
begin
    return json_array_unpack_f64(`mlr_term`(state));
end //
delimiter ;


delimiter //
create or replace function slr_term_json (
    state record (xpx array(double not null)) not null)
    returns json not null as
begin
    return to_json(`slr_term`(state));
end //
delimiter ;


delimiter //
create or replace function aov_term_json(
    state record (
        efflev array(bigint(20) not null),
        xpx    array(double not null)) not null)
    returns json not null as
begin
    return json_array_unpack_f64(`aov_term`(state));
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


create or replace aggregate slr(double not null, double not null) 
    returns json not null
    with state record (xpx array(double not null)) not null
    initialize with `slr_init`
    iterate    with `slr_iter`
    merge      with `slr_merge`
    terminate  with slr_term_json;


create or replace aggregate mlrl(double not null, blob not null) 
    returns json not null
    with state record (
        nvars bigint(20) not null,
        xpx  array(double not null)) not null
    initialize with `mlr_init`
    iterate    with `mlr_iter`
    merge      with `mlr_merge`
    terminate  with mlr_term_json2;

/*--------------------------------------------------------------*/
/* The aggregate function for the fixed-effect analysis of      */
/* variance.                                                    */
create or replace aggregate aov_agg(blob not null) 
    returns json not null
    with state record (
        efflev array(bigint(20) not null),
        xpx    array(double not null)) not null
    initialize with `aov_init`
    iterate    with `aov_iter`
    merge      with `aov_merge`
    terminate  with aov_term_json;
/*--------------------------------------------------------------*/

/*--------------------------------------------------------------*/
/* aov_debug() generates the query string that performs the     */
/* fixed-effects analysis of variance.                          */

delimiter //
create or replace procedure aov_debug(tbl     text, 
                                      target  text, 
                                      factors array(text))
returns text
as declare
    qstr text;
begin
    qstr = 'select aov_agg(vec_pack_f64([ ';
    for i in 0 .. length(factors)-1 loop
        qstr = concat(qstr,'NumX',i,',');
    end loop;
    qstr = concat(qstr,'Target,');
    for i in 0 .. length(factors)-1 loop
        if i > 0 then 
            qstr = concat(qstr,', ');
        end if;
        qstr = concat(qstr,'X',i,'Level');   
    end loop;

    qstr = concat(qstr,'])) from ( with foo as ( select ');
    qstr = concat(qstr,target,' as "Target", ' );
    for i in 0 .. length(factors)-1 loop
        if i > 0 then 
            qstr = concat(qstr,', ');
        end if;
        qstr = concat(qstr, factors[i],' as X',i);
    end loop;
    qstr = concat(qstr,' from ',tbl,' where ', target,' is not null and ');
    for i in 0 .. length(factors)-1 loop
        if i > 0 then 
            qstr = concat(qstr,' and ');
        end if;
        qstr = concat(qstr,factors[i],' is not null');
    end loop;
    qstr = concat(qstr,'),');

    /*--- loop over the factors and add the main effects ---*/
    for i in 0 .. length(factors)-1 loop
        if i > 0 then
            qstr = concat(qstr,', ');
        end if;
        qstr = concat(qstr,'X',i,'_counts as (' );
        qstr = concat(qstr,'select X',i, ', Row_Number() over(order by X',i,') as X',i,'Level');
        qstr = concat(qstr,' from foo group by X',i,')');
    end loop;

    qstr = concat(qstr,' select ');
 
    for i in 0 .. length(factors)-1 loop
        qstr = concat(qstr,'(select count(distinct X',i,') :> DOUBLE from foo) as NumX',i,',');
    /*    qstr = concat(qstr,'t0.X',i,' :> DOUBLE,'); */
    end loop;

    qstr = concat(qstr,'t0.Target :> DOUBLE as "Target", ');
    
    for i in 0 .. length(factors)-1 loop
        if i > 0 then
            qstr = concat(qstr,', ');
        end if;
        qstr = concat(qstr,'t',i+1,'.X',i,'Level :> DOUBLE as X',i,"Level");
    end loop;
    
    qstr = concat(qstr,' from foo as t0 ');
    for i in 0 .. length(factors)-1 loop
        qstr = concat(qstr,' inner join X',i,'_counts as t',i+1,' on t',i+1,'.X',i,' = t0.X',i);
    end loop;
    qstr = concat(qstr,') option(materialize_ctes="off")');
    qstr = concat(qstr,';');

    return qstr;
end //
delimiter ;

/*
echo aov_debug('titanic','Survived',['SibSp','Pclass']);
*/

/*---------------------------------------------------------------*/
/* aov() generates the query for a Chi-square test of inde-   */
/* pendence for two classificaition variables.                   */
/* Usage: echo chisq('titanic','Survived','Pclass');             */
delimiter //
create or replace procedure aov(tbl    text, 
                               target  text, 
                               factors array(text)) 
returns json not null
as declare
    res json not null;
    q query(j json not null) = to_query(aov_debug(tbl,target,factors));
begin
    res = scalar(q);
    return res;
end //
delimiter ;
/*---------------------------------------------------------------*/


/*---------------------------------------------------------------*/
/* Example: Survival as a function of SibSp and Pclass factors   */
/* for the titanic data.                                         */
/*
echo aov('titanic','Survived',['SibSp','Pclass']);
*/
/*---------------------------------------------------------------*/
