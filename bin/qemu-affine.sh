#!/bin/bash

set -x

CPU_COUNT=$1

while :
do
	QEMU_PID=($(pgrep qemu-system))
	if [ -z $QEMU_PID ]; then
		echo "didn't find any qemu process; will retry..."
		sleep 1
		continue;
	fi
	if [ ${#QEMU_PID[@]} != 1 ]; then
		echo "Found more than one qemu process; aborting"
		exit 1
	fi
	echo 1
	echo "QEMU pid=${QEMU_PID[0]}"
	VCPUS_TIDS=($(ps -T -p ${QEMU_PID[0]} -o tid=,comm= | grep CPU | awk '{print $1}'))
	echo "vCPUs tids=(${VCPUS_TIDS})"
	if [ ${#VCPUS_TIDS[@]} == ${CPU_COUNT} ]; then
		break
	fi
	echo "didn't find expected vcpu count; will retry..."
	sleep 1
done

CPU_ID=0
SIBLING_ID=-1
for i in $(seq 0 $((CPU_COUNT-1))); do
	VCPU_TID=$VCPUS_TIDS[i]
	if [ ${SIBLING_ID} != -1 ] && [ ${SIBLING_ID} -ge ${#SIBLINGS[@]} ]; then
		SIBLING_ID=-1
		# dubious logic - will only work as intented if no task has been affined to the next cpu yet ... works on my machine (tm) but should be made more robust
		CPU_ID=$((CPU_ID+1))
	fi
	if [ ${SIBLING_ID} == -1 ]; then
		FILE="/sys/devices/system/cpu/cpu${CPU_ID}/topology/thread_siblings_list"
		SIBLINGS_RAW=$(cat ${FILE})
		IFS=',-' read -ra SIBLINGS <<< $SIBLINGS_RAW
		echo $SIBLINGS
		SIBLING_ID=0
	fi
	VCPU_PCPU_ID=${SIBLINGS[SIBLING_ID]}
	taskset -pc $VCPU_PCPU_ID ${VCPUS_TIDS[i]}
	echo "vCPU $i with pid ${VCPUS_TIDS[i]} affined to pCPU $VCPU_PCPU_ID"
	SIBLING_ID=$((SIBLING_ID+1))
done
