# schtest

## initial goal

Test benchmark to assess the correctness of the core affinity scheduler patches to support upstreaming process.

### overlaps

The correctness of the core scheduler is defined by its ability to eliminate overlaps.
An overlap is when a process with a cookie is scheduled on the CPU sibling of a process with a different or no cookie.

## future goals

Test benchmark to assess the correctness and performance of the scheduler.


## tools

### perf-schtest.sh

Entry point. An end-to-end overlap instrumentation script of schtest via perf.

The script launches a set of processes using schtest and monitors scheduler events with perf sched.
Schtest will launches processes, optionally affine them to a set of cpus and assign them cookies.
Prior to calling schtest, run.sh will create the necessary cgroups.

The script then executes a perf script which computes for each CPU a timeline of running cookies and checks for overlaps between siblings.
The summary of found overlaps is printed while a fully detailed report is dumped in a result file.

All script options are passed through to `./bin/schtest`; refer to that script for available options.
The options -d (result directory) and -g (cgroup) are injected by run.sh and must not be supplied.

Example usage:
`./bin/run.sh -t "bin/stress -d 10" -n 16 -c 8 -s "0,48"`

* mount cgroups
* launches schtest with the following config
 * 16 stressors processes running for 10 seconds
 * 8 cookies
 * 2 cpus (0 and 48)
* trace schtest with perf
* post process collected data and assert correctness of core scheduling (no overlaps)


### schtest

Tool to launches tasks, and optionally assign them a cpuset and various cookies.
Run it without arguments to learn how to use it.


### stress

Single threaded tool to stress cpu using a task rate arrival model.
Units of work are created at periodic wall clock time intervals.



## qemu


### perf-schtest-qemu.sh

Equivalent functionality than perf-schtest.sh but runs in QEMU.

Example:
`./bin/perf-schtest-qemu.sh /home/ubuntu/linux-stable "perf-schtest.sh -t 'stress -d 10' -n 8 -c 4 -d 4" 8`

* 1st argument: local kernel path, must contain a bzImage and a perf executable; these will be executed in the VM
* 2nd argument: command line to run the test, see 'perf-schtest' doc above
* 3rd argument: vCPU count
* 4th argument (optional): behavior after test. Supported:
  * shell (default): will drop into the VM shell
  * exit: will poweroff the VM and return into userspace

While the tests are ran in QEMU, the post analysis via perf script is ran in our userspace (because otherwise we'd need to ship a python runtime); this introduces some compatibility constraints between the VM and the current OS.

The QEMU vCPUs will be pinned to pCPUs using the following algorithm:
* 1st vcpu pinned to cpu0
* 2nd vcpu pinned to sibling 1 of cpu0
* ith vcpu with sibling i of cpu0
* Once running out of siblings, (i+1)th vcpu pinned to cpu1
* etc.

## Kernel config requirements

* In QEMU to share guest/host folder via 9p virtio: https://wiki.qemu.org/Documentation/9psetup
* Perf eBPF: CONFIG_BPF_SYSCALL=y
* Core scheduler: CONFIG_SCHED_CORE=y
* Remove verbose debug messages: CONFIG_DEBUG_STACK_USAGE=n
* /proc/pid/sched: CONFIG_SCHED_DEBUG=y
