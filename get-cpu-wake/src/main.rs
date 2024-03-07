extern crate libc;

use clap::Parser;
use libc::sched_getcpu;
use rand::{thread_rng, Rng};
use std::process;
use std::thread;
use std::thread::sleep;
use std::time::{Duration, Instant};

///
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// the delay between two busy iterations, in milliseconds
    #[arg(short = 's', long, default_value = "1")]
    sleep_ms: u64,

    /// the number of spawned threads
    #[arg(short = 't', long, default_value = "1")]
    threads: usize,

    /// the min amount of work for each loop
    #[arg(short = 'm', long, default_value = "1")]
    min_work: u64,

    /// the max amount of work for each loop
    #[arg(short = 'M', long, default_value = "10000")]
    max_work: u64,
}
fn main() {
    let cli = Cli::parse();

    let mut handles = vec![];

    for i in 0..cli.threads {
        let handle = thread::spawn(move || {
            get_cpu(i, cli.sleep_ms, cli.min_work, cli.max_work);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }
}

fn sort(n: u64) {
    let mut rng = thread_rng();
    let mut vec = (0..n).map(|_| rng.gen_range(0..=n)).collect::<Vec<_>>();
    vec.sort();
}

fn keep_busy(n: u64) {
    sort(n);
}

fn get_cpu(thread_id: usize, sleep_ms: u64, min_work: u64, max_work: u64) {
    let mut cpu: i32;
    let mut prev_cpu: i32 = -1;
    let pid = process::id();
    let tid = get_tid();
    let mut rng = thread_rng();
    let start = Instant::now();

    let mut it = 0;
    loop {
        cpu = unsafe { sched_getcpu() };
        if cpu == -1 {
            process::exit(1);
        }
        if cpu != prev_cpu {
            let duration = start.elapsed();
            let duration_per_loop = if it > 0 {
                duration / it
            } else {
                Duration::ZERO
            };
            println!(
                "Thread {} (TID: {}) - PID {} running on CPU {} after {} loops, loop velocity is {:?}",
                thread_id, tid, pid, cpu, it, duration_per_loop
            );
        }

        prev_cpu = cpu;

        let n = rng.gen_range(min_work..=max_work);
        keep_busy(n);

        if sleep_ms > 0 {
            sleep(Duration::from_micros(sleep_ms));
        }

        it += 1;
    }
}

fn get_tid() -> libc::pid_t {
    unsafe { libc::syscall(libc::SYS_gettid) as libc::pid_t }
}
