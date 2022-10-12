

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
use crate::linreg::Slrstate;
use crate::linreg::Aovstate;
use crate::linreg::Slrsummary;
use crate::matrix::*;
pub mod matrix;

struct Linreg;

impl linreg::Linreg for Linreg {

    /*----------------------------------------------------------*/
    /* AOV                                                      */
    fn aov_init() -> Aovstate {
        Aovstate { efflev: Vec::new(), xpx  : Vec::new(),}
    }
 
    /* aov_agg(vec_pack_f64([ NumX0, NumX1, Target, X0Level, X1Level])) */
    fn aov_iter_debug(mut in_state:Aovstate, xrow: Vec<f64> ) -> Aovstate {
        let data_packed = Self::vec_pack_f64(xrow);
        Self::aov_iter(in_state,data_packed)
    }

    fn aov_iter(in_state:Aovstate, xpacked:Vec<u8>) -> Aovstate {
        let data_row = Self::vec_unpack_f64(xpacked);
        /* Some of the sizes used here */
        // nfac  = number of factors
        // nc    = number of coefficients (includes intercept)
        // nc1   = size of x || y row
        // nsym1 = allocation size for lower-triangular XpX with Y-border 
        let nfac   = (data_row.len()-1)/2;
        let nc = 1 + data_row[0..nfac].iter().sum::<f64>() as usize;
        let nc1  : usize = nc + 1;
        let nsym1: usize = nc1 * (nc1 + 1) / 2;
        let mut st: Aovstate;
        let mut vec : Vec<f64> = vec![0.;nc1];

        if in_state.xpx.is_empty() {
            st = Aovstate { 
                efflev : vec![0 ; nfac+1],
                xpx    : vec![0.; nsym1],
            };
            st.efflev[0] = 1; // intercept
            for i in 0..nfac {
                st.efflev[i+1] = data_row[i] as i64;
            }
        } else {
            st = Aovstate {
                efflev : in_state.efflev.to_vec(), 
                xpx    : in_state.xpx.to_vec(),
            };
        }
        /* build the dense x-row */
        vec[0] = 1.; // the intercept
        let mut has_nan = false;
        let mut pos : usize = 0; 
        for i in 0..nfac { 
            pos += st.efflev[i] as usize; // starting position for this effect
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
            sscp(&mut st.xpx,&vec,nc1);
        }
        st
    }


    fn aov_merge(mut st1:Aovstate,mut st2:Aovstate) -> Aovstate {
        if st1.xpx.is_empty() {
            st2
        } else if st2.xpx.is_empty() {
            st1
        } else {
            vector_add(&mut st1.xpx,&st2.xpx);
            st1
        }
    }

