create table if not exists titanic (
  PassengerId   bigint(20) DEFAULT NULL,
  Survived      float DEFAULT NULL,
  Pclass        bigint(20) DEFAULT NULL,
  Name          text,
  Sex           text,
  Age           float DEFAULT NULL,
  SibSp         bigint(20) DEFAULT NULL,
  Parch         bigint(20) DEFAULT NULL,
  Ticket        text,
  Fare          float DEFAULT NULL,
  Cabin         text,
  Embarked      text
);

delete from titanic;

load data local infile './titanic_train.csv'
  fields terminated by ','
  optionally enclosed by '"'
  TRAILING NULLCOLS
  NULL defined by ''
  IGNORE 1 lines
  into table titanic;

load data local infile './titanic_test.csv'
  fields terminated by ','
  optionally enclosed by '"'
  TRAILING NULLCOLS
  NULL defined by ''
  IGNORE 1 lines
  replace into table titanic
  (PassengerId, Pclass, Name, Sex, age, SibSp, Parch, Ticket, Fare, Cabin, Embarked);

