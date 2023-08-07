extern crate anyhow;
extern crate indicatif;

use std::fs::OpenOptions;
use std::io::Write;
use std::ops::Add;


#[derive(Clone, Copy)]
struct Frac(u16,u16);

impl Add for Frac{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Frac(self.0+rhs.0,self.1+rhs.1)
    }
}

//#[tokio::main]
fn main()->anyhow::Result<()>{
    const RECURSIONS:u64 = 25;

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
/// pinned box is necessary to allow recursion
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

    let mut ret = left;
    ret.reserve(ret.len()+right.len()+1);
    ret.push(middle);
    ret.append(&mut right);

    return ret
}

