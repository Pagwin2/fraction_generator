extern crate anyhow;
extern crate indicatif;

use std::cmp::PartialOrd;
use std::collections::VecDeque;
use std::fmt::{Debug, Display};
use std::fs::OpenOptions;
use std::io::Write;
use std::mem::MaybeUninit;
use std::ops::Add;

fn main() -> anyhow::Result<()> {
    const THREAD_RECURSIONS = 2;
    const RECURSIONS: u32 = 29;
    const ELEM_COUNT: u64 = 2_u64.pow(RECURSIONS) + 1;
    let prog = indicatif::ProgressBar::new(ELEM_COUNT);

    let mut opts = OpenOptions::new();
    let mut file = opts
        .create(true)
        .write(true)
        .append(false)
        .open("./fracts")?;
    let fracs = recurse(Frac(0, 1), Frac(1, 0), RECURSIONS, Some(prog));
    // might be worth adding some logic to add line numbers
    for frac in fracs {
        let _ = file.write_all(format!("{}/{}\n", frac.0, frac.1).as_bytes());
    }

    Ok(())
}

struct Task {
    //
    f1: (Frac, usize),
    f2: (Frac, usize),
    remaining_depth: u32,
}

fn recurse(
    f1: Frac,
    f2: Frac,
    max_depth: u32,
    progress: Option<indicatif::ProgressBar>,
) -> Vec<Frac> {
    let ret_size = 2_usize.pow(max_depth) + 1;
    let mut stack: VecDeque<Task> = VecDeque::new();
    stack.push_back(Task {
        f1: (f1, 0),
        f2: (f2, ret_size - 1),
        remaining_depth: max_depth,
    });

    let mut ret: Vec<MaybeUninit<Frac>> =
        Vec::from_iter((0..ret_size).map(|_| MaybeUninit::uninit()));
    ret[0] = MaybeUninit::new(f1);
    ret[ret_size - 1] = MaybeUninit::new(f2);
    while let Some(task) = stack.pop_back() {
        if task.remaining_depth == 0 {
            continue;
        }
        let midpoint = (task.f1.1 + task.f2.1) / 2;
        //eprintln!("{} {} {}", task.f1.1, midpoint, task.f2.1);
        let midpoint_value = task.f1.0 + task.f2.0;
        ret[midpoint] = MaybeUninit::new(midpoint_value);
        if let Some(progress) = progress.clone() {
            progress.inc(1);
        }
        stack.push_back(Task {
            f1: task.f1,
            f2: (midpoint_value, midpoint),
            remaining_depth: task.remaining_depth - 1,
        });
        stack.push_back(Task {
            f1: (midpoint_value, midpoint),
            f2: task.f2,
            remaining_depth: task.remaining_depth - 1,
        });
    }

    return ret
        .into_iter()
        .map(|frac| {
            // Safety: if we've generated every fraction down to max_depth then every slot in the Vec
            // should be filled
            unsafe { frac.assume_init() }
        })
        .collect();
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Frac(u32, u32);

impl PartialOrd for Frac {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        f64::from(*self).partial_cmp(&f64::from(*other))
    }
}
impl From<Frac> for f64 {
    fn from(value: Frac) -> Self {
        (value.0 as f64) / (value.1 as f64)
    }
}
impl Add for Frac {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Frac(self.0 + rhs.0, self.1 + rhs.1)
    }
}
impl Display for Frac {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}/{}", self.0, self.1))
    }
}
