#!/bin/bash
# * runs qemu with:
#   * a virtual machine composed of CPU_COUNT cpus and 16G of memory
#   * a hyperthread topology mirroring the host's if the host has 2 HTs per core (see affine-qemu.sh)
#   * the kernel x86 image found in KERNEL_REPO
#   * the perf binary found in KERNEL_REPO
#   * a generated initramfs that executes the given SCRIPT in busybox
#   * a virtfs to share files with the host and mounted as /mnt/share
#   * results written by the script to /mnt/share/out are copied in the local directory
#   * drops in a shell in qemu or poweroff by specifiyng 'exit' as MODE

set -euxo pipefail

KERNEL_REPO=$1
SCRIPT=$2
CPU_COUNT=$3
MODE=$4

MEMORY=16G

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
bash /mnt/share/bin/userscript || echo "run.sh failed"

mv perf.data /mnt/share/out
EOF
chmod u+x $MAIN_SCRIPT

case $MODE in
        exit)
                echo "poweroff -f" >> $MAIN_SCRIPT
                ;;
        *)
                echo "/bin/busybox sh" >> $MAIN_SCRIPT
                ;;
esac

DIR=$(dirname ${BASH_SOURCE[0]})
$DIR/make-initramfs.sh $INITRAMFS $SHARED_DIR $PERF $SCRIPT_NAME
$DIR/affine-qemu.sh $CPU_COUNT > /dev/null 2>&1 &
QEMU_AFFINE_PID=$!
qemu-system-x86_64 \
        -nographic \
        -enable-kvm \
        -cpu host,+vmx \
        -smp $CPU_COUNT,threads=2 \
        -m $MEMORY \
        -name test,debug-threads=on \
        -kernel $KERNEL_IMAGE \
        -initrd $INITRAMFS \
        -append "console=ttyS0" \
        -serial mon:stdio \
        -virtfs local,path=$SHARED_DIR,mount_tag=host_share,security_model=none
wait $QEMU_AFFINE_PID || { echo "qemu-affine.sh failed; aborting"; exit 1; }
rm -f $INITRAMFS
cp -a $SHARED_DIR/out/. ./
