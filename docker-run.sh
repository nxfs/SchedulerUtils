set -euxo pipefail

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

# set -euxo pipefail

docker build . -t schtest

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
SCRIPT_NAME="${SCRIPT%% -*}"
SCRIPT_CMD=$(basename $SCRIPT_NAME)
SCRIPT_ARGS=$(echo "$SCRIPT" | sed 's/^[^ ]* //')
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



docker run -it --mount type=bind,source=$KERNEL_REPO,target=/kernel_repo \
  --mount type=bind,source=$PWD/out,target=/out \
  --device=/dev/kvm \
  schtest \
  /schtest/host/run-qemu.sh /kernel_repo "$2" 2 exit

