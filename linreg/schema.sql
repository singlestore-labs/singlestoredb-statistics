
/*-----------------------------------------------------------------------------*/
/* 
Multiple linear regression example from the PROC REG documentation of SAS:
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

/*--------------------------------------------------------------*/
/* Examples                                                     */
/* Long output, full model                                      */
select mlr(oxygen,vec_pack_f64([Age, Weight, RunTime, RestPulse, RunPulse, MaxPulse])) from fitness;

/* Long output, small model                                     */
select mlrl(oxygen,vec_pack_f64([Age, Weight])) from fitness;

/* Short output: just the regression coefficients               */
select mlr(oxygen,vec_pack_f64([Age, Weight])) from fitness;

/*--------------------------------------------------------------*/
/* Simple linear regression using the slr() and mlr() functions */
select slr(oxygen,Age) from fitness;
/* just the regression coefficients                             */
select mlr(oxygen,vec_pack_f64([Age])) from fitness; 
/* detailed (long) output                                       */
select mlrl(oxygen,vec_pack_f64([Age])) from fitness; 

/*--------------------------------------------------------------*/
/* Simple linear regression with group-by                       */
select slr(oxygen,Age) from fitness group by agegroup;
/*--------------------------------------------------------------*/

/*--------------------------------------------------------------*/
/* Multiple linear regression with group by processing          */
select AgeGroup, mlr(oxygen,vec_pack_f64([Weight, RunPulse])) from fitness group by AgeGroup;
/*--------------------------------------------------------------*/


/*--------------------------------------------------------------*/
/* Simple linear regression using the slr() and mlr() functions */
/* with the iris data.                                          */
/* select slr(sepal_width,sepal_length) from iris;              */
/* select mlr(sepal_width,vec_pack_f64([sepal_length])) from iris; */
/*--------------------------------------------------------------*/


/*--------------------------------------------------------------*/
/* Create a table for the CalCOFI bottle data                   */
/* https://www.kaggle.com/datasets/sohier/calcofi               */

create table if not exists bottle (
  Cst_Cnt   int default null,
  Btl_Cnt   int default null,
  Sta_ID        text  default null,
  Depth_ID      text  default null,
  Depthm        int   default null,
  T_degC        float default null,
  Salnty        float default null,
  O2ml_L        float default null,
  STheta        float default null,
  O2Sat         float default null,
  Oxy           float default null,
  BtlNum        float default null,
  RecInd        int   default null,
  T_prec        int   default null,
  T_qual        int   default null,
  S_prec        int   default null,
  S_qual        int   default null,
  P_qual        int   default null,
  O_qual        int   default null,
  SThtaq        int   default null,
  O2Satq        int   default null,
  ChlorA        float default null,
  Chlqua        int   default null,
  Phaeop        float default null,
  Phaqua        int   default null,
  PO4           float default null,
  PO4q          int   default null,
  SiO3          float default null,
  SiO3q         int   default null,
  NO2           float default null,
  NO2q          int   default null,
  NO3           float default null,
  NO3q          int   default null,
  NH3           float default null,
  NH3q          int   default null,
  C14As1        float default null,
  C14A1p        float default null,
  C14Aq         int   default null,
  C14As2        float default null,
  C14A2p        float default null,
  C14A2q        int   default null,
  DarkAs        float default null,
  DarkAp        float default null,
  DarkAq        int   default null,
  MeanAs        float default null,
  MeanAp        float default null,
  MeanAq        int   default null,
  IncTim        text  default null,
  LightP        float default null,
  R_Depth       float default null,
  R_Temp        float default null,
  R_PoTemp      float default null,
  R_Salinity    float default null,
  R_Sigma       float default null,
  R_SVA         float default null,
  R_Dynht       float default null,
  R_O2          float default null,
  R_O2Sat       float default null,
  R_SIO3        float default null,
  R_PO4         float default null,
  R_NO3         float default null,
  R_NO2         float default null,
  R_NH4         float default null,
  R_CHLA        float default null,
  R_PHAEO       float default null,
  R_Pres        float default null,
  R_Samp        float default null,
  DIC1          float default null,
  DIC2          float default null,
  TA1           float default null,
  TA2           float default null,
  pH2           float default null,
  pH1           float default null,
  DICQuality    text default null
);
delete from bottle;
  
load data local infile './data/bottle.csv'
  fields terminated by ','
  optionally enclosed by '"'
  TRAILING NULLCOLS
  NULL defined by ''
  IGNORE 1 lines
  into table bottle;

/*--------------------------------------------------------------*/
/* Simple linear regression with mlr() and slr() functions to   */
/* model Salinity as a function of T_degc                       */
select slr(Salnty,T_degc) from bottle where T_degc is not null and Salnty is not null;
select mlr(Salnty,vec_pack_f64([T_degc])) from bottle where T_degc is not null and Salnty is not null;
/*--------------------------------------------------------------*/

