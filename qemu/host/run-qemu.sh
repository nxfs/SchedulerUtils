#!/bin/bash
# * runs qemu with:
#   * a virtual machine composed of CPU_COUNT cpus and 16G of memory
#   * a hyperthread topology mirroring the host's if the host has 2 HTs per core (see affine-qemu.sh)
#   * the kernel x86 image found in KERNEL_REPO
#   * the perf binary found in KERNEL_REPO
#   * a generated initramfs that executes the given SCRIPT in busybox
#   * a virtfs to share files with the host and mounted as /mnt/share
#   * results written by the script to /mnt/share/out are copied in the local directory
#   * drops in a shell in qemu or poweroff by specifiyng 'exit' as EXIT_MODE

# args
# -k: kernel repo
# -s: script
# -c: cpu count
# -m: memory
# -e: exit mode (don't drop into shell)

set -euxo pipefail

OPTARG=""
CPU_COUNT=$(nproc)
MEMORY=16G
EXIT_MODE="shell"
while getopts "k:s:c:m:e" opt; do
	case ${opt} in
		k) KERNEL_REPO=${OPTARG}
			;;
		s) SCRIPT=${OPTARG}
			;;
		c) CPU_COUNT=${OPTARG}
			;;
		m) MEMORY=${OPTARG}
			;;
		e) EXIT_MODE="exit"
			;;
		:)
			echo "Option -${OPTARG} requires an argument."
			exit 1
			;;
		?)
			echo "Invalid option: -${OPTARG}."
			exit 1
			;;
	esac
done

KERNEL_IMAGE=$KERNEL_REPO/arch/x86/boot/bzImage
PERF=$KERNEL_REPO/tools/perf/perf
INITRAMFS=initramfs.cpio.gz

SHARED_DIR=/tmp/share
mkdir -p $SHARED_DIR/bin
rm -f $INITRAMFS
USER_SCRIPT=$SHARED_DIR/bin/userscript
rm -rf $USER_SCRIPT
echo "#!bin/bash" >> $USER_SCRIPT
SCRIPT_NAME="${SCRIPT%% *}"
SCRIPT_CMD=$(basename $SCRIPT_NAME)
PATTERN="[[:space:]]+"
if [[ "$SCRIPT" =~ $PATTERN ]]; then
	SCRIPT_ARGS=$(echo "$SCRIPT" | sed 's/^[^ ]* //')
else
	echo "no args"
	SCRIPT_ARGS=""
fi
echo "$SCRIPT_CMD $SCRIPT_ARGS" >> $USER_SCRIPT
chmod u+x $USER_SCRIPT

MAIN_SCRIPT=$SHARED_DIR/bin/mainscript
cat > ${MAIN_SCRIPT} << EOF
#!/bin/busybox sh

set -euxo pipefail
rm -rf /mnt/share/out
mkdir -p /mnt/share/out
chmod +x /mnt/share/bin/userscript
bash /mnt/share/bin/userscript || echo "run.sh failed"

grep -E "processor|core id" /proc/cpuinfo > /mnt/share/out/topo.txt
mv perf.data /mnt/share/out
EOF
chmod u+x $MAIN_SCRIPT

case $EXIT_MODE in
        exit)
                echo "poweroff -f" >> $MAIN_SCRIPT
                ;;
        *)
                echo "/bin/busybox sh" >> $MAIN_SCRIPT
                ;;
esac

DIR=$(dirname ${BASH_SOURCE[0]})
$DIR/make-initramfs.sh -i$INITRAMFS -d$SHARED_DIR -p$PERF -s$SCRIPT_NAME
$DIR/affine-qemu.sh $CPU_COUNT > /dev/null 2>&1 &
QEMU_AFFINE_PID=$!
qemu-system-x86_64 \
	-enable-kvm \
	-cpu host,+vmx \
	-smp $CPU_COUNT,threads=2 \
	-m $MEMORY \
	-name test,debug-threads=on \
	-kernel $KERNEL_IMAGE \
	-initrd $INITRAMFS \
	-display none \
	-append "earlyprintk=serial console=ttyS0 sched_verbose" \
	-serial mon:stdio \
	-virtfs local,path=$SHARED_DIR,mount_tag=host_share,security_model=none
wait $QEMU_AFFINE_PID || { echo "qemu-affine.sh failed; aborting"; exit 1; }
rm -f $INITRAMFS
cp -a $SHARED_DIR/out/. ./
