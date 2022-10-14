
/*------------------------------------------------------------- */
/* Simple linear regression, multiple linear regression, and    */
/* basic analysis of variance routines.                         */
/*------------------------------------------------------------- */


#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(clippy::needless_late_init)]
#![allow(clippy::needless_range_loop)]

extern crate statrs;
use statrs::distribution::{StudentsT, ContinuousCDF};
use statrs::distribution::{FisherSnedecor};
use crate::matrix::*;

const EPS: f64 = 1e-12;

#[derive(Default)]
#[derive(Debug)]
pub struct Slroutput {
    pub n   : i64,
    pub b0  : f64,
    pub b1  : f64,
    pub r2  : f64,
    pub sse : f64,
    pub pval: f64,
}

pub fn aov_add_row(data_row:Vec<f64>, xpx: &mut Vec<f64>, efflev: &Vec<i64>) {
    /* Some of the sizes used here */
    // nfac  = number of factors
    // nc    = number of coefficients (includes intercept)
    // nc1   = size of x || y row
    // nsym1 = allocation size for lower-triangular XpX with Y-border 
    let nfac   = (data_row.len()-1)/2;
    let nc = 1 + data_row[0..nfac].iter().sum::<f64>() as usize;
    let nc1  : usize = nc + 1;
    let nsym1: usize = nc1 * (nc1 + 1) / 2;
    let mut vec : Vec<f64> = vec![0.;nc1];


    /* build the dense x-row */
    vec[0] = 1.; // the intercept
    let mut has_nan = false;
    let mut pos : usize = 0; 
    for i in 0..nfac { 
        pos += efflev[i] as usize; // starting position for this effect
        let value = data_row[nfac+1+i] - 1.; // 0-based level
        if value.is_nan() {
            has_nan = true;
            break;
        }
        let level= (data_row[nfac + 1 + i] as usize) - 1; // level of this variable
        vec[pos+level] = 1.;  // position in the overall X row
       }
       vec[nc] = data_row[nfac];
       if !has_nan {
           sscp(xpx,&vec,nc1);
       }

}

pub fn aov_terminate(xpx: &mut Vec<f64>, efflev: &Vec<i64>) -> Vec<f64> {
    // nfac  = number of factors
    // nc    = number of coefficients (includes intercept)
    // nc1   = size of x || y row
    // nsym1 = allocation size for lower-triangular XpX with Y-border 
    let neff= efflev.len();
    let nfac= neff - 1; 
    let nc  = efflev.iter().sum::<i64>() as usize;
    let nc1  : usize = nc + 1;
    let nsym1: usize = nc1 * (nc1 + 1) / 2;

    let mut vec  : Vec<f64> = vec![0.; nc1];
    let mut work : Vec<f64> = vec![0.; nc1];

    let noutcol = 5;
    let mut result : Vec<f64> = vec![0.; noutcol*(neff+3)];
    let mut pos = 0;
    let mut ss_ctotal = xpx[nsym1-1];
    let mut df_ctotal = xpx[0];
    let mut df_model = 0;
    let mut row: usize = 0;

    /* For each effect compute df  SS  MS  F Pr>F */
    for i in 0..neff {
        let ss_prev = xpx[nsym1-1];
        let nlev = efflev[i] as usize;
        let df = sweep_eff(xpx,nc1,pos,nlev,&mut work);
        if df > 0 {
            // we swept some rows
            if i == 0 {
                ss_ctotal = xpx[nsym1-1];
                df_ctotal -= df as f64;
            } else {
                df_model += df;
            }
        }           
        let sse =  xpx[nsym1-1]; // SS(Error) after having swept out all previous effects 
        row = i*noutcol;
        result[row] = df as f64;
        if df > 0 {
            result[row+1] = ss_prev - sse;       // SS(effect)
            result[row+2] = (ss_prev - sse)/(df as f64);  // MS(effect)
        }
        pos += nlev;
    }
    let ss_model = ss_ctotal - xpx[nsym1-1];
    let df_error = df_ctotal - df_model as f64;
    let ms_error = xpx[nsym1-1] / df_error;
    // Compute mean squares, F statistics, p-values
    for i in 0..neff {
        row = i*noutcol;
        let fstat = result[row+2] / ms_error;
        result[row+3] = fstat; // F value

        let fdist_result = FisherSnedecor::new(result[row],df_error);
        let fdist = match fdist_result {
            Ok(fdist) => result[row+4]= 1.-fdist.cdf(fstat),
            Err(_) => result[row+4] = -1.,
        };
    }
    // Add the overall Anova results at the bottom
    row = 5*neff; // Error row
    result[row  ] = df_error;
    result[row+1] = xpx[nsym1-1];
    result[row+2] = ms_error;
    
    row = 5*(neff+1); // Model row
    result[row  ] = df_model as f64;
    result[row+1] = ss_model;
    result[row+2] = ss_model / df_model as f64;
    result[row+3] = result[row+2] / ms_error;
    let fdist_result = FisherSnedecor::new(result[row],df_error);
    let fdist = match fdist_result {
        Ok(fdist) => result[row+4] = 1. - fdist.cdf(result[row+3]),
        Err(_) => result[row+4] = -1.,
    };
    row = 5*(neff+2); // Corrected Total
    result[row  ] = df_ctotal;
    result[row+1] = ss_ctotal;

    result

}

