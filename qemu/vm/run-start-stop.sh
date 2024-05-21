#!/mnt/share/bin/bash

run () {
	W=$1
	Q=$2
	S="${i}_w${W}_q${Q}"
	cp $BASEDIR/stress $BASEDIR/s$S
	cp $BASEDIR/get-cpu-wake $BASEDIR/w$S
	schedulerutils --task "s$S -c $THREAD_COUNT" --threads-count $THREAD_COUNT --timeout-secs $TASK_DURATION --cgroup $S --weight $W --cookie-groups-size 3 --cookie-affinity --cfs-bw-quota $Q &
	schedulerutils --task "w$S -t $THREAD_COUNT -M 5000" --threads-count $THREAD_COUNT --timeout-secs $TASK_DURATION --cgroup $S --weight $W --cookie-groups-size 3 --cookie-affinity --cfs-bw-quota $Q &
}

set -euxo pipefail

DURATION=30
DURATION_MS=$(( 1000 * DURATION ))
BUFFER_DURATION=5
PERF_DURATION=$(( DURATION + BUFFER_DURATION ))
TASK_DURATION=8

THREAD_COUNT=4
PROCESS_PER_LOOP=8
OVERSUB_LEVEL=8

NPROC=$(nproc)
SLEEP_DURATION=$((TASK_DURATION * PROCESS_PER_LOOP * THREAD_COUNT / (NPROC * OVERSUB_LEVEL)))

LOOPS=$(( DURATION / SLEEP_DURATION ))

BASEDIR=/mnt/share/bin

perf sched record -D 0-$DURATION_MS -- sleep $PERF_DURATION && echo "perf done" &

for i in $(seq $LOOPS)
do
	# W=$((10 + $RANDOM % 90))
	# Q=$((10 + $RANDOM % 90))
	run 100 100
	run 100 80
	run 80 100
	run 80 80
	sleep $SLEEP_DURATION
done

sleep $BUFFER_DURATION
