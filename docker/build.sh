#!/bin/bash

set -euxo pipefail

cd /schtest/schtest
cargo build
cd /linux
if make -j $(nproc) ; then
	echo "build succeeded, not touching the kernel config"
else
	make defconfig
	scripts/kconfig/merge_config.sh .config /schtest/docker/linux_config
	make -j $(nproc)
fi
make -C tools/perf
