#!/bin/bash

set -exo pipefail

OPTARG=""
CONFIGURE=0
while getopts "c" opt; do
  case $opt in
    c) CONFIGURE=1
    ;;
    \?) echo "Invalid option -$OPTARG" >&2
    exit 1
    ;;
  esac

  case $OPTARG in
    -*) echo "Option $opt needs a valid argument"
    exit 1
    ;;
  esac
done

cd /schedulerutils/schedulerutils
cargo build
cd /schedulerutils/get-cpu-wake
cargo build
cd /linux
if [ $CONFIGURE -ne 0 ]; then
	make defconfig
	scripts/kconfig/merge_config.sh .config /schedulerutils/docker/linux_config
fi
make -j $(nproc)
make -C tools/perf