pub fn slr_add_row(y: f64, x:f64, xpx: &mut Vec<f64>) {
    let mut xrow = vec![1.;3];  
    xrow[1] = x;
    xrow[2] = y;
    sscp(xpx, &xrow,3);
}


pub fn slr_terminate(xpx: &mut Vec<f64>, mut out: Slroutput) -> Slroutput {
    let mut out = Slroutput{..Default::default()};

    out.n = xpx[0] as i64;
    if out.n > 0 {
        let mut work : Vec<f64> = vec![0.; 3];
        let ss_total = sweep_xpx(xpx, 3, &mut work);
        out.b0  = xpx[3];
        out.b1  = xpx[4];
        out.sse = xpx[5];
        out.r2  = 1. - out.sse/ss_total;
        let dfe = (out.n as f64)- 2.;
        let mse = out.sse / dfe;
        let stder = -1. * xpx[2] * mse;
        if stder > EPS {
            let tstat = out.b1 / stder.sqrt();
            let tdist_result = StudentsT::new(0.0, 1.0, dfe);
            let tdist = match tdist_result {
                Ok(tdist) => out.pval = 2.*(1. - tdist.cdf(tstat.abs())),
                Err(_) => out.pval = -1.,
            };
        }
    }
    out
}


pub fn mlr_add_row(y: f64, data_row:Vec<f64>,xpx: &mut Vec<f64>, ) {
    let nvars  = data_row.len();     /* number of regressors   */
    let nc   : usize = nvars+1;             /* number of coefficients */
    let nc1  : usize = nc + 1;              /* size of x || y row     */
    let nsym1: usize = nc1 * (nc1 + 1) / 2; /* room for Y-border      */
    let mut vec : Vec<f64> = vec![0.;nc1];

    vec[0] = 1.; // the intercept
    vec[1..nc].clone_from_slice(&data_row[..nvars]);
    vec[nc] = y;
    let mut has_nan = false;
    for i in 0..nvars {
        if data_row[i].is_nan() {
            has_nan = true;
            break;
        }
    }
    if !has_nan {
        sscp(xpx,&vec,nc1);
    }
}

pub fn mlr_terminate(xpx: &mut Vec<f64>,nc: usize) {
    let mut work : Vec<f64> = vec![0.; nc+1];
    sweep_xpx(xpx, nc+1, &mut work);
}

pub fn mlr_terminate_long(xpx: &mut Vec<f64>,nvars: usize) -> Vec<f64> {
    let nc : usize = nvars+1;
    let nc1  : usize = nc+1;    /* size of x || y row */
    let nsym  = nc * (nc+1)/2;
    let nsym1 = nc1 * (nc1+1)/2;
    let mut work   : Vec<f64> = vec![0.0; nc1];
    
    /* How the results will be laid out in the blob */
    /* nobs | nc | ss[0..3] | df[0..3] | Fval | pval | est[0..nc] | std[0..nc] | tval[0..nc] | tpval[0..nc] */

    let nalloc : usize = 2 + 6 + 2 + nc * 4;
    let mut result : Vec<f64> = vec![0.; nalloc];
    let mut index : usize = 0;

    result[index  ] = xpx[0];
    result[index+1] = nc as f64;
    index  += 2;
    /* The sums of squares */
    let nobs = xpx[0]; 
    result[index+2] = sweep_xpx(xpx, nc1, &mut work); /* SS(Total) */
    result[index+1] = xpx[nsym1-1];                            /* SS(Error) */
    result[index  ] = result[index+2] - result[index+1];          /* SS(Model) */
    let mut ms_model = result[index  ];
    let mut ms_error = result[index+1];
    index += 3;

    /* The degrees of freedom */
    result[index  ] = nvars as f64;                          /* df(model) */
    result[index+1] = nobs - nc as f64;                      /* df(error) */
    result[index+2] = nobs - 1.;                             /* df(total, corrected) */
    ms_model /= result[index];
    ms_error /= result[index+1];

    let fdist_result = FisherSnedecor::new(result[index],result[index+1]);
    let tdist_result = StudentsT::new(0.0, 1.0, result[index+1]);
    index += 3;
    /* F statistic and p-value */       
    result[index  ] = ms_model / ms_error;
    let fdist = match fdist_result {
        Ok(fdist) => result[index+1] = 1. - fdist.cdf(result[index]),
        Err(_) => result[index+1] = -1.,
    };
    index += 2;

    /* Coefficients */
    let mut est = vec![0.;nc];
    est[0..nc].clone_from_slice(&xpx[nsym..(nsym+nc)]);

    let mut dindex : usize = 0;
    /* extract the diagonals and compute the standard errors */        
    for i in 0..nc {
        /* estimate */
        result[index+i] = est[i];
        if i > 0 { dindex += i+1; }
        let s2 = -1. * xpx[dindex] * ms_error;
        if s2 > EPS {
            /* standard error */
            result[index+nc+i] = s2.sqrt();
        }
        if result[index+nc+i] > EPS {
            /* t statistic */
            let tstat = est[i]/result[index+nc+i];
            result[index+2*nc+i] = tstat;
            /* p value */
            let tdist = match tdist_result { 
                Ok(tdist) => result[index+3*nc+i] = 2.0*(1. -  tdist.cdf(tstat.abs())),
                Err(_) => result[index+3*nc+i] = -1.,
            };
        }
    }
    result
}
