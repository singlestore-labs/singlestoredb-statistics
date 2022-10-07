


#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_variables)]
#![allow(clippy::needless_late_init)]
#![allow(clippy::needless_range_loop)]

wit_bindgen_rust::export!("categorical.wit");

const EPS: f64 = 1e-12;

extern crate statrs;
use statrs::distribution::{ChiSquared, ContinuousCDF};
// extern crate probability;
// use probability::prelude::*;
// use probability::distribution::Gamma;

// Pull structres from the bindings of the wit file
use crate::categorical::Chiresult;
use crate::categorical::State;   /* for pairwise correlation */

// Define a struct to be the concrete implementatoin of the wit interface
 struct Categorical;

 impl categorical::Categorical for Categorical {

    fn chisq_init() -> State {
        State {
            wrk : vec![0.; 3], /* Vec::new() */
        }
    }

    /*---Get col and row number, cell count and marginal totals for */
    /*---one row of the cross-classification. Note that the table   */
    /*---will not contain entries for cells that did not occur in   */
    /*---the data.                                                  */
    fn chisq_iter(in_state:State, nr:i64, nc:i64, cell: f64, rt: f64, ct:f64) -> State {
        let mut st = State { wrk : in_state.wrk.to_vec(),};
        st.wrk[0] = ((nr as f64)-1.) * ((nc as f64)-1.);
        st.wrk[1] += cell;  /* the total */
        st.wrk[2] += (cell / rt) * (cell / ct);  /* n_ij * n_ij / (n_i. * n_.j) */
        st
    }   
    
    fn chisq_merge(st1:State, st2:State) -> State {
        if st1.wrk.is_empty() {
            st2
        } else if st2.wrk.is_empty() {
            st1
        } else {
            let mut out = State {
                wrk  : vec![0.; 3],
            };
            out.wrk[1] += st1.wrk[1] + st2.wrk[1];  /* add to total */
            out.wrk[2] += st1.wrk[2] + st2.wrk[2];  /* add to leading term of test statistic */
            out
        }
    }

    fn chisq_term(st:State) -> Chiresult {
        let mut result = Chiresult{ df : 0., chisq : 0., pvalue : -1. };
        let ntot = st.wrk[1];
        result.df    = st.wrk[0];
        result.chisq = ntot * (st.wrk[2] - 1.); 
        let chidist = ChiSquared::new(result.df).unwrap();
        result.pvalue = 1. - chidist.cdf(result.chisq);
        // let chidist = Gamma::new(result.chisq/2., 0.5);
        // result.pvalue = 1.-chidist.distribution(result.chisq);

        result
    }
}

 