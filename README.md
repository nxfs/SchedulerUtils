## Tools

### schedulerutils

Launches a task and configures its cgroup tunables and cookies

#### build

`cd schedulerutils && cargo build`

#### usage

`cargo run -- --help`

Example:

`schedulerutils -t 'stress -c 8' -n 8 -W 2 -g mygroup -s '0,1' -w 50 -c 2 -T 10 -p 80000 -q 50 -a`

* `-t 'stress -c 8'`: launch the task 'stress -c 8' (imposes load on 8 threads)
* `-n 8 -W 2`: wait for 8 threads to be spawned (excluding the main task thread) with a timeout of 2 seconds
* `-g mygroup` create a cgroup named 'mygroup', move all spawned threads in that cgroup
* `-s '0,1'` affine the cgroup to the cpuset '0,1'
* `-w 50` assign a cpu weight of 50 to that cgroup
* `-c 2` create two core cookies and assign them to the spawned threads, round robin
* `-T 10` kill all spawned threads after 10 seconds
* `-p 80000 -q 50` set cpu bandwidth period to 80000 us and quota to 50% (cpu util of all threads)
* `-a` sets cookie affinity. This relies on kernel functionality that is not merged upstream: https://github.com/nxfs/linux/tree/v6.8-rc7-affine

### get-cpu-wake

Stresses the task CPU wake path

#### build

`cd get-cpu-wake && cargo build`

#### usage

`cargo run -- --help`

Example:

`get-cpu-wake -t 4 -s 2 -m 100 -M 1000`

* Launches 4 tasks
* Run busy loops, for each loop perform a work unit of weight between 100 and 1000 (work is not linear)
* Sleep 2ms between each work unit
* The average time between loop will be output to stdout. Use it to calibrate work units.

### qemu/host/run-qemu.sh

Runs a qEMU virtual machine with a simple busybox based userspace.
Designed to run stress workloads in qEMU, instrument them with perf and expose the results to the host.

#### usage

`qemu/host/run-qemu.sh -k $KERNEL_REPO -s $SCRIPT -c $CPU_COUNT -m $MEMORY -e`

Runs qemu with:
* a virtual machine composed of CPU_COUNT cpus and 16G of memory
* a hyperthread topology mirroring the host's if the host has 2 HTs per core (see affine-qemu.sh)
* the kernel x86 image found in KERNEL_REPO
* the perf binary found in KERNEL_REPO
* a generated initramfs that includes a bunch of host binaries and executes the given SCRIPT in busybox
* a virtfs to share files with the host and mounted as /mnt/share
* results written by the script to /mnt/share/out are copied in the local directory
* doesn't drop in a shell in qemu or poweroff by specifiyng '-e' (exit mode)

#### working with schedulerutils

For convenience a `run-payload.sh` script is included.
That script executes a given payload in the VM with perf monitoring and all cli arguments propagated.
The payload binary must be copied to the VM (see make-initramfs.sh).
Ths schedulerutils payload binary is included by default.

Example: `qemu/host/run-qemu.sh -k /home/ubuntu/linux -s "qemu/vm/run-payload.sh schedulerutils -t 'stress -c 8' -n 8 -W 2 -g mygroup -s '1,2' -w 50 -c 2 -T 10" -c 4 -m 16G -e`

More complex use cases (for example launching multiple instances of schedulerutils) may require to use custom scripts.
See for example "qemu/vm/run-many-schedulerutils.sh"

## docker

schedulerutils provides a containerized dev environment.

The expected workflow is as follows:
* designed use case would be to benchmark the effect of kernel patches using schedulerutils
* user has schedulerutils checked out
* user has linux checked out
* user has docker installed
* the container runs with the local schedulerutils and linux mounted as volumes

### once off setup

Build the container:
`docker build -t schedulerutils .`

Prepare the container:
LOCAL_KERNEL must point to a local checkout of linux. If not available, remove the linux volume as the container comes with a linux checkout anyways.
`docker run -it --rm -v $LOCAL_KERNEL:/linux -v $(pwd):/schedulerutils schedulerutils:latest ./schedulerutils/docker/build.sh -c`

Note the -c argument to generate linux configuration. This can be done once off. The configuration is the default config plus some flags set in 'docker/linux_config'.

### workflow

* If you change the schedulerutils binary application or the the local linux, run the build.sh script again.
* To run a benchmark, run:
`docker run --device=/dev/kvm -it --rm -v $LOCAL_KERNEL:/linux -v $(pwd):/schedulerutils -v /tmp/schedulerutils:/out schedulerutils:latest ./schedulerutils/docker/run.sh qemu/vm/run-many-schedulerutils.sh`

Note that argument propagation to the VM script ('qemu/vm/run-many-schedulerutils.sh' in the example) is not supported. Rather, modify that script as needed.

## Authors

* Xiaoyi Chen
* Fernand Sieber
