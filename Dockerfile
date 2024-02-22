# First lets build schtest
# LABEL schtest
FROM rust:slim AS rust

RUN mkdir /schtest
COPY schtest /schtest

RUN cd /schtest && cargo build --release

# Second, set up the qemu host
FROM debian:stable

RUN apt-get update
RUN apt-get install -y qemu-system 
RUN apt-get install -y busybox stress cpio
# RUN apt-get install -y linux-perf

RUN mkdir /schtest

COPY --from=rust /schtest/target/release/schtest /schtest/schtest
COPY qemu /schtest/.




# ENTRYPOINT 