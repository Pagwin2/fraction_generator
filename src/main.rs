extern crate anyhow;
extern crate indicatif;

use std::fs::OpenOptions;
use std::io::Write;
use std::ops::Add;
use std::cmp::PartialOrd;
use std::fmt::{Display,Debug};

//#[tokio::main]
fn main()->anyhow::Result<()>{
    const RECURSIONS:u64 = 20;

    let prog = indicatif::ProgressBar::new(2_u64.pow(RECURSIONS as u32)-1);
    let fracs = recurse(Frac(0,1),Frac(1,0),RECURSIONS,Some(prog));

    let mut opts = OpenOptions::new();
    let mut file = opts.create(true)
        .write(true)
        .append(true)
        .open("./fracts")?;

    // might be worth adding some logic to add line numbers
    for frac in fracs{
        let _ = file.write_all(format!("{}/{}\n",frac.0,frac.1).as_bytes());
    }

    Ok(())
}

/// Returns all the fractions generated but not f1 and f2
fn recurse(f1:Frac,f2:Frac, remaining:u64, progress:Option<indicatif::ProgressBar>)->Vec<Frac>{
    if remaining == 0 {
        return Vec::new()
    }
    let middle = f1+f2;
    if let Some(bar) = progress.clone(){
        bar.inc(1);
    }
    let left = recurse(f1,middle,remaining-1,progress.clone());
    let mut right = recurse(middle,f2,remaining-1,progress);
    
    debug_assert!(left.len() == 0 || left[left.len()-1]<middle,"left assertion failed {} not < {}", left[left.len()-1],middle);
    debug_assert!(right.len() == 0 || right[0]<middle,"right assertion failed {} not > {}", right[right.len()-1],middle);

    let mut ret = left;
    ret.reserve(ret.len()+right.len()+1);
    ret.push(middle);
    ret.append(&mut right);

    return ret
}

#[derive(Clone, Copy, Debug,PartialEq)]
struct Frac(u32,u32);

impl PartialOrd for Frac{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        f64::from(*self).partial_cmp(&f64::from(*other))
    }
}
impl From<Frac> for f64{
    fn from(value: Frac) -> Self {
        (value.0 as f64)/(value.1 as f64)
    }
}
impl Add for Frac{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Frac(self.0+rhs.0,self.1+rhs.1)
    }
}
impl Display for Frac{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}/{}",self.0,self.1))
    }
}
