mod cgroup;
mod prctl;
mod proc;
mod run_command;

use std::time::Duration;

use clap::Parser;
use run_command::{run_command, RunCommandCfg};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// task to run
    #[arg(short = 't', long = "task", default_value = "sleep 10")]
    pub task: String,

    /// number of threads that the task to run will spawn, excluding the task group leader
    /// the program execution will block until all threads are detected
    /// note that if the task spawns other processes, these will also be accounted for, recursively
    #[arg(
        short = 'n',
        long = "threads-count",
        default_value = "0",
        verbatim_doc_comment
    )]
    pub threads: usize,

    /// the maximum wait time for the task to spawn all its threads
    #[arg(short = 'W', long = "threads-wait-secs", default_value = "1")]
    pub threads_wait_secs: u64,

    /// the maximum duration of the test, after which the task will be killed
    #[arg(short = 'T', long = "timeout-secs", default_value = "0")]
    pub timeout_secs: u64,

    /// the cgroup name into which the task and its threads will be put
    #[arg(short = 'g', long = "cgroup", default_value = "")]
    pub cgroup: String,

    /// the cpu affinity of the cgroup
    #[arg(short = 's', long = "cpuset", default_value = "")]
    pub cpuset: String,

    /// the cpu weight of the cgroup
    #[arg(short = 'w', long = "weight", default_value = "100")]
    pub weight: u64,

    /// the cfs bandwidth period of the cgroup, in usecs
    #[arg(short = 'p', long = "cfs-bw-period-usecs", default_value = "80000")]
    pub cfs_bw_period_us: u64,

    /// the cpu quota of the cgroup, in percentage
    /// will be translated into quota_us using the following formula
    /// quota = (val / 100) * period * thread_count
    /// if zero, no cfs bandwith configuration will be applied
    #[arg(
        short = 'q',
        long = "cfs-bw-quota",
        default_value = "0",
        verbatim_doc_comment
    )]
    pub cfs_bw_quota_pc: u64,

    /// the size, in number of threads, of each cookie group
    /// if zero, no cookies are used
    /// if one, each threads has an unique cookie
    /// if more than one, thread_count / c groups of size c are created, and tasks are added to groups in order
    /// cookie
    #[arg(
        short = 'c',
        long = "cookie-groups-size",
        default_value = "0",
        verbatim_doc_comment
    )]
    pub cookie_group_size: u64,

    /// whether to activate cookie affinity
    /// this is a kernel functionality that is not merged upstream
    /// see: https://github.com/nxfs/linux/tree/v6.8-rc7-affine
    #[arg(short = 'a', long, verbatim_doc_comment)]
    pub cookie_affinity: bool,
}

impl From<Args> for RunCommandCfg {
    fn from(args: Args) -> RunCommandCfg {
        RunCommandCfg {
            task: args.task,
            threads: args.threads,
            thread_wait: Duration::from_secs(args.threads_wait_secs),
            timeout: Duration::from_secs(args.timeout_secs),
            cgroup: args.cgroup,
            cpuset: args.cpuset,
            weight: args.weight,
            cookie_group_size: args.cookie_group_size,
            cfs_bw_period_us: args.cfs_bw_period_us,
            cfs_bw_quota_pc: args.cfs_bw_quota_pc,
            cookie_affinity: args.cookie_affinity,
        }
    }
}

fn main() {
    let cli = Args::parse();
    if cli.cgroup.is_empty() {
        if !cli.cpuset.is_empty() {
            panic!("cannot specify cpuset without cgroups");
        }
        if cli.weight != 100 {
            panic!("cannot specify weight without cgroups");
        }
        if cli.cfs_bw_quota_pc > 0 {
            panic!("cannot specify quota without cgroups");
        }
        if cli.cookie_affinity {
            panic!("cannot specify cookie affinity without cgroups");
        }
    }
    if cli.cookie_affinity && cli.cookie_group_size == 0 {
        panic!("cannot specify cookie affinity with a zero cookie group size");
    }
    run_command(cli.into())
}
