#!/mnt/share/bin/bash

set -euxo pipefail

perf sched record -D 0-60000 -- sleep 62 && echo "perf done" &
schedulerutils --task 'stress-ng --timer 32 --timer-freq 1000000 --timeout 62' --threads-count 32 --timeout-secs 62 --cgroup s1 --cookie-groups-size 1 --cookie-affinity &

sleep 65
