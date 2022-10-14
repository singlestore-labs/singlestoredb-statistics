/*---------------------------------------------------------------*/
/* Some basic vector and matrix routines to support the          */
/* statistical calculations.                                     */
/*---------------------------------------------------------------*/

const EPS: f64 = 1e-12;

fn vector_dot_product(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b).map(|(a, b)| a * b).sum()
}

fn vector_mult(a: &[f64], b: f64) -> Vec<f64> {
    a.iter().map(|a| a * b).collect()
}

pub fn vector_add(a: &mut [f64], b: &[f64]) -> Vec<f64> {
    for i in 0..a.len() {
        a[i] += b[i];
    }
    a.to_vec()
    //  a.iter().zip(b).map(|(a, b)| a + b).collect()
}

fn mvaxpy(v : &mut [f64], x : &[f64], a : &f64, n : usize) {
    for i in 0..n {
        v[i] += a*x[i];
    }
}
pub fn sscp(xpx : &mut Vec<f64>, xrow : &[f64], _nc : usize) {
    let mut start_pos =0;
    for (i,xi) in xrow.iter().enumerate() {
        // println!("x[{:?}] = {:?}",i,xi);
        mvaxpy(&mut xpx[start_pos..],xrow,xi,i+1);
        start_pos += i+1;
    }
}

/*----------------------------------------------------------- */
/* Sweep an effect of nlev levels  out of a cross-product     */
/* matrix. The effect columns start at k and end at k+nlev    */
/* The function returns the errors sum of squares after having*/
/* swept out the effect.                                      */
pub fn sweep_eff(xpx: &mut [f64], nc:usize, k: usize, nlev:usize, work: &mut [f64]) -> usize {
    assert!(k+nlev < nc);
    let mut sse = 0.;
    let mut nswept: usize = 0;
    for i in k..(k+nlev) {
        if sweep_row(xpx, nc, i, work) {
            nswept += 1;
        }
    }
    nswept
}
/*----------------------------------------------------------- */




/*----------------------------------------------------------- */
/* Sweep a cross-product matrix on all rows. Assuming that    */
/* the first column corresponds to the intercept, return the  */
/* corrected total sums of squares after sweeping out the     */
/* first row.                                                 */
pub fn sweep_xpx(xpx: &mut [f64], nc:usize, work: &mut [f64]) -> f64 {
    let mut ss_total = 0.0;
    for k in 0..(nc-1) {
        sweep_row(xpx, nc, k, work);
        if k==0 {
            ss_total = xpx[(nc*(nc+1)/2)-1];
        }
    }
    ss_total
}
/*----------------------------------------------------------- */


// Sweep the matrix xpx (symmetric row storage) on row k
fn sweep_row(xpx : &mut [f64], n:usize, k : usize, work : &mut [f64]) -> bool {
    let trik = k*(k+1)/2;
    let mut start_pos = 0;
    let d = xpx[trik+k]; // the pivot element
    let mut swept = true;
    if d.abs() < EPS {
        // zero the kth row and column
        for i in 0..k {
            xpx[trik+i] = 0.;
        }
        for i in (k+1)..n {
            start_pos = i * (i+1) / 2;
            xpx[start_pos+k] = 0.0;
        }
        swept = false;
    } else {
        work[..k].clone_from_slice(&xpx[trik..trik+k]);
        
        for (i, wi) in work[(k+1)..].iter_mut().enumerate() {
            start_pos = (i+k+1) * (i+k+2)/2;
            *wi = xpx[start_pos + k];
        }
 
        for i in 0..n {
            start_pos = i * (i+1) / 2;
        
            for j in 0..(i+1) {
                if i==k && j==k {
                    xpx[start_pos+j] = -1./d;
                } else if (i==k) || (j==k) {
                    xpx[start_pos+j] /= d;
                } else if i < k {
                    xpx[start_pos+j] -= xpx[trik+i]*xpx[trik+j]/d;
                } else {
                    xpx[start_pos+j] -= work[i]*work[j]/d;
                }
            }
        }
    }
    swept
}
