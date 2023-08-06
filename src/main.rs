extern crate anyhow;
extern crate indicatif;
extern crate tokio;
extern crate futures;

use tokio::fs::OpenOptions;
use tokio::io::AsyncWriteExt;

use std::ops::Add;
use std::future::Future;
use std::pin::Pin;

#[derive(Clone, Copy)]
struct Frac(u16,u16);

impl Add for Frac{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Frac(self.0+rhs.0,self.1+rhs.1)
    }
}

#[tokio::main]
async fn main()->anyhow::Result<()>{
    const RECURSIONS:u64 = 19;

    let prog = indicatif::ProgressBar::new(2_u64.pow(RECURSIONS as u32)-1);
    let fracs = recurse(Frac(0,1),Frac(1,0),RECURSIONS,Some(prog)).await;

    let mut opts = OpenOptions::new();
    let mut file = opts.create(true)
        .write(true)
        .append(true)
        .open("./fracts").await?;

    // might be worth adding some logic to add line numbers
    for frac in fracs{
        let _ = file.write_all(format!("{}/{}\n",frac.0,frac.1).as_bytes()).await;
    }

    Ok(())
}

/// Returns all the fractions generated but not f1 and f2
/// pinned box is necessary to allow recursion
fn recurse(f1:Frac,f2:Frac, remaining:u64, progress:Option<indicatif::ProgressBar>)->Pin<Box<dyn Future<Output = Vec<Frac>> + Send + 'static>>{
    // the things I do for recursion
    Box::pin(async move{
        if remaining == 0 {
            return Vec::new()
        }
        let middle = f1+f2;
        if let Some(bar) = progress.clone(){
            tokio::task::spawn_blocking(move ||bar.inc(1));
        }
        let left_task = tokio::task::spawn(recurse(f1,middle,remaining-1,progress.clone()));
        let right_task = tokio::task::spawn(recurse(middle,f2,remaining-1,progress));

        let left = left_task.await.expect("left future failure");
        let mut right = right_task.await.expect("right future failure");
        let mut ret = left;
        ret.reserve(ret.len()+right.len()+1);
        ret.push(middle);
        ret.append(&mut right);

        return ret
    })
}

