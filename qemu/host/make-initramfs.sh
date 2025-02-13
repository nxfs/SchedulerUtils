#!/bin/bash
# Create a busybox based initramfs.

set -exo pipefail

# copy executable to host/guest shared dir and copy all dynamic libraries
install_exec() {
	f=$1
	if [ ! -f $f ]; then
		f=$(which $f)
	fi
	mkdir -p $2/bin
	cp $f $2/bin
	for lib in $(ldd $f | cut -d '(' -f 1 | cut -d '>' -f 2 | grep -v vdso); do
		LIBDIR=$(dirname $lib)
		mkdir -p $3/${LIBDIR}
		if ! [ -f $3/$lib ]; then
			cp $lib $3/$lib
		fi
	done
}

OPTARG=""
while getopts "i:d:p:s:" opt; do
	case ${opt} in
		i) INITRAMFS=${OPTARG}
			;;
		d) SHARED_DIR=${OPTARG}
			;;
		p) PERF=${OPTARG}
			;;
		s) SCRIPT_NAME=${OPTARG}
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

BUSYBOX=$(which busybox)
INITRAMFS_DIR=initramfs

# Create a directoy tree for the initramfs. Make sure we start from a
# fresh state.

rm -rf ${INITRAMFS_DIR}
mkdir -p ${INITRAMFS_DIR}/{bin,sbin,etc,proc,sys,dev}
touch ${INITRAMFS_DIR}/etc/mdev.conf

DIR=$(dirname ${BASH_SOURCE[0]})
install_exec ${DIR}/../../schedulerutils/target/debug/schedulerutils ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec ${DIR}/../../get-cpu-wake/target/debug/get-cpu-wake ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec ${DIR}/../../${SCRIPT_NAME} ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec awk ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec bash ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec cat ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec chmod ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec cp ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec dmesg ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec dirname ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec grep ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec ls ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec mkdir ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec mv ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec mount ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec nproc ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec pkill ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec poweroff ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec rm ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec seq ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec sleep ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec stress ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec stress-ng ${SHARED_DIR} ${INITRAMFS_DIR}
install_exec $PERF ${SHARED_DIR} ${INITRAMFS_DIR}

# Copy busybox into the right place.
if ldd ${BUSYBOX} &> /dev/null; then
    echo "busybox must be statically compiled"
    exit 1
fi
cp -a ${BUSYBOX} ${INITRAMFS_DIR}/bin/busybox

# Create the init script
cp ${DIR}/../vm/init ${INITRAMFS_DIR}/init
chmod +x ${INITRAMFS_DIR}/init

# Create the cpio archive and cleanup
cd ${INITRAMFS_DIR}
find . | cpio -o -H newc | gzip > ../$INITRAMFS
cd -
rm -rf ${INITRAMFS_DIR}
