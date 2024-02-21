use nix::libc::{pid_t, prctl};
use std::{io::Error, thread};

#[allow(non_camel_case_types)]
#[allow(dead_code)]
enum SchedCorePrctlOption {
    PR_SCHED_CORE_GET = 0,
    PR_SCHED_CORE_CREATE = 1,         /* create unique core_sched cookie */
    PR_SCHED_CORE_SHARE_TO = 2,       /* push core_sched cookie to pid */
    PR_SCHED_CORE_SHARE_FROM = 3,     /* pull core_sched cookie to pid */
    PR_SCHED_CORE_MAX = 4,
}

#[allow(non_camel_case_types)]
#[allow(dead_code)]
enum PIDType {
    PIDTYPE_PID,
    PIDTYPE_TGID,
    PIDTYPE_PGID,
    PIDTYPE_SID,
    PIDTYPE_MAX,
}

/* Request the scheduler to share a core */
const PR_SCHED_CORE: i32 = 62;
const PIDTYPE_PID: i32 = PIDType::PIDTYPE_PID as i32;

#[allow(dead_code)]
pub fn get_cookie(pid: pid_t) -> u64 {
    let cookie: u64 = 0;
    let option = SchedCorePrctlOption::PR_SCHED_CORE_GET as i32;
    let result = unsafe { prctl(PR_SCHED_CORE, option, pid, PIDTYPE_PID, &cookie) };
    if result != 0 {
        panic!("could not get cookie for {}", pid);
    }
    cookie
}

pub fn create_cookie(pid: pid_t) {
    println!("creating cookie for {}", palaver::thread::gettid());
    let option = SchedCorePrctlOption::PR_SCHED_CORE_CREATE as i32;
    let result = unsafe { prctl(PR_SCHED_CORE, option, pid, PIDTYPE_PID, 0) };
    if result != 0 {
        panic!(
            "could not create cookie for {}: {}",
            pid,
            Error::last_os_error()
        );
    }
}

pub fn share_cookie_to(pid: pid_t) {
    println!("{} sharing cookie to {}", palaver::thread::gettid(), pid);
    let option = SchedCorePrctlOption::PR_SCHED_CORE_SHARE_TO as i32;
    let result = unsafe { prctl(PR_SCHED_CORE, option, pid, PIDTYPE_PID, 0) };
    if result != 0 {
        panic!(
            "could not share cookie to {}: {}",
            pid,
            Error::last_os_error()
        );
    }
}

pub fn create_cookie_grp(pids: Vec<pid_t>) {
    create_cookie(palaver::thread::gettid() as pid_t);
    for pid in pids {
        share_cookie_to(pid);
    }
}

pub fn create_cookies(pid_grps: Vec<Vec<pid_t>>) {
    // we need to do this async cause the cookie api only has share from and share to against self
    // so we create thread to do it while avoiding to mutate the current process
    let threads: Vec<_> = pid_grps.into_iter()
        .map(|pid_grp| {
            thread::spawn(move || {
                create_cookie_grp(pid_grp.to_vec());
            })
        })
        .collect();

    for handle in threads {
        handle.join().unwrap();
    }
}
