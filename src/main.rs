extern crate anyhow;
extern crate indicatif;

use std::fs::OpenOptions;
use std::ops::Add;
use std::io::Write;

#[derive(Clone, Copy)]
struct Frac(u16,u16);

impl Add for Frac{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Frac(self.0+rhs.0,self.1+rhs.1)
    }
}
fn main()->anyhow::Result<()>{
    let mut fracs = Vec::<Frac>::new();
    fracs.reserve_exact(2_usize.pow(16)+1);

    // gotta seed the list with 2 starting values due to the algorithm being used(these are the
    // upper and lower bounds)
    fracs.push(Frac(0,1));
    fracs.push(Frac(1,0));

    #[allow(non_upper_case_globals)]
    const iterations:usize = 19;

    for i in 0..iterations{
        //if i%10 ==0 {
        //    eprintln!("{}\t{}",i,fracs.len());
        //}
        eprintln!("{}",i);
        step(&mut fracs);
    }

    let mut opts = OpenOptions::new();
    let mut file = opts.create(true)
        .write(true)
        .append(true)
        .open("./fracts")?;
    // might be worth adding some logic to add line numbers and remove 1/0
    for frac in fracs{
        let _ = file.write_fmt(format_args!("{}/{}\n",frac.0,frac.1));
    }
    Ok(())
}
fn step(list:&mut Vec<Frac>){
    let mut i = 0;
    let bar = indicatif::ProgressBar::new(list.len() as u64);
    while i < list.len()-1{
        bar.inc(1);
        list.insert(i+1,list[i]+list[i+1]);
        i+=2;
    }
    bar.finish_and_clear();
}
