use cgroups_rs::{Cgroup, CgroupPid, Controller};
use cgroups_rs::cpuset::CpuSetController;

pub fn create_cgroup(name: &str) -> Option<Cgroup> {
    if name.is_empty() {
        None
    } else {
        println!("creating cgroup {}", name);
        Some(Cgroup::new_with_specified_controllers(
            cgroups_rs::hierarchies::auto(),
            String::from(name),
            Some(vec![String::from("cpuset"), String::from("cpu")]),
        ).expect("failed to create cgroup"))
    }
}

pub fn add_task_to_cgroup_by_tgid(cgroup: &Cgroup, tgid: u64) {
    println!("adding {} to cgroup", tgid);
    let cpus: &cgroups_rs::cpu::CpuController = cgroup.controller_of().unwrap();
    cpus.add_task_by_tgid(&CgroupPid::from(tgid)).expect("error adding pid to cpu cgroup");
    let cpuset: &CpuSetController = cgroup.controller_of().unwrap();
    cpuset.add_task_by_tgid(&CgroupPid::from(tgid)).expect("error adding pid to cpuset cgroup");
}

pub fn set_cpu_affinity(cgroup: &Cgroup, cpus: &str) {
    let cpuset: &CpuSetController = cgroup.controller_of().unwrap();
    cpuset.set_cpus(cpus).expect("unable to set cpus");
}
