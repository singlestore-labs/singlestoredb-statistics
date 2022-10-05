


#![allow(dead_code)]
#![allow(unused_assignments)]
#![allow(unused_variables)]
#![allow(clippy::needless_late_init)]
#![allow(clippy::needless_range_loop)]

wit_bindgen_rust::export!("categorical.wit");

const EPS: f64 = 1e-12;

extern crate probability;
use probability::prelude::*;
use probability::distribution::Gamma;

//use rstat::univariate::chi_sq::ChiSq;
//use rstat::Distribution; 


// Pull structres from the bindings of the wit file
use crate::categorical::Chiresult;
use crate::categorical::State;   /* for pairwise correlation */
// use byte_slice_cast::*;


// define a struct to be the concrete implementatoin of the wit interface
 struct Categorical;

 impl categorical::Categorical for Categorical {

    fn chisq2_init() -> State {
        State {
            wrk : Vec::new()
        }
    }

    /*---Get col and row number, cell count and marginal totals for */
    /*---one row of the cross-classification. Note that the table   */
    /*---will not contain entries for cells that did not occur in   */
    /*---the data.                                                  */
    /* returns query(NumRows int, NumCols int, RowNum int, ColNum int, Cell double, RowTotal double, ColTotal double)
        as declare */
    fn chisq2_iter(in_state:State, nr:i64, nc:i64, i:i64, j:i64, cell: f64, rt: f64, ct:f64) -> State {
        let nalloc : usize = (3 + nr + nc) as usize;
        let roffset: usize = 3;
        let coffset: usize = (3 + nr) as usize;
        let mut st: State;
        if in_state.wrk.is_empty() {
            st = State { wrk  : vec![0.; nalloc], };
        } else {
            st = State { wrk  : in_state.wrk.to_vec(),};
        }
        st.wrk[0] = ((nr as f64)-1.) * ((nc as f64)-1.);
        st.wrk[1] += cell;  /* the total */
        st.wrk[2] += (cell / rt) * (cell / ct);
        if st.wrk[roffset+i as usize] == 0. { st.wrk[roffset+i as usize] = rt; } /* row total    */
        if st.wrk[coffset+j as usize] == 0. { st.wrk[coffset+j as usize] = ct; } /* column total */
        st
    }   
    
    fn chisq2_merge(st1:State, st2:State) -> State {
        if st1.wrk.is_empty() {
            st2
        } else if st2.wrk.is_empty() {
            st1
        } else {
            let mut out = State {
                wrk  : st1.wrk.to_vec(),
            };
            out.wrk[1] += st2.wrk[1];  /* add to total */
            out.wrk[2] += st2.wrk[2];  /* add to test statistic */
            out
        }
    }

    fn chisq2_term(st:State,) -> Chiresult {
        let mut result = Chiresult{ df : 0., chisq : 0., pvalue : -1. };
        let ntot = st.wrk[1];
        result.df = st.wrk[0];
        result.chisq = ntot * (st.wrk[2] - 1.);
        let chidist = Gamma::new(result.chisq/2., 0.5);
        result.pvalue = chidist.distribution(result.chisq);

        result
    }
}

 