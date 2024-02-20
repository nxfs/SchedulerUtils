mod cgroup;
mod proc;
mod run_command;

use std::time::Duration;

use clap::Parser;
use run_command::{run_command, RunCommandCfg};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 't', long = "task", default_value = "echo 'hello schtest'")]
    pub task: String,

    #[arg(short = 'n', long = "threads-count", default_value = "0")]
    pub threads: usize,

    #[arg(short = 'w', long = "threads-wait-secs", default_value = "1")]
    pub threads_wait_secs: u64,

    #[arg(short = 'g', long = "cgroup", default_value = "")]
    pub cgroup: String,

    #[arg(short = 's', long = "cpuset", default_value = "")]
    pub cpuset: String,
}

impl From<Args> for RunCommandCfg {
    fn from(args: Args) -> RunCommandCfg {
        RunCommandCfg {
            task: args.task,
            threads: args.threads,
            thread_wait: Duration::from_secs(args.threads_wait_secs),
            cgroup: args.cgroup,
            cpuset: args.cpuset
        }
    }
}

fn main() {
    let cli = Args::parse();
    if cli.cgroup.is_empty() && !cli.cpuset.is_empty() {
        panic!("cannot specify cpuset without cgroups");
    }
    run_command(cli.into())
}