    fn aov_term_debug(mut st:Aovstate) -> Vec<f64> {
        // nfac  = number of factors
        // nc    = number of coefficients (includes intercept)
        // nc1   = size of x || y row
        // nsym1 = allocation size for lower-triangular XpX with Y-border 
        let neff= st.efflev.len();
        let nfac= neff - 1; 
        let nc  = st.efflev.iter().sum::<i64>() as usize;
        let nc1  : usize = nc + 1;
        let nsym1: usize = nc1 * (nc1 + 1) / 2;

        let mut vec  : Vec<f64> = vec![0.; nc1];
        let mut work : Vec<f64> = vec![0.; nc1];

        let noutcol = 5;
        let mut result : Vec<f64> = vec![0.; noutcol*(neff+3)];
        let mut pos = 0;
        let mut ss_ctotal = st.xpx[nsym1-1];
        let mut df_ctotal = st.xpx[0];
        let mut df_model = 0;
        let mut row: usize = 0;

        /* For each effect compute df  SS  MS  F Pr>F */
        for i in 0..neff {
            let ss_prev = st.xpx[nsym1-1];
            let nlev = st.efflev[i] as usize;
            let df = sweep_eff(&mut st.xpx,nc1,pos,nlev,&mut work);
            if df > 0 {
                // we swept some rows
                if i == 0 {
                    ss_ctotal = st.xpx[nsym1-1];
                    df_ctotal -= df as f64;
                } else {
                    df_model += df;
                }
            }           
            let sse =  st.xpx[nsym1-1]; // SS(Error) after having swept out all previous effects 
            row = i*noutcol;
            result[row] = df as f64;
            if df > 0 {
                result[row+1] = ss_prev - sse;       // SS(effect)
                result[row+2] = (ss_prev - sse)/(df as f64);  // MS(effect)
            }
            pos += nlev;
        }
        let ss_model = ss_ctotal - st.xpx[nsym1-1];
        let df_error = df_ctotal - df_model as f64;
        let ms_error = st.xpx[nsym1-1] / df_error;
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
        result[row+1] = st.xpx[nsym1-1];
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

    fn aov_term(mut st:Aovstate) -> Vec<u8> {
        Self::vec_pack_f64(Self::aov_term_debug(st))
    }
    /* eo AOV                                                   */
    /*----------------------------------------------------------*/


    /*----------------------------------------------------------*/
    /* SLR Begin                                                */
    fn slr_init() -> Slrstate {
        Slrstate {xpx : vec![0.;6]}
    }

    fn slr_iter(mut st:Slrstate, y:f64, x:f64) -> Slrstate {
        let mut xrow = vec![1.;3];
        xrow[1] = x;
        xrow[2] = y;
        sscp(&mut st.xpx,&xrow,3);
        st
    }

    fn slr_merge(mut st1:Slrstate, mut st2:Slrstate) -> Slrstate {
        vector_add(&mut st1.xpx,&st2.xpx);
        st1
    }

    fn slr_term(mut st:Slrstate) -> Slrsummary {
        let mut out = Slrsummary{b0:0., b1:0., n:0., r2: 0., sse:0., pvalue:0.,};
        let mut work : Vec<f64> = vec![0.; 3];

        out.n = st.xpx[0];
        if out.n > 0. {
            let ss_total = sweep_xpx(&mut st.xpx, 3, &mut work);
            out.b0  = st.xpx[3];
            out.b1  = st.xpx[4];
            out.sse = st.xpx[5];
            out.r2  = 1. - out.sse/ss_total;
            let dfe = out.n-2.;
            let mse = out.sse / dfe;
            let stder = -1. * st.xpx[2] * mse;
            if stder > EPS {
                let tstat = out.b1 / stder.sqrt();
                let tdist_result = StudentsT::new(0.0, 1.0, dfe);
                let tdist = match tdist_result {
                    Ok(tdist) => out.pvalue = 2.*(1. - tdist.cdf(tstat.abs())),
                    Err(_) => out.pvalue = -1.,
                };
            }
        }
        out
   }
    /* eo SLR                                                   */
    /*----------------------------------------------------------*/

    /*----------------------------------------------------------*/
    /* MLR Begin                                                */
    fn mlr_init() -> State {
        State {nvars: 0, xpx  : Vec::new(), }
    }

    fn mlr_iter_debug(in_state:State, y:f64, xrow:Vec<f64>) -> State {
        let data_packed = Self::vec_pack_f64(xrow);
        Self::mlr_iter(in_state,y, data_packed)
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
        let mut has_nan = false;
        for i in 0..nvars {
            if data_row[i].is_nan() {
                has_nan = true;
                break;
            }
        }
        if !has_nan {
            sscp(&mut st.xpx,&vec,nc1);
        }
        st
    }

    fn mlr_merge(mut st1:State,mut st2:State,) -> State {
        if st1.xpx.is_empty() {
            st2
        } else if st2.xpx.is_empty() {
            st1
        } else {
            // let mut st = State {
            //     nvars  : st1.nvars,  
            //     xpx    : st1.xpx.to_vec(),
            // };
            // for i in 0..st1.xpx.len() {
            //     st1.xpx[i] += st2.xpx[i];
            // }
            vector_add(&mut st1.xpx,&st2.xpx);
            st1
        }
    }

    fn mlr_term_debug(mut st:State) -> Vec<f64> {   
        let nvars = st.nvars as usize;
        let nc   : usize = nvars+1;  // accounts for intercept
        let nc1  : usize = nc+1;     // size of x || y row 
        let nsym   = nc * (nc+1)/2;
        let mut work   : Vec<f64> = vec![0.; nc1];
        let mut result : Vec<f64> = vec![0.; nc ];
        sweep_xpx(&mut st.xpx, nc1, &mut work);    /* SS(Total) */
        result[0..nc].clone_from_slice(&st.xpx[nsym..(nsym+nc)]);
        result
    }

    /* return only the regression coefficients */
    fn mlr_term(mut st: State) -> Vec<u8> {
        Self::vec_pack_f64(Self::mlr_term_debug(st))
    } 

    fn mlr_terml_debug(mut st: State) -> Vec<f64> {
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
                let tdist = match tdist_result { 
                    Ok(tdist) => result[index+3*nc+i] = 2.0*(1. -  tdist.cdf(tstat.abs())),
                    Err(_) => result[index+3*nc+i] = -1.,
                };
                // result[index+3*nc+i] = 2.0 * (1. -  tdist.cdf(1.96)); 
            }
        }
        result
    }

    /*---Return vectors of floats for the estimates, standard errors, t-values */
    /*---and p-values */
    fn mlr_terml(mut st: State) -> Vec<u8> {
        Self::vec_pack_f64(Self::mlr_terml_debug(st))
    }
    /* eo MLR                                                   */
    /*----------------------------------------------------------*/
 
    /*----------------------------------------------------------*/
    /* Utilities                                                */
    fn vec_pack_f64(v:Vec<f64>,) -> Vec<u8> {
        v.as_byte_slice().to_vec()
    }
    fn vec_unpack_f64(v:Vec<u8>,) -> Vec<f64> {
        v.as_slice_of::<f64>().unwrap().to_vec()
    }
    /* eo Utilities                                             */
    /*----------------------------------------------------------*/
 
   
}


