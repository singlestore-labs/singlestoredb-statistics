

#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(clippy::needless_late_init)]
#![allow(clippy::needless_range_loop)]

wit_bindgen_rust::export!("linreg.wit");

use byte_slice_cast::*;

use crate::linreg::State;
use crate::linreg::Slrstate;
use crate::linreg::Aovstate;
use crate::linreg::Slrsummary;
use crate::matrix::*;
use crate::aovreg::*;

pub mod matrix; 
pub mod aovreg;

struct Linreg;

impl linreg::Linreg for Linreg {

    /*----------------------------------------------------------*/
    /* AOV                                                      */
    fn aov_init() -> Aovstate {
        Aovstate { efflev: Vec::new(), xpx: Vec::new(),}
    }
 
    /* aov_agg(vec_pack_f64([ NumX0, NumX1, Target, X0Level, X1Level])) */
    fn aov_iter_debug(mut in_state:Aovstate, xrow: Vec<f64> ) -> Aovstate {
        let data_packed = Self::vec_pack_f64(xrow);
        Self::aov_iter(in_state,data_packed)
    }

    fn aov_iter(mut in_state:Aovstate, xpacked:Vec<u8>) -> Aovstate {
        let data_row = Self::vec_unpack_f64(xpacked);

        if in_state.xpx.is_empty() {
            // nfac  = number of factors
            // nc1   = size of x || y row, includes intercept
            let nfac   = (data_row.len()-1)/2;
            let nc1 = 2 + data_row[0..nfac].iter().sum::<f64>() as usize;
            let nsym1: usize = nc1 * (nc1 + 1) / 2;
            in_state.efflev = vec![0 ; nfac+1];
            in_state.xpx    = vec![0.; nsym1 ];

            in_state.efflev[0] = 1; // intercept
            for i in 0..nfac {
                in_state.efflev[i+1] = data_row[i] as i64;
            }
        }
        aov_add_row(data_row, &mut in_state.xpx, &in_state.efflev);
        in_state
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
          aov_terminate(&mut st.xpx, &st.efflev)
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
        slr_add_row(y, x, &mut st.xpx);
        st
    }

    fn slr_merge(mut st1:Slrstate, mut st2:Slrstate) -> Slrstate {
        vector_add(&mut st1.xpx,&st2.xpx);
        st1
    }

    fn slr_term(mut st:Slrstate) -> Slrsummary {
        let mut out = Slrsummary{b0:0., b1:0., n:0., r2: 0., sse:0., pvalue:0.,};
        let mut slrout = Slroutput{..Default::default()};

        slrout = slr_terminate(&mut st.xpx, slrout);

        out.n      = slrout.n  as f64;
        out.b0     = slrout.b0;
        out.b1     = slrout.b1;
        out.sse    = slrout.sse;
        out.r2     = slrout.r2;
        out.pvalue = slrout.pval;

        out
   }
    /* eo SLR                                                   */
    /*----------------------------------------------------------*/

    /*----------------------------------------------------------*/
    /* MLR Begin                                                */
    fn mlr_init() -> State {
        State {nvars: 0, xpx: Vec::new(), }
    }

    fn mlr_iter_debug(in_state:State, y:f64, xrow:Vec<f64>) -> State {
        let data_packed = Self::vec_pack_f64(xrow);
        Self::mlr_iter(in_state,y, data_packed)
    }

    fn mlr_iter(mut in_state:State, y: f64, vars:Vec<u8>) -> State {
        let data_row = Self::vec_unpack_f64(vars);

        if in_state.xpx.is_empty() {
            let nvars  = data_row.len();     /* number of regressors   */
            let nc   : usize = nvars+1;             /* number of coefficients */
            let nc1  : usize = nc + 1;              /* size of x || y row     */
            let nsym1: usize = nc1 * (nc1 + 1) / 2; /* room for Y-border      */
            in_state.nvars = nvars as i64;
            in_state.xpx   = vec![0.; nsym1];
        }
        mlr_add_row(y,data_row,&mut in_state.xpx);
        in_state
    }

    fn mlr_merge(mut st1:State,mut st2:State,) -> State {
        if st1.xpx.is_empty() {
            st2
        } else if st2.xpx.is_empty() {
            st1
        } else {
            vector_add(&mut st1.xpx,&st2.xpx);
            st1
        }
    }

    fn mlr_term_debug(mut st:State) -> Vec<f64> {   
        let nvars = st.nvars as usize;
        let nc   : usize = nvars+1;  // accounts for intercept
        let nsym   = nc * (nc+1)/2;
        let mut result : Vec<f64> = vec![0.;nc];

        mlr_terminate(&mut st.xpx,nc);
        result[0..nc].clone_from_slice(&st.xpx[nsym..(nsym+nc)]);
        result
    }

    /* return only the regression coefficients */
    fn mlr_term(mut st: State) -> Vec<u8> {
        Self::vec_pack_f64(Self::mlr_term_debug(st))
    } 

    fn mlr_terml_debug(mut st: State) -> Vec<f64> {
        let nvars = st.nvars as usize;
        mlr_terminate_long(&mut st.xpx,nvars)
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