#!/mnt/share/bin/bash

set -euxo pipefail

perf sched record -D 0-10000 -- sleep 15 && echo "perf done" &

schedulerutils --task 'stress -c 2' --threads-count 2 --timeout-secs 12 --cgroup s1 --cpuset 0,1 --weight 50 --cookie-groups-size 2 --cookie-affinity &
schedulerutils --task 'stress -c 2' --threads-count 2 --timeout-secs 12 --cgroup s2 --cpuset 0,1 --weight 100 --cookie-groups-size 2 --cookie-affinity &
schedulerutils --task 'stress -c 4' --threads-count 4 --timeout-secs 12 --cgroup s3 --cpuset 0,1,2,3 --weight 100 --cookie-groups-size 2 --cookie-affinity &
schedulerutils --task 'get-cpu-wake -t 4' --threads-count 4 --timeout-secs 12 --cgroup w1 --cpuset 0,1,2,3 --weight 100 --cookie-groups-size 2 --cookie-affinity &

sleep 15