mod test {

    mod regression {
        pub fn slr_basic() {}
        pub fn mlr_basic() {}
        pub fn mlr_sing () {}
        pub fn mlr_merge() {}
    }

    #[test]
    fn slr_basic() {
        let state0 = <super::Linreg as super::linreg::Linreg>::slr_init();
        assert!(!state0.xpx.is_empty());
        let state1 = <super::Linreg as super::linreg::Linreg>::slr_iter(state0, 6.0, 1.0);
        assert!(state1.xpx[1] == 1.0);
        let state2 = <super::Linreg as super::linreg::Linreg>::slr_iter(state1, 10.0, 2.0);
        assert!(state2.xpx[1] == 3.0);
        let state3 = <super::Linreg as super::linreg::Linreg>::slr_iter(state2, 2.0, 3.0);
        assert!(state3.xpx[5] == 140.0);
        
        let result = <super::Linreg as super::linreg::Linreg>::slr_term(state3);
        println!("SLR results {:?}", result);                 
    }

    #[test]
    fn mlr_basic() {
        // A basic test for the multiple linear regression module with four observations, two regressors
        // Displaying both short and long results
        let state0 = <super::Linreg as super::linreg::Linreg>::mlr_init();
        assert!(state0.nvars == 0);

        // let vars = <super::Linreg as super::linreg::Linreg>::vec_pack_f64(vec![2.0, 4.0, 8.0]);
        let state1 = <super::Linreg as super::linreg::Linreg>::mlr_iter_debug(state0, 44.609, vec![44., 62.]);
        assert!(state1.nvars == 2);
        
        let state2 = <super::Linreg as super::linreg::Linreg>::mlr_iter_debug(state1, 45.313, vec![40., 62.]);
        assert!(state2.nvars == 2);

        let state3 = <super::Linreg as super::linreg::Linreg>::mlr_iter_debug(state2, 54.297, vec![44., 45.]);
 
        let state4 = <super::Linreg as super::linreg::Linreg>::mlr_iter_debug(state3, 59.571, vec![42., 40.]);
        println!("XpX matrix after four obs: \n{:?}",state4);
 
        let result = <super::Linreg as super::linreg::Linreg>::mlr_terml_debug(state4);
        println!("Long results: {:?}", result);
    }

    #[test]
    fn mlr_sing() {
        // Two regressors, three data points. This is a perfect fit regression with SS(Error) = 0.
        // Short and long results
        let state0 = <super::Linreg as super::linreg::Linreg>::mlr_init();
        assert!(state0.nvars == 0);

        let state1 = <super::Linreg as super::linreg::Linreg>::mlr_iter_debug(state0, 44.609, vec![44., 62.]);
        assert!(state1.nvars == 2);
        
        let state2 = <super::Linreg as super::linreg::Linreg>::mlr_iter_debug(state1, 45.313, vec![40., 62.]);
        assert!(state2.nvars == 2);
        
        let state3 = <super::Linreg as super::linreg::Linreg>::mlr_iter_debug(state2, 54.297, vec![44., 45.]);
        println!("XpX matrix after three obs: \n{:?}",state3);
 
        let state3a = state3.clone();
        let result1 = <super::Linreg as super::linreg::Linreg>::mlr_term_debug(state3);
        println!("Parameter estimates: {:?}", result1);

        let result2 = <super::Linreg as super::linreg::Linreg>::mlr_terml_debug(state3a);
        println!("Long results: {:?}", result2);
        assert!(result2[0]==result2[1]  , "Nobs should equal # of coefficients");
        assert!(result2[3].abs() < 1E-7, "SS(Error) = 0");
        assert!(result2[6].abs() < 1E-7, "DF(Error) = 0");
    }

    #[test]
    fn mlr_merge() {
        // Test the merge function in regression
        let state0 = <super::Linreg as super::linreg::Linreg>::mlr_init();
        assert!(state0.nvars == 0);

        let state1 = <super::Linreg as super::linreg::Linreg>::mlr_iter_debug(state0, 44.609, vec![44., 62.]);
        assert!(state1.nvars == 2);
        let state1a = state1.clone();

        let state2 = <super::Linreg as super::linreg::Linreg>::mlr_iter_debug(state1, 45.313, vec![40., 62.]);
        assert!(state2.nvars == 2);
        println!("First  XpX for merge:  {:?}", state1a.xpx);
        println!("Second XpX for merge:  {:?}", state2.xpx);
        let state3 = <super::Linreg as super::linreg::Linreg>::mlr_merge(state1a,state2);
        println!("Merged result       :  {:?}", state3.xpx)
    }

    
    fn mlr_all() {
        regression::slr_basic();
        regression::mlr_basic();
        regression::mlr_sing();
        regression::mlr_merge();

    }

}