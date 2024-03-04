FROM ubuntu:latest AS linuxbuilder

RUN apt-get update

#fetch linux from git if not provided by a volume
RUN apt-get install -y git
RUN git clone --depth=1 https://github.com/torvalds/linux

# install linux building tools
RUN apt-get install -y apt-utils
RUN apt-get install -y libncurses-dev gawk flex bison openssl libssl-dev dkms libelf-dev libudev-dev libpci-dev libiberty-dev autoconf llvm bc pkg-config

# install perf building tools
RUN apt-get install -y libzstd1 libdwarf-dev libdw-dev binutils-dev libcap-dev libelf-dev libnuma-dev python3 python3-dev python-setuptools libssl-dev libunwind-dev libdwarf-dev zlib1g-dev liblzma-dev libaio-dev libtraceevent-dev debuginfod libpfm4-dev libslang2-dev systemtap-sdt-dev libperl-dev binutils-dev libbabeltrace-dev libiberty-dev libzstd-dev clang python3-setuptools default-jdk
RUN git clone --depth=1 https://github.com/rostedt/libtraceevent
WORKDIR /libtraceevent
RUN make && make install
WORKDIR /


# install qemu
RUN apt-get install -y qemu-system
RUN apt-get install -y busybox stress cpio

# build statically configured busybox
RUN git clone --depth 1 https://github.com/mirror/busybox
WORKDIR /busybox
RUN make defconfig
RUN make clean && make LDFLAGS=-static -j $(nproc)
RUN mv busybox $(which busybox)
WORKDIR /

# install rust build tools
RUN apt-get install -y curl
RUN curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain nightly -y
ENV PATH="/root/.cargo/bin:${PATH}"

# copy schedulerutils in container
RUN mkdir /schedulerutils
COPY . /schedulerutils

# initial build
WORKDIR /schedulerutils/docker
RUN ./build.sh -c && echo "built"
WORKDIR /

# utils
RUN apt-get install neovim -y
