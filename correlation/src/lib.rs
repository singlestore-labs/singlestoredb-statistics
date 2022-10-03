
#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(clippy::needless_late_init)]
#![allow(clippy::needless_range_loop)]

wit_bindgen_rust::export!("correlation.wit");

const EPS: f64 = 1e-12;

// Pull structres from the bindings of the wit file
use crate::correlation::C2summary;
use crate::correlation::C2state;   /* for pairwise correlation */
use crate::correlation::Cmstate;    /* for vorrelation matrix */
use byte_slice_cast::*;


// define a struct to be the concrete implementatoin of the wit interface
 struct Correlation;

 fn vector_dot_product(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}

fn vector_mult(a: &[f64], b: f64) -> Vec<f64> {
    a.iter().map(|a| a * b).collect()
}

impl correlation::Correlation for Correlation {

    /*---------------------------------------------------- */
    /* Functions for computing the pair-wise Pearson       */
    /* product-moment correlation between two numeric      */
    /* variables begin here                                */
    /*---------------------------------------------------- */
  
    fn corr2_init() -> C2state {
        C2state {
            wrk   : vec![0.; 5],  /* [sumx, sumy, sumxy, sumxx, sumyy] */
            n     : vec![0 ; 2],  /* [nxy, nmiss] */
        }
    }

    fn corr2_iter(in_state: C2state, x: f64,y: f64) -> C2state { 
        
        let mut s = C2state {
            wrk  : in_state.wrk.to_vec(), 
            n    : in_state.n.to_vec(),
        };
 
        if !(x.is_nan() || y.is_nan()) {
            s.wrk[0] += x;
            s.wrk[1] += y;
            s.wrk[2] += x*y;
            s.wrk[3] += x*x;
            s.wrk[4] += y*y;
            s.n[0]   += 1;
        } else {
            s.n[1]   += 1;  /* # of pairs with missing values */
        }
        s
    }  

    fn corr2_merge(st1: C2state, st2: C2state) -> C2state {
        let mut out:C2state = Self::corr2_init();
        for i in 0..st1.wrk.len() {
            out.wrk[i] = st1.wrk[i] + st2.wrk[i];
        }
        out.n[0] = st1.n[0] + st2.n[0];
        out.n[1] = st1.n[1] + st2.n[1];
        out
    }

    fn corr2_term(st: C2state) -> C2summary {
        let mut out = C2summary{n:0, nmiss:0, x_avg:0., y_avg:0., corr:0., r2:0., b0:0.,b1:0., sse:0.};
        out.nmiss = st.n[1];  /* # of pairs with missing values */
        let nxy = st.n[0] as f64;
    
        if st.n[0] > 0 {
            let sxx = st.wrk[3] - st.wrk[0] * st.wrk[0] / nxy;
            let syy = st.wrk[4] - st.wrk[1] * st.wrk[1] / nxy;
            let sxy = st.wrk[2] - st.wrk[0] * st.wrk[1] / nxy;
    
            out.x_avg = st.wrk[0] / nxy;
            out.y_avg = st.wrk[1] / nxy;
            if (sxx > EPS) && (syy > EPS) {
                out.corr = sxy / (sxx * syy).sqrt();
                out.b1   = sxy/sxx;
                out.b0   = out.y_avg - out.b1*out.x_avg;
                out.sse  = syy - out.b1*sxy;
                out.r2   = 1. - (out.sse/syy);
            } else {
                out.corr = -99.0;
                out.b1   = -99.0;
                out.b0   = out.y_avg - out.b1*out.x_avg;
                out.sse  = -99.0;
                out.r2   = -99.0;
            }
            out.n     = st.n[0];
        } else {
            out.x_avg =  -99.0;
            out.y_avg =  -99.0;
            out.corr  =  -99.0;
            out.r2    =  -99.0;
            out.b0    =  -99.0;
            out.b1    =  -99.0;
            out.sse   =  -99.0;
            out.n     = -1;
        }
        out
    }

    fn corr2_termd(st: C2state) -> f64 {
        let mut corr = -99.0;
        if st.n[0] > 0 {
            let nxy = st.n[0] as f64;
            let sxx = st.wrk[3] - st.wrk[0] * st.wrk[0] / nxy;
            let syy = st.wrk[4] - st.wrk[1] * st.wrk[1] / nxy;
            let sxy = st.wrk[2] - st.wrk[0] * st.wrk[1] / nxy;
            if (sxx > EPS) && (syy > EPS) {
                corr = sxy / (sxx * syy).sqrt();
            }
        }
        corr
    }
   /*---------------------------------------------------- */
    /* eo corr2_d */


    /*---------------------------------------------------- */
    /* Functions for computing the matrix of correlations  */
    /* between all pairs of input variables start here.    */
    /*---------------------------------------------------- */
     
    fn vec_pack_f64(v: Vec<f64>) -> Vec<u8> {
        v.as_byte_slice().to_vec()
    }

