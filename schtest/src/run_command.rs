use std::{
    process::{Child, Command},
    time::Duration,
};
use wait_timeout::ChildExt;

use crate::{cgroup, proc::wait_for_threads};

pub struct RunCommandCfg {
    pub task: String,
    pub threads: usize,
    pub thread_wait: Duration,
    pub timeout: Duration,
    pub cgroup: String,
    pub cpuset: String,
    pub weight: u64,
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
    if let Some(ref cgroup) = maybe_cgroup {
        cgroup::add_task_to_cgroup_by_tgid(&cgroup, handle.id() as u64);
        cgroup::set_weight(&cgroup, cfg.weight);
        if !cfg.cpuset.is_empty() {
            cgroup::set_cpu_affinity(&cgroup, &cfg.cpuset);
        }
    }
    wait_for_threads(handle.id() as i32, cfg.threads, cfg.thread_wait);
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
