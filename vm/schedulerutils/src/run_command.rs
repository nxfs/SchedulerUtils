use cgroups_rs::Cgroup;
use nix::sys::signal;
use nix::sys::signal::Signal::SIGTERM;
use nix::unistd::Pid;
use std::{process::Command, time::Duration, usize};
use wait_timeout::ChildExt;

use crate::{
    cgroup::{self, create_cgroup},
    prctl,
    proc::{get_all_children_processes_ids, wait_for_threads},
};

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
    pub cookie_affinity: bool,
}

pub fn run_command(cfg: RunCommandCfg) {
    let maybe_cgroup = cgroup::create_cgroup(&cfg.cgroup);
    println!("spawning task \"{}\"", cfg.task);
    let tokens: Vec<&str> = cfg.task.split(" ").collect();
    let mut cmd = Command::new(tokens[0]);
    for t in tokens.iter().skip(1) {
        cmd.arg(t);
    }
    let mut handle = cmd.spawn().unwrap();
    println!("task \"{}\" has pid {}", cfg.task, handle.id());
    let thread_ids = wait_for_threads(handle.id() as i32, cfg.threads, cfg.thread_wait);
    if let Some(ref cgroup) = maybe_cgroup {
        // with cookie affinity we are going to create sub cgroups and move tasks there instead
        if !cfg.cookie_affinity {
            cgroup::add_task_to_cgroup(&cgroup, handle.id() as u64);
            for thread_id in thread_ids.iter() {
                cgroup::add_task_to_cgroup(&cgroup, *thread_id as u64);
            }
        }
        cgroup::set_weight(&cgroup, cfg.weight);
        if cfg.cfs_bw_quota_pc > 0 {
            let quota_us: i64 =
                (cfg.cfs_bw_period_us * cfg.threads as u64 * cfg.cfs_bw_quota_pc / 100) as i64;
            cgroup::set_quota(&cgroup, quota_us, cfg.cfs_bw_period_us)
        }
        if !cfg.cpuset.is_empty() {
            cgroup::set_cpu_affinity(&cgroup, &cfg.cpuset);
        }
    }
    create_cookies(
        cfg.cookie_group_size,
        thread_ids,
        cfg.cookie_affinity,
        maybe_cgroup.as_ref(),
    );

    println!("waiting for task to join");
    let id = Pid::from_raw(handle.id().try_into().unwrap());
    if !cfg.timeout.is_zero() {
        if handle.wait_timeout(cfg.timeout).unwrap().is_none() {
            println!(
                "timed out waiting for all threads to join, sending kill signal to {}",
                id
            );
            if let Err(err) = signal::kill(id, Some(SIGTERM)) {
                eprintln!("failed to kill process {}: {:?}", id, err);
            }
        } else {
            let out = handle.wait_with_output().unwrap();
            println!(
                "task {}: out='{}', err='{}'",
                id,
                String::from_utf8(out.stdout).unwrap(),
                String::from_utf8(out.stderr).unwrap()
            );
        }
    }

    // more cleanup
    let current_pid = Pid::this();
    for child_pid in get_all_children_processes_ids(current_pid.as_raw()) {
        if child_pid == current_pid.as_raw() {
            continue;
        }
        println!("killing remaining pid {}", child_pid);
        if let Err(err) = signal::kill(Pid::from_raw(child_pid), Some(SIGTERM)) {
            eprintln!("failed to kill process {}: {:?}", id, err);
        }
    }
}

pub fn create_cookies(
    cookie_group_size: u64,
    thread_ids: Vec<i32>,
    cookie_affinity: bool,
    cgroup: Option<&Cgroup>,
) {
    if cookie_group_size == 0 {
        return;
    }

    // The first (main) thread gets a separate cookie, hence the -1 + 1 dance.
    let cookie_group_count =
        ((thread_ids.len() - 1) as f64 / cookie_group_size as f64).ceil() as usize + 1;

    let mut pid_grps: Vec<Vec<i32>> = Vec::with_capacity(cookie_group_count as usize);
    let mut cgroups: Vec<Cgroup> = Vec::with_capacity(cookie_group_count as usize);

    for (idx, pid) in thread_ids.iter().enumerate() {
        if idx < cookie_group_count as usize {
            pid_grps.push(vec![]);
            if cookie_affinity {
                let parent_path = cgroup.unwrap().path();
                let sub_path = format!("{}/cookie-group-{}", parent_path, idx);
                let cookie_cgroup = create_cgroup(&sub_path).unwrap();
                cgroups.push(cookie_cgroup);
            }
        }
        // The first (main) thread gets a separate cookie, hence the -1 + 1 dance.
        let grp_idx = if idx == 0 {
            0
        } else {
            (idx - 1) % (cookie_group_count - 1) as usize + 1
        };
        pid_grps[grp_idx].push(*pid);

        if cookie_affinity {
            let cookie_cgroup = &cgroups[grp_idx];
            cgroup::set_core_affinity(&cookie_cgroup);
            cgroup::add_task_to_cgroup(&cookie_cgroup, *pid as u64);
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
