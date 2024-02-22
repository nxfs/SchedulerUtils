## Tools

### schtest

Launches a task and configures its cgroup tunables and cookies

#### build

`cd schtest && cargo build`

#### usage

`cargo run -- --help`

Example:

`schtest -t 'stress -c 8' -n 8 -W 2 -g mygroup -s '1,2' -w 50 -c 2 -T 10`

* `-t 'stress -c 8'`: launch the task 'stress -c 8' (imposes load on 8 threads)
* `-n 8 -W 2`: wait for 8 threads to be spawned (excluding the main task thread) with a timeout of 2 seconds
* `-g mygroup` create a cgroup named 'mygroup', move all spawned threads in that cgroup
* `-s '1,2'` affine the cgroup to the cpuset '1,2'
* `-w 50` assign a cpu weight of 50 to that cgroup
* `-c 2` create two core cookies and assign them to the spawned threads, round robin
* `-T 10` kill all spawned threads after 10 seconds

### qemu/host/run-qemu.sh

Runs a qEMU virtual machine with a simple busybox based userspace.
Designed to run stress workloads in qEMU, instrument them with perf and expose the results to the host.

#### usage

`qemu/host/run-qemu.sh $KERNEL_REPO $SCRIPT $CPU_COUNT $MODE`

Runs qemu with:
* a virtual machine composed of CPU_COUNT cpus and 16G of memory
* a hyperthread topology mirroring the host's if the host has 2 HTs per core (see affine-qemu.sh)
* the kernel x86 image found in KERNEL_REPO
* the perf binary found in KERNEL_REPO
* a generated initramfs that includes a bunch of host binaries and executes the given SCRIPT in busybox
* a virtfs to share files with the host and mounted as /mnt/share
* results written by the script to /mnt/share/out are copied in the local directory
* drops in a shell in qemu or poweroff by specifiyng 'exit' as MODE

#### working with schtest

For convenience a `run-schtest.sh` script is included.
That script executes `schtest` in the VM with perf monitoring and all cli arguments propagated.
More complex use cases (for example launching multiple instances of schtest) require using custom scripts.

Example: `qemu/host/run-qemu.sh /home/ubuntu/linux "qemu/vm/run-schtest.sh schtest -t 'stress -c 8' -n 8 -W 2 -g mygroup -s '1,2' -w 50 -c 2 -T 10" 4 exit
