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
}

fn main() {
    let cli = Args::parse();
    run_command(cli.tasks, cli.cgroup);
}
