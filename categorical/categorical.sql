
/*---------------------------------------------------------------*/
/* Example of a pre-grouped table: the cell count are already    */
/* computed in column Nij                                        */
/* Example is from Section 8.13 in Ott, L.R. "An Introduction to"*/
/* Statistical Methods and Data Analysis", Duxbury Press, 1993   */
/* Usage: echo chisq_grouped('employee_sat','EmpClass','Opinion','Nij');*/
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
/*---------------------------------------------------------------*/


/*---------------------------------------------------------------*/
/* Example of a small table with ungroupd data. Call chisq() for */
/* a test of independence                                        */
/* Usage: echo chisq('xclass','A','B');                          */
create table if not exists test.xclass (
   A int, B int, Y double
);
delete from xclass;
insert into test.xclass(A,B,Y) values
(1, 1, 0.1),
(1, 1, 0.2),
(1, 2, 0.3),
(1, 2, 0.4),
(2, 2, 0.5),
(2, 2, 0.6),
(1, 3, 0.7),
(1, 3, 0.8),
(2, 3, 0.9),
(3, 1, 1.0),
(3, 1, 1.1)
;
/*---------------------------------------------------------------*/

/*---------------------------------------------------------------*/
/* Helper UDF to translate the return  from chisq_term to JSON   */
delimiter //
create or replace function chisq_term_json (
    state record (
        wrk  array(double not null) not null
    ) not null)
    returns json not null as
begin
    return to_json(`chisq_term`(state));
end //
delimiter ;
/*---------------------------------------------------------------*/

/*---------------------------------------------------------------*/
/* The aggregate function that performs a Chi-square test of     */
/* independence for two classification variables.                */
/* This aggregate function is called from the chisq( ) and       */
/* chisq_grouped( ) procedures.                                  */
create or replace aggregate chisq_agg(bigint(20) not null,   /* NumRows */
                                      bigint(20) not null,   /* NumCols */
                                      double     not null,   /* Cell    */
                                      double     not null,   /* RowTotal*/
                                      double     not null    /* ColTotal*/)
    returns json not null
    with state record (
        wrk  array(double not null) not null
    ) not null
    initialize with `chisq_init`
    iterate    with `chisq_iter`
    merge      with `chisq_merge`
    terminate  with  chisq_term_json;
/*---------------------------------------------------------------*/


/*---------------------------------------------------------------*/
/* chisq_debug() generates the query for a Chi-square test of    */
/* independence for two classificaition variables.               */
/* Usage: echo chisq_debug('titanic','Survived','Pclass');       */
delimiter //
create or replace procedure chisq_debug(tbl    text, 
                                        rowvar text, 
                                        colvar text)
returns text
as declare
    qstr text;
begin
    qstr = 'select chisq_agg(NumRows, NumCols, Cell, RowTotal, ColTotal) from (
select';
    qstr = concat(qstr,'(select count(distinct ', rowvar, ') from ', tbl, ') as NumRows,
                        (select count(distinct ', colvar, ') from ', tbl, ') as NumCols,
    t1.Cell, t2.RowTotal, t3.ColTotal
from (
    select ');
    qstr = concat(qstr, rowvar, ' as "Row", ', colvar, ' as "Col", count(*) as "Cell" from ', tbl);
    qstr = concat(qstr,' where ', rowvar, ' is not null and ', colvar, ' is not null group by ', rowvar, ',', colvar, ') as t1');

    qstr = concat(qstr, '
inner join (
    select ', rowvar, ' as "Row",
            count(*) as RowTotal');
    qstr = concat(qstr,' from ', tbl, ' where ', rowvar, ' is not null and ',colvar, ' is not null group by ', rowvar, ') as t2 on t2.Row = t1.Row');
  
    qstr = concat(qstr,'
inner join (
    select ', colvar, ' as "Col",
        count(*) as ColTotal');
    qstr = concat(qstr,' from ', tbl, ' where ', rowvar, ' is not null and ', colvar, ' is not null group by ', colvar, ') as t3 on t1.Col = t3.Col');
    qstr = concat(qstr, ' order by t1.Row, t1.Col);' );
    return qstr;
end //
delimiter ;
/*---------------------------------------------------------------*/


/*---------------------------------------------------------------*/
/* chisq_() generates the query for a Chi-square test of inde-   */
/* pendence for two classificaition variables.                   */
/* Usage: echo chisq('titanic','Survived','Pclass');             */
delimiter //
create or replace procedure chisq(tbl    text, 
                                  rowvar text, 
                                  colvar text)
returns json not null
as declare
    res json not null;
    q query(j json not null) = to_query(chisq_debug(tbl,rowvar,colvar));
begin
    res = scalar(q);
    return res;
end //
delimiter ;
/*---------------------------------------------------------------*/



/*---------------------------------------------------------------*/
/* chisq_grouped_debug() generates the query for a Chi-square    */
/* test of independence for two classificaition variables when   */
/* the input table contains already grouped data.                */
/* Usage: echo chisq_grouped_debug('employee_sat','EmpClass','Opinion',"Nij"); */
delimiter //
create or replace procedure chisq_grouped_debug(tbl    text, 
                                                rowvar text, 
                                                colvar text,
                                                grpvar text)
returns text
as declare
    qstr text;
begin
    qstr = 'select chisq_agg(NumRows, NumCols, Cell, RowTotal, ColTotal) from (';
    qstr = concat(qstr,' with ');
    qstr = concat(qstr,' foo as ( select * from ', tbl, ' where ', rowvar, ' is not null and ', colvar, ' is not null),');
    qstr = concat(qstr,' cell_counts as ( select ', rowvar, ' as "Row",', colvar, ' as "Col", sum(',grpvar,') as "Cell" ');
    qstr = concat(qstr,' from foo group by ', rowvar, ', ', colvar, '),');

    qstr = concat(qstr,' v1_counts as ( select ', rowvar, ' as "Row", sum(',grpvar,') as RowTotal from foo group by ', rowvar, '),');
    qstr = concat(qstr,' v2_counts as ( select ', colvar, ' as "Col", sum(',grpvar,') as ColTotal from foo group by ', colvar, ') ');

    qstr = concat(qstr, 'select (select count(distinct ', rowvar, ') from foo) as NumRows,');
    qstr = concat(qstr, '       (select count(distinct ', colvar, ') from foo) as NumCols,');
    qstr = concat(qstr, ' t1.Cell, t2.RowTotal, t3.ColTotal');
    qstr = concat(qstr, ' from cell_counts as t1 ');
    qstr = concat(qstr, ' inner join v1_counts as t2 on t2.Row = t1.Row');
    qstr = concat(qstr, ' inner join v2_counts as t3 on t1.Col = t3.Col');
    qstr = concat(qstr, ' order by 3)');
    qstr = concat(qstr, ' option(materialize_ctes="off");');
    return qstr;
end //
delimiter ;
/*---------------------------------------------------------------*/

/*---------------------------------------------------------------*/
/* chisq_grouped() computes the Chi-square test of independence  */
/* for two classificaition variables when the input table        */
/* contains already grouped data.                                */
/* Usage: echo chisq_grouped('employee_sat','EmpClass','Opinion',"Nij"); */
delimiter //
create or replace procedure chisq_grouped(tbl    text, 
                                          rowvar text, 
                                          colvar text,
                                          grpvar text)
returns json not null
as declare
    res json not null;
    q query(j json not null) = to_query(chisq_grouped_debug(tbl,rowvar,colvar,grpvar));
begin
    res = scalar(q);
    return res;
end //
delimiter ;
/*---------------------------------------------------------------*/
