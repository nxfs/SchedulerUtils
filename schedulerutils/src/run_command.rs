use std::{
    process::{Child, Command},
    time::Duration, usize,
};
use wait_timeout::ChildExt;

use crate::{cgroup, prctl, proc::wait_for_threads};

pub struct RunCommandCfg {
    pub task: String,
    pub threads: usize,
    pub thread_wait: Duration,
    pub timeout: Duration,
    pub cgroup: String,
    pub cpuset: String,
    pub weight: u64,
    pub cookie_group_size: u64,
    pub cfs_bw_period_us: u64,
    pub cfs_bw_quota_pc: u64,
}

pub fn run_command(cfg: RunCommandCfg) {
    let maybe_cgroup = cgroup::create_cgroup(&cfg.cgroup);
    let mut handles: Vec<Child> = vec![];
    println!("spawning task \"{}\"", cfg.task);
    let tokens: Vec<&str> = cfg.task.split(" ").collect();
    let mut cmd = Command::new(tokens[0]);
    for t in tokens.iter().skip(1) {
        cmd.arg(t);
    }
    let handle = cmd.spawn().unwrap();
    println!("task \"{}\" has pid {}", cfg.task, handle.id());
    let thread_ids = wait_for_threads(handle.id() as i32, cfg.threads, cfg.thread_wait);
    if let Some(ref cgroup) = maybe_cgroup {
        cgroup::add_task_to_cgroup(&cgroup, handle.id() as u64);
        for thread_id in thread_ids.iter() {
            cgroup::add_task_to_cgroup(&cgroup, *thread_id as u64);
        }
        cgroup::set_weight(&cgroup, cfg.weight);
        if cfg.cfs_bw_quota_pc > 0 {
            let quota_us: i64 = (cfg.cfs_bw_period_us * cfg.threads as u64 * cfg.cfs_bw_quota_pc / 100) as i64;
            cgroup::set_quota(&cgroup, quota_us, cfg.cfs_bw_period_us)
        }
        if !cfg.cpuset.is_empty() {
            cgroup::set_cpu_affinity(&cgroup, &cfg.cpuset);
        }
    }
    create_cookies(cfg.cookie_group_size, thread_ids);
    handles.push(handle);
    println!("waiting for all threads to join");
    while let Some(mut handle) = handles.pop() {
        let id = handle.id();
        if !cfg.timeout.is_zero() {
            if handle.wait_timeout(cfg.timeout).unwrap().is_none() {
                println!("timed out waiting for all threads to join, sending kill signal");
                handle.kill().unwrap();
            }
        }
        let out = handle.wait_with_output().unwrap();
        println!(
            "task {}: out='{}', err='{}'",
            id,
            String::from_utf8(out.stdout).unwrap(),
            String::from_utf8(out.stderr).unwrap()
        );
    }
}

pub fn create_cookies(cookie_group_size: u64, thread_ids: Vec<i32>) {
    if cookie_group_size == 0 {
        return;
    }

    // The first (main) thread gets a separate cookie, hence the -1 + 1 dance.
    let cookie_group_count = ((thread_ids.len() - 1) as f64 / cookie_group_size as f64).ceil() as usize + 1;

    let mut pid_grps: Vec<Vec<i32>> = Vec::with_capacity(cookie_group_count as usize);

    for (idx, pid) in thread_ids.iter().enumerate() {
        if idx < cookie_group_count as usize {
            pid_grps.push(vec![]);
        }
        // The first (main) thread gets a separate cookie, hence the -1 + 1 dance.
        if idx == 0 {
            pid_grps[0].push(*pid);
        } else {
            pid_grps[(idx - 1) % (cookie_group_count - 1) as usize + 1].push(*pid);
        }
    }

    prctl::create_cookies(pid_grps);

    for thread_id in thread_ids {
        println!(
            "cookie for {} is {}",
            thread_id,
            prctl::get_cookie(thread_id)
        );
    }
}
