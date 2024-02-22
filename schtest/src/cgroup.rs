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

pub fn add_task_to_cgroup_by_pid(cgroup: &Cgroup, pid: u64) {
    println!("adding pid {} to cgroup", pid);
    let cpus: &cgroups_rs::cpu::CpuController = cgroup.controller_of().unwrap();
    cpus.add_task(&CgroupPid::from(pid))
        .expect("error adding pid to cpu cgroup");
    let cpuset: &CpuSetController = cgroup.controller_of().unwrap();
    cpuset
        .add_task(&CgroupPid::from(pid))
        .expect("error adding pid to cpuset cgroup");
}

pub fn add_task_to_cgroup_by_tgid(cgroup: &Cgroup, tgid: u64) {
    println!("adding tgid {} to cgroup", tgid);
    let cpus: &cgroups_rs::cpu::CpuController = cgroup.controller_of().unwrap();
    cpus.add_task_by_tgid(&CgroupPid::from(tgid))
        .expect("error adding pid to cpu cgroup");
    let cpuset: &CpuSetController = cgroup.controller_of().unwrap();
    cpuset
        .add_task_by_tgid(&CgroupPid::from(tgid))
        .expect("error adding pid to cpuset cgroup");
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