    fn vec_unpack_f64(v: Vec<u8>) -> Vec<f64> {
        v.as_slice_of::<f64>().unwrap().to_vec()
    }

    fn corrmat_init() -> Cmstate {
        Cmstate {
            nvar  : 0,
            nxy   : Vec::new(),
            sx    : Vec::new(),
            sy    : Vec::new(),
            sxx   : Vec::new(),
            syy   : Vec::new(),
            sxy   : Vec::new(),
        }
    }

    fn corrmat_iter(in_state: Cmstate, vars: Vec<u8>) -> Cmstate {     
        let data_row = Self::vec_unpack_f64(vars);
        let nvar = data_row.len();
        assert!(nvar > 1);
        let ncor = if nvar==2 { 1 } else { nvar*(nvar-1)/2 };
        let mut st : Cmstate;

        // Allocate an empty vector on the first go-around
        // and add to the existing vector afterwards.
        // Note that we need to make a copy of the incoming state
        if in_state.nxy.is_empty() {
            st = Cmstate {
                nvar : nvar as i64,
                nxy  : vec![0.; ncor],
                sx   : vec![0.; ncor],
                sy   : vec![0.; ncor],
                sxx  : vec![0.; ncor],
                syy  : vec![0.; ncor],
                sxy  : vec![0.; ncor],
            };
        } else {
            st = Cmstate {
                nvar : nvar as i64,
                nxy  : in_state.nxy.to_vec(), 
                sx   : in_state.sx.to_vec(),
                sy   : in_state.sy.to_vec(),
                sxx  : in_state.sxx.to_vec(),
                syy  : in_state.syy.to_vec(),
                sxy  : in_state.sxy.to_vec(),
            };
        }

        let mut index: usize = 0;
        for i in 0..nvar {
            let x = data_row[i];
            if x.is_nan() { index += nvar-i-1; continue; }
            for j in 0..i+1 {
                if i == j { continue; }
                let y: f64 = data_row[j];
                if y.is_nan() { index += 1; continue; }
                assert!(index < ncor);
                st.sx [index] += x;
                st.sy [index] += y;
                st.nxy[index] += 1.0;
                st.sxx[index] += x*x;
                st.syy[index] += y*y;
                st.sxy[index] += x*y;
                index += 1;
            }
        }
        st
    }  

    fn corrmat_merge(st1: Cmstate, st2: Cmstate) -> Cmstate {
        if st1.nxy.is_empty() {
            st2
        } else if st2.nxy.is_empty() {
            st1
        } else {
            let nvar = st1.nvar as usize;
            let ncor = if nvar==2 { 1 } else { nvar*(nvar-1)/2 };
            let mut out = Cmstate {
                nvar  : nvar as i64,
                nxy   : vec![0.; ncor],
                sx    : vec![0.; ncor],
                sy    : vec![0.; ncor],
                sxx   : vec![0.; ncor],
                syy   : vec![0.; ncor],
                sxy   : vec![0.; ncor],
            };

            for i in 0..ncor {
                out.sx [i] = st1.sx [i] + st2.sx [i];
                out.sy [i] = st1.sy [i] + st2.sy [i];
                out.sxx[i] = st1.sxx[i] + st2.sxx[i];
                out.syy[i] = st1.syy[i] + st2.syy[i];
                out.sxy[i] = st1.sxy[i] + st2.sxy[i];
                out.nxy[i] = st1.nxy[i] + st2.nxy[i];
            }
            out
        }
    }

    fn corrmat_term(st: Cmstate) -> Vec<u8>  {
        let nvar = st.nvar as usize;
        let nsym = nvar*(nvar+1)/2;
        let mut result = vec![0.; nsym];

        /*---Compute correlations from individual sums of squares and cross-products */
        /*---This assume that each pair of variables can have a different missing    */
        /*---value pattern.                                                          */
        /*---Expand the result matrix to lower triangular storage (nsym)             */
        let mut index: usize = 0;
        let mut corij = 0;
        
        for i in 0..nvar {
            for j in 0..i+1 {
                if i == j {
                    result[index] = 1.0;    
                } else {
                    let sxy = st.sxy[corij] - st.sx[corij] * st.sy[corij] / st.nxy[corij];
                    let sxx = st.sxx[corij] - st.sx[corij] * st.sx[corij] / st.nxy[corij];
                    let syy = st.syy[corij] - st.sy[corij] * st.sy[corij] / st.nxy[corij];
                    if (sxx > EPS) && (syy > EPS) {
                        result[index] = sxy / (sxx * syy).sqrt();
                    } else {
                        result[index] = -99.0;
                    }
                    corij += 1;
                }
                index += 1;
            }
        }
        /* pack the result vector */
        Self::vec_pack_f64(result)  
    }   
    /*---------------------------------------------------- */
    /* eo corrmat*/
}
    