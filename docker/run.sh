#!/bin/bash

set -euxo pipefail

cd ..
KERNEL_REPO=/linux
SCRIPT="$@"
CPU_COUNT=$(nproc)
MODE=exit
/schtest/qemu/host/run-qemu.sh $KERNEL_REPO $SCRIPT $CPU_COUNT $MODE
