#!/mnt/share/bin/bash

run () {
	local W=$1
	local Q=$2
	S="${i}_w${W}_q${Q}"
	cp $BASEDIR/stress $BASEDIR/s$S
	cp $BASEDIR/get-cpu-wake $BASEDIR/w$S
	schedulerutils --task "s$S -c $THREAD_COUNT" --threads-count $THREAD_COUNT --timeout-secs $TASK_DURATION --cgroup s$S --weight $W --cookie-groups-size 3 --cookie-affinity --cfs-bw-quota $Q &
	schedulerutils --task "w$S -t $THREAD_COUNT -M 5000" --threads-count $THREAD_COUNT --timeout-secs $TASK_DURATION --cgroup w$S --weight $W --cookie-groups-size 3 --cookie-affinity --cfs-bw-quota $Q &
}

set -euxo pipefail

DURATION=1800
DURATION_MS=$(( 1000 * DURATION ))
BUFFER_DURATION=5
PERF_DURATION=$(( DURATION + BUFFER_DURATION ))
TASK_DURATION=180

THREAD_COUNT=4
PROCESS_PER_LOOP=8
OVERSUB_LEVEL=5

NPROC=$(nproc)
SLEEP_DURATION=$((TASK_DURATION * PROCESS_PER_LOOP * THREAD_COUNT / (NPROC * OVERSUB_LEVEL)))
SLEEP_DURATION_MS=$(( 1000 * SLEEP_DURATION ))

LOOPS=$(( DURATION / SLEEP_DURATION ))

BASEDIR=/mnt/share/bin

HI_LAT_THRESHOLD=100
hi_lat=0
perf sched record -- sleep $((SLEEP_DURATION * 2)) && echo "perf done" &
for i in $(seq $LOOPS)
do
	# W=$((10 + $RANDOM % 90))
	# Q=$((10 + $RANDOM % 90))
	GW=33
	GQ=33
	run 100 100
	run 100 $GQ
	run $GW 100
	run $GW $GQ
	perf_restart=$(( SLEEP_DURATION / 2 ))
	for ((t=1; t<=$SLEEP_DURATION; t++))
	do
		set +x
		max_delay=0
		while IFS=' ' read -r name max_delay_str _ throttled_time; do
			if [[ "$max_delay_str" =~ ^max_delay=([0-9]+)$ ]]; then
				delay="${BASH_REMATCH[1]}"
				if ((delay > max_delay)); then
					max_delay="$delay"
				fi
			fi
		done < /sys/fs/cgroup/cpu.children_latency_ns
		set -x
		cat /sys/fs/cgroup/cpu.children_latency_ns
		echo "max delay is $max_delay; hi lat is $hi_lat"
		if ((max_delay > hi_lat)); then
			hi_lat=$max_delay
			cat /proc/uptime
			if ((hi_lat > HI_LAT_THRESHOLD)); then
				break
			fi
		fi
		if ((t == perf_restart)); then
			pkill sleep || echo "sleep is not running"
			sleep 1
			cat /proc/uptime
			perf sched record -- sleep $((SLEEP_DURATION * 2)) && echo "perf done" &
		else
			sleep 1
		fi
	done
	sleep 1
	if ((hi_lat > HI_LAT_THRESHOLD)); then
		echo "high latency detected!"
		break
	fi
done
sleep $BUFFER_DURATION
pkill sleep || echo "sleep is not running"
sleep 2
echo "exiting with hi_lat=$hi_lat"
