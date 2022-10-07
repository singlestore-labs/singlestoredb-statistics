

#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(clippy::needless_late_init)]
#![allow(clippy::needless_range_loop)]

wit_bindgen_rust::export!("linreg.wit");

const EPS: f64 = 1e-12;

extern crate statrs;
use statrs::distribution::{StudentsT, ContinuousCDF};
use statrs::distribution::{FisherSnedecor};
use byte_slice_cast::*;

use crate::linreg::State;
use crate::matrix::*;
pub mod matrix;


struct Linreg;

impl linreg::Linreg for Linreg {
    fn mlr_init() -> State {
        State {
            nvars: 0,
            xpx  : Vec::new(),
        }
    }

    fn mlr_iter(in_state:State, y: f64, vars:Vec<u8>) -> State {
        let data_row = Self::vec_unpack_f64(vars);
        let nvars  = data_row.len();     /* number of regressors   */
        let nc   : usize = nvars+1;             /* number of coefficients */
        let nc1  : usize = nc + 1;              /* size of x || y row     */
        let nsym1: usize = nc1 * (nc1 + 1) / 2; /* room for Y-border      */
        let mut st: State;
        let mut vec : Vec<f64> = vec![0.;nc1];

        if in_state.xpx.is_empty() {
            st = State {
                nvars : nvars as i64,
                xpx   : vec![0.; nsym1],
            };
        } else {
            st = State {
                nvars : nvars as i64,
                xpx   : in_state.xpx.to_vec(),
            };
        }
  
        vec[0] = 1.; // the intercept
        vec[1..nc].clone_from_slice(&data_row[..nvars]);
        vec[nc] = y;

        /*---TODO: check for null values in data_row ---*/
        sscp(&mut st.xpx,&vec,nc1);
        st
    }

    fn mlr_merge(mut st1:State,mut st2:State,) -> State {
        if st1.xpx.is_empty() {
            st2
        } else if st2.xpx.is_empty() {
            st1
        } else {
            vector_add(&st1.xpx,&st2.xpx);
            st1
        }
    }

    /* return only the regression coefficients */
    fn mlr_term(mut st: State) -> Vec<u8> {
        let nvars = st.nvars as usize;
        let nc   : usize = nvars+1;
        let nc1  : usize = nc+1;    /* size of x || y row */
        let nsym   = nc * (nc+1)/2;
        let mut work   : Vec<f64> = vec![0.; nc1];
        let mut result : Vec<f64> = vec![0.; nc ];
        sweep_xpx(&mut st.xpx, nc1, &mut work);    /* SS(Total) */
        result[0..nc].clone_from_slice(&st.xpx[nsym..(nsym+nc)]);

        Self::vec_pack_f64(result)
    } 

    /*---Return vectors of floats for the estimates, standard errors, t-values */
    /*---and p-values */
    fn mlr_terml(mut st: State) -> Vec<u8> {
        let nvars = st.nvars as usize;
        let nc   : usize = nvars+1;
        let nc1  : usize = nc+1;    /* size of x || y row */
        let nsym  = nc * (nc+1)/2;
        let nsym1 = nc1 * (nc1+1)/2;
        let mut work   : Vec<f64> = vec![0.0; nc1];
        
        /* How the results will be laid out in the blob */
        /* nobs | nc | ss[0..3] | df[0..3] | Fval | pval | est[0..nc] | std[0..nc] | tval[0..nc] | tpval[0..nc] */

        let nalloc : usize = 2 + 6 + 2 + nc * 4;
        let mut result : Vec<f64> = vec![0.; nalloc];
        let mut index : usize = 0;
 
        result[index  ] = st.xpx[0];
        result[index+1] = nc as f64;
        index  += 2;
        /* The sums of squares */
        let nobs = st.xpx[0]; 
        result[index+2] = sweep_xpx(&mut st.xpx, nc1, &mut work); /* SS(Total) */
        result[index+1] = st.xpx[nsym1-1];                            /* SS(Error) */
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

        let fdist = FisherSnedecor::new(result[index],result[index+1]).unwrap();
        let tdist = StudentsT::new(0.0, 1.0, result[index+1]).unwrap();
        index += 3;
        /* F statistic and p-value */       
        result[index  ] = ms_model / ms_error;
        result[index+1] = 1. - fdist.cdf(result[index]);
        index += 2;

        /* Coefficients */
        let mut est = vec![0.;nc];
        est[0..nc].clone_from_slice(&st.xpx[nsym..(nsym+nc)]);

        let mut dindex : usize = 0;
        /* extract the diagonals and compute the standard errors */        
        for i in 0..nc {
            /* estimate */
            result[index+i] = est[i];
            if i > 0 { dindex += i+1; }
            let s2 = -1. * st.xpx[dindex] * ms_error;
            if s2 > crate::EPS {
                /* standard error */
                result[index+nc+i] = s2.sqrt();
            }
            if result[index+nc+i] > crate::EPS {
                /* t statistic */
                let tstat = est[i]/result[index+nc+i];
                result[index+2*nc+i] = tstat;
                /* p value */
                result[index+3*nc+i] = 2.0*(1. -  tdist.cdf(tstat.abs())); 
                // result[index+3*nc+i] = 2.0 * (1. -  tdist.cdf(1.96)); 
            }
        }
        Self::vec_pack_f64(result)
    }

    fn vec_pack_f64(v:Vec<f64>,) -> Vec<u8> {
        v.as_byte_slice().to_vec()
    }
    fn vec_unpack_f64(v:Vec<u8>,) -> Vec<f64> {
        v.as_slice_of::<f64>().unwrap().to_vec()
    }

   
}
