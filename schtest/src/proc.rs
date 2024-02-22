use procfs::process::Process;
use std::{
    fs, thread,
    time::{Duration, Instant},
};

pub fn get_all_children_processes_ids(pid: i32) -> Vec<i32> {
    let mut pids = vec![pid];
    let mut idx = 0;
    while idx < pids.len() {
        pids.append(&mut get_child_processes_ids(pids[idx]));
        idx += 1;
    }
    pids
}

pub fn get_child_processes_ids(pid: i32) -> Vec<i32> {
    let path = format!("/proc/{}/task/{}/children", pid, pid);
    let content = fs::read_to_string(path).unwrap();

    let children: Vec<i32> = content
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();

    children
}

pub fn get_thread_ids(pid: i32) -> Vec<i32> {
    let process = Process::new(pid).unwrap();
    let mut thread_ids = Vec::new();

    for task_result in process.tasks().unwrap() {
        let task = task_result.unwrap();
        thread_ids.push(task.tid);
    }

    thread_ids
}

pub fn wait_for_threads(pid: i32, expected_thread_count: usize, timeout: Duration) -> Vec<i32> {
    println!("waiting for all threads to appear");
    if pid == 0 {
        return vec![];
    }
    let start_time = Instant::now();
    loop {
        let child_pids = get_all_children_processes_ids(pid);
        let mut pids: Vec<i32> = vec![];
        for child_pid in child_pids {
            pids.append(&mut get_thread_ids(child_pid));
        }
        // tgid is excluded, so we need one more thread
        if pids.len() > expected_thread_count {
            println!("detected all threads, pids={:?}", pids);
            return pids;
        }
        if Instant::now().duration_since(start_time) > timeout {
            panic!(
                "timeout waiting for threads, only {} found, expected {}",
                pids.len(),
                expected_thread_count
            );
        }
        thread::sleep(Duration::from_millis(100));
    }
}
