#!/bin/bash
# pins qEMU vCPUs to specific pCPUs
# after this, virtual hyperthread topology mirrors physical hyperthread topology, as long as guest hyperthread siblings == host hyperthread siblings 

set -x

CPU_COUNT=$1

# first collect all vCPUs tids
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

# then iterate through vCPUs and affine them to pCPUs
# the first vCPUs are assigned to the pCPUs corresponding to the first thread siblings list
# once the list is exhausted we move on to the next list
# overall logic is quite brittle and should be improved on
CPU_ID=0
SIBLING_ID=-1
for i in $(seq 0 $((CPU_COUNT-1))); do
        VCPU_TID=$VCPUS_TIDS[i]
        if [ ${SIBLING_ID} != -1 ] && [ ${SIBLING_ID} -ge ${#SIBLINGS[@]} ]; then
                SIBLING_ID=-1
                # dubious logic - will only work as intented if no task has been affined to the next cpu yet
		# works on my machine (tm) but should be made more robust
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
