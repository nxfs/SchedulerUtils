use std::process::{Child, Command};

use crate::cgroup;

pub fn run_command(tasks: Vec<String>, cgroup: String) {
    let maybe_cgroup = cgroup::create_cgroup(&cgroup);
    let mut _handles: Vec<Child> = vec![];
    for task in tasks {
        println!("spawning task \"{}\"", task);
        let tokens: Vec<&str> = task.split(" ").collect();
        let mut cmd = Command::new(tokens[0]);
        for t in tokens.iter().skip(1) {
            cmd.arg(t);
        }
        let handle = cmd.spawn().unwrap();
        if let Some(ref cgroup) = maybe_cgroup {
            cgroup::add_task_to_cgroup(&cgroup, handle.id() as u64);
        }
        _handles.push(handle);
        let out = cmd.output().expect("failed to execute task");
        println!("task {}: out='{}', err='{}'", task, String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap());
    }
}
