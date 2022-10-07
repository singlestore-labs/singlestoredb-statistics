
/*-----------------------------------------------------------------------------*/
/* 
Example from the PROC REG documentation of SAS:
https://documentation.sas.com/doc/en/pgmsascdc/9.4_3.3/statug/statug_reg_examples02.htm

Oxygen = f(Age, Weight, RunTime, RestPulse, RunPulse, MaxPulse)

   All Variables Entered: R-Square = 0.8487 and C(p) = 7.0000
 
                         Analysis of Variance
Source	        DF	Sum of Squares	Mean  Square	F Value	Pr > F
Model	        6	    722.54361	    120.42393	22.43	<.0001
Error	        24	    128.83794	      5.36825	 	 
Corrected Total	30	    851.38154	 	 	 

                        Parameter Estimates
Variable	Parameter Estimate	Standard Error	Type II SS	F Value	Pr > F
Intercept	        102.93448	12.40326	    369.72831	68.87	<.0001
Age	                 -0.22697	0.09984	         27.74577	 5.17	0.0322
Weight             	 -0.07418	0.05459	          9.91059	 1.85	0.1869
RunTime	             -2.62865	0.38456	        250.82210	46.72	<.0001
RestPulse	         -0.02153	0.06605	          0.57051	 0.11	0.7473
RunPulse	         -0.36963	0.11985	         51.05806	 9.51	0.0051
MaxPulse	          0.30322	0.13650	         26.49142	 4.93	0.0360
*/
/*-----------------------------------------------------------------------------*/

create table if not exists 
    fitness(Age       double, 
            Weight    double, 
            Oxygen    double, 
            RunTime   double, 
            RestPulse double,
            RunPulse  double,
            MaxPulse  double);
delete from fitness;
load data local infile './data/fitness.csv'
     fields terminated by ','
     NULL defined by ''
     IGNORE 1 lines
     replace into table test.fitness;

alter table fitness
add column agegroup as (case 
    when age < 44 then "< 44" 
    when age >= 44 and age < 50 then "44-50" 
    else "> 50" end) persisted text;

/* Examples */
/* Long output, full model */
select mlr(oxygen,vec_pack_f64([Age, Weight, RunTime, RestPulse, RunPulse, MaxPulse])) from fitness;

/* Long output, small model */
select mlrl(oxygen,vec_pack_f64([Age, Weight])) from fitness;


/* Short output: just the regression coefficients */
select mlr(oxygen,vec_pack_f64([Age, Weight])) from fitness;
/* Simple linear regression */
/* select mlr(oxygen,vec_pack_f64([Age])) from fitness; */
/* select mlrl(oxygen,vec_pack_f64([Age])) from fitness; */

/* select mlr(oxygen,vec_pack_f64([Age, Weight])) from fitness; */

/* with group by processing */
select AgeGroup, mlr(oxygen,vec_pack_f64([Weight, RunPulse])) from fitness group by AgeGroup;
