extern crate anyhow;
extern crate indicatif;

use std::fs::OpenOptions;
use std::io::Write;
use std::ops::Add;
use std::cmp::PartialOrd;
use std::fmt::{Display,Debug};
use std::collections::VecDeque;
use std::sync::{Arc, RwLock};

fn main()->anyhow::Result<()>{
    const RECURSIONS:u64 = 26;
    const ELEM_COUNT:usize = 2_usize.pow(RECURSIONS as u32)-1;
    let prog = indicatif::ProgressBar::new((2*ELEM_COUNT) as u64);
    //let fracs = recurse(Frac(0,1),Frac(1,0),RECURSIONS,Some(prog));
    
    // need to stick the buffer onto the heap so we can send it between threads
    // could init as zeroed or uninit for speed gain maybe but eh rust compiler can probably figure
    // it out
    let buffer: &'static mut [Frac] = Box::leak(Box::<[Frac;ELEM_COUNT]>::new([Frac(0,0);ELEM_COUNT]));

    //we're never going to touch buffer and fracs at the same time but I don't want to bother
    //proving that to the rust compiler with lifetimes
    let fracs = unsafe{(buffer as *mut [Frac]).clone().as_ref().unwrap()};

    let mut queue = VecDeque::<QueueTask>::with_capacity(ELEM_COUNT);
    queue.push_back(QueueTask{buffer,frac1:Frac(0,1),frac2:Frac(1,0)});
    buf_gen_fracs::<1>(queue, Some(prog));

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

struct QueueTask {
    frac1: Frac,
    frac2: Frac,
    buffer: &'static mut [Frac],
}
type QueueType<T> = Arc<RwLock<VecDeque<T>>>;
//queue is taken as an argument to allow for optimizations around the buffer being a fixed number
//of elements
fn buf_gen_fracs<const THREAD_COUNT:usize>(queue:VecDeque<QueueTask>, bar: Option<indicatif::ProgressBar>){
    let shared_queue:QueueType<QueueTask> = Arc::new(RwLock::new(queue));
    let mut threads:Vec<std::thread::JoinHandle::<()>> = Vec::with_capacity(THREAD_COUNT);
    for _ in 0..THREAD_COUNT {
        let queue_copy = shared_queue.clone();
        let bar_copy = bar.clone();
        threads.push(std::thread::spawn(move || worker_loop(queue_copy, bar_copy)));
    }
    for t in threads{ let _ = t.join();}
}
fn worker_loop(mut queue:QueueType<QueueTask>, mut progress_bar:Option<indicatif::ProgressBar>){
    while let Some(task) = fetch_task(&mut queue){
        let new_tasks = exec_task(task);
        if let Some((t1,t2)) = new_tasks {
            append_tasks(&mut queue, (t1,t2));
        }
        if let Some(bar) = &mut progress_bar{
            bar.inc(1);
        }
    }
}
fn fetch_task(queue:&mut QueueType<QueueTask>)->Option<QueueTask>{
    let handle = queue.write();
    match handle{
        Ok(mut handle)=>{
            handle.pop_front()
        }
        Err(_)=>{
            eprintln!("Error when getting queue handle in thread {:?}", std::thread::current().id());
            None
        }
    }
}
fn exec_task(task:QueueTask)->Option<(QueueTask, QueueTask)>{
    let QueueTask{frac1,frac2,buffer} = task;
    
    let middle = frac1+frac2;
    buffer[buffer.len()/2] = middle;
    
    // we don't wanna infinitely add tasks to the queue
    if buffer.len() == 1 {
        return None;
    }

    let (first_half,second_half) = buffer.split_at_mut(buffer.len()/2);
    Some(
        (QueueTask{
            frac1,
            frac2:middle,
            buffer:first_half
        },
        QueueTask{
            frac1:middle,
            frac2,
            buffer:second_half
        }))
}
fn append_tasks(queue:&mut QueueType<QueueTask>, (task1, task2):(QueueTask,QueueTask)){
    let mut handle = queue.write().expect("Queue wouldn't accept another task, fractions were lost");
    handle.push_back(task1);
    handle.push_back(task2);
}
/// Returns all the fractions generated but not f1 and f2 (slow version that uses vec rather than a
/// buffer)
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
