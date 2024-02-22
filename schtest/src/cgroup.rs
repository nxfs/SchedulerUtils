use anyhow::{Context, Result};
use cgroups_rs::cpu::CpuController;
use cgroups_rs::cpuset::CpuSetController;
use cgroups_rs::{Cgroup, CgroupPid, Controller};

pub fn create_cgroup(name: &str) -> Option<Cgroup> {
    if name.is_empty() {
        None
    } else {
        println!("creating cgroup {}", name);
        Some(
            Cgroup::new_with_specified_controllers(
                cgroups_rs::hierarchies::auto(),
                String::from(name),
                Some(vec![String::from("cpuset"), String::from("cpu")]),
            )
            .expect("failed to create cgroup"),
        )
    }
}

fn add_task_to_cgroup_by_pid(cgroup: &Cgroup, pid: u64) -> Result<()> {
    let cpus: &cgroups_rs::cpu::CpuController = cgroup.controller_of().unwrap();
    cpus.add_task(&CgroupPid::from(pid))
        .context("could not add pid to cpus cgroup")?;
    let cpuset: &CpuSetController = cgroup.controller_of().unwrap();
    cpuset
        .add_task(&CgroupPid::from(pid))
        .context("cound not add pid to cpuset cgroup")
}

fn add_task_to_cgroup_by_tgid(cgroup: &Cgroup, tgid: u64) -> Result<()> {
    let cpus: &cgroups_rs::cpu::CpuController = cgroup.controller_of().unwrap();
    cpus.add_task_by_tgid(&CgroupPid::from(tgid))
        .context("cound not add tgid to cpus cgroup")?;
    let cpuset: &CpuSetController = cgroup.controller_of().unwrap();
    cpuset
        .add_task_by_tgid(&CgroupPid::from(tgid))
        .context("cound not add tgid to cpuset cgroup")
}

pub fn add_task_to_cgroup(cgroup: &Cgroup, pid: u64) {
    println!("adding {} to cgroup {}", pid, cgroup.path());
    if add_task_to_cgroup_by_tgid(&cgroup, pid).is_err() {
        add_task_to_cgroup_by_pid(cgroup, pid).expect("error adding task to cgroup");
    }
}

pub fn set_cpu_affinity(cgroup: &Cgroup, cpus: &str) {
    println!("setting cgroup {} affinity to {}", cgroup.path(), cpus);
    let cpuset: &CpuSetController = cgroup.controller_of().unwrap();
    cpuset.set_cpus(cpus).expect("unable to set cpus");
}

pub fn set_weight(cgroup: &Cgroup, weight: u64) {
    println!("setting cgroup {} weight to {}", cgroup.path(), weight);
    let cpu: &CpuController = cgroup.controller_of().unwrap();
    cpu.set_shares(weight).expect("unable to set weight");
}
