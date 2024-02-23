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

For convenience a `run-payload.sh` script is included.
That script executes a given payload in the VM with perf monitoring and all cli arguments propagated.
The payload binary must be copied to the VM (see make-initramfs.sh).
Ths schtest payload binary is included by default.

Example: `qemu/host/run-qemu.sh /home/ubuntu/linux "qemu/vm/run-payload.sh schtest -t 'stress -c 8' -n 8 -W 2 -g mygroup -s '1,2' -w 50 -c 2 -T 10" 4 exit`

More complex use cases (for example launching multiple instances of schtest) may require to use custom scripts.
See for example "qemu/vm/run-many-schtest.sh"

## docker

Schtest provides a containerized dev environment.

The expected workflow is as follows:
* designed use case would be to benchmark the effect of kernel patches using schtest
* user has schtest checked out
* user has linux checked out
* user has docker installed
* the container runs with the local schtest and linux mounted as volumes

### once off setup

Build the container:
`docker build -t schtest .`

Prepare the container:
LOCAL_KERNEL must point to a local checkout of linux. If not available, remove the linux volume as the container comes with a linux checkout anyways.
`docker run -it --rm -v $LOCAL_KERNEL:/linux -v $(pwd):/schtest schtest:latest ./schtest/docker/build.sh -c`

Note the -c argument to generate linux configuration. This can be done once off. The configuration is the default config plus some flags set in 'docker/linux_config'.

### workflow

* If you change the schtest binary application or the the local linux, run the build.sh script again.
* To run a benchmark, run:
`docker run --device=/dev/kvm -it --rm -v $LOCAL_KERNEL:/linux -v $(pwd):/schtest -v /tmp/schtest:/out schtest:latest ./schtest/docker/run.sh qemu/vm/run-many-schtest.sh`

Note that argument propagation to the VM script ('qemu/vm/run-many-schtest.sh' in the example) is not supported. Rather, modify that script as needed.
