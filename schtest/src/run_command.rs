use std::process::Command;

pub fn run_command(tasks: Vec<String>) {
    for task in tasks {
        let tokens: Vec<&str> = task.split(" ").collect();
        let mut cmd = Command::new(tokens[0]);
        for t in tokens.iter().skip(1) {
            cmd.arg(t);
        }
        let out = cmd.output().expect("failed to execute task");
        println!("task {}: out='{}', err='{}'", task, String::from_utf8(out.stdout).unwrap(), String::from_utf8(out.stderr).unwrap());
    }
}
