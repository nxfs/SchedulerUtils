mod cgroup;
mod run_command;

use clap::Parser;
use run_command::run_command;


#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short = 't', long = "tasks", num_args = 1.., default_value = "echo 'hello schtest'")]
    pub tasks: Vec<String>,

    #[arg(short = 'g', long = "cgroup", default_value = "")]
    pub cgroup: String,

    #[arg(short = 's', long = "cpuset", default_value = "")]
    pub cpuset: String,
}

fn main() {
    let cli = Args::parse();
    if cli.cgroup.is_empty() && !cli.cpuset.is_empty() {
        panic!("cannot specify cpuset without cgroups");
    }
    run_command(cli.tasks, cli.cgroup, cli.cpuset);
}
