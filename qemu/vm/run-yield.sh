#!/mnt/share/bin/bash

set -euxo pipefail

perf sched record -D 0-10000 -- sleep 32 && echo "perf done" &
sleep 1
schedulerutils --task 'get-cpu-wake -t 32 -s 100 -m 10000 -M 100000' --threads-count 32 --timeout-secs 32 --cgroup s1 --cookie-groups-size 2 --cookie-affinity &
sleep 35
