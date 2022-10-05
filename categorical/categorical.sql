
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
echo chisq('xclass','A','B');


delimiter //
create or replace procedure chisq(tbl     varchar(100), 
                                  rowvar  varchar(100), 
                                  colvar  varchar(100))
returns query(NumRows int, NumCols int, RowNum int, ColNum int, Cell double, RowTotal double, ColTotal double)
as declare
    qstr varchar(1024);
    q query(NumRows int, NumCols int, RowNum int, ColNum int, Cell double, RowTotal double, ColTotal double);
begin
    qstr = 'select * from (
select';
    qstr = concat(qstr,'(select count(distinct ', rowvar, ') from ', tbl, ') as NumRows,
                        (select count(distinct ', colvar, ') from ', tbl, ') as NumCols,
    t2.RowNum, t3.ColNum, t1.Cell, t2.RowTotal, t3.ColTotal
from (
    select ');
    qstr = concat(qstr, rowvar, ' as "Row", ', colvar, ' as "Col", count(*) as "Cell" from ', tbl);
    qstr = concat(qstr,' group by ', rowvar, ',', colvar, ') as t1');

    qstr = concat(qstr, '
inner join (
    select ', rowvar, ' as "Row",
            count(*) as RowTotal,
            Row_Number() over(order by ', rowvar, ') as RowNum');
    qstr = concat(qstr,' from ', tbl, ' group by ', rowvar, ') as t2 on t2.Row = t1.Row');
  
    qstr = concat(qstr,'
inner join (
    select ', colvar, ' as "Col",
        count(*) as ColTotal,
        Row_Number() over(order by ', colvar,') as ColNum');
    qstr = concat(qstr,' from ', tbl, ' group by ', colvar, ') as t3 on t1.Col = t3.Col');
    qstr = concat(qstr, ' order by t1.Row, t1.Col);' );

    q = to_query(qstr);
    return q;
end //
delimiter ;



delimiter //
create or replace procedure chisq_debug(tbl     varchar(100) not null, 
                                        rowvar  varchar(100) not null, 
                                        colvar  varchar(100) not null)
returns text
as declare
  qstr varchar(1024);
begin
    qstr = 'select * from (
select';
    qstr = concat(qstr,'(select count(distinct ', rowvar, ') from ', tbl, ') as NumRows,
                        (select count(distinct ', colvar, ') from ', tbl, ') as NumCols,
    t2.RowNum, t3.ColNum, t1.Cell, t2.RowTotal, t3.ColTotal
from (
    select ');
    qstr = concat(qstr, rowvar, ' as "Row", ', colvar, ' as "Col", count(*) as "Cell" from ', tbl);
    qstr = concat(qstr,' group by ', rowvar, ',', colvar, ') as t1');

    qstr = concat(qstr, '
inner join (
    select ', rowvar, ' as "Row",
            count(*) as RowTotal,
            Row_Number() over(order by ', rowvar, ') as RowNum');
    qstr = concat(qstr,' from ', tbl, ' group by ', rowvar, ') as t2 on t2.Row = t1.Row');
  
    qstr = concat(qstr,'
inner join (
    select ', colvar, ' as "Col",
        count(*) as ColTotal,
        Row_Number() over(order by ', colvar, ') as ColNum');
    qstr = concat(qstr,' from ', tbl, ' group by ', colvar, ') as t3 on t1.Col = t3.Col');
    qstr = concat(qstr, ' order by t1.Row, t1.Col);' );

    return qstr;
end //
delimiter ;

call chisq('titanic','Survived','Pclass');
echo chisq('titanic','Survived','Pclass');

echo chisq_debug('titanic','Survived','Pclass');


echo chisq_debug('titanic','Survived','Pclass');

    

with
  foo as (
    select * from titanic
    where survived is not null and Pclass is not null
  ),
  cell_counts as (
    select
      Survived as 'Survived',
      Pclass   as 'Class',
      count(*) as 'Cell'
    from foo
    group by Survived, Pclass
  ),
  v1_counts as (
    select
      Survived,
      count(*) as RowTotal
    from foo
    group by Survived
  ),
  v2_counts as (
    select
      Pclass,
      count(*) as ColTotal
    from foo
    group by Pclass
  )
select
  (select count(distinct Survived) from foo) as NumRows,
  (select count(distinct Pclass  ) from foo) as NumCols,
  t1.Survived, t1.Class,
  t1.Cell,
  t2.RowTotal, t3.ColTotal
from cell_counts as t1
inner join v1_counts as t2 on t2.Survived = t1.Survived
inner join v2_counts as t3 on t1.class    = t3.Pclass

order by 3, 4
option(materialize_ctes="off");


