#!/mnt/share/bin/bash

set -euxo pipefail

perf sched record -D 0-10000 -- sleep 15 && echo "perf done" &

schedulterutils --task 'stress -c 2' --threads-count 2 --timeout-secs 12 --cgroup g1 --cpuset 0,1 --weight 50 --cookie-count 1 &
schedulterutils --task 'stress -c 2' --threads-count 2 --timeout-secs 12 --cgroup g2 --cpuset 0,1 --weight 100 --cookie-count 1 &
schedulterutils --task 'stress -c 4' --threads-count 4 --timeout-secs 12 --cgroup g3 --cpuset 0,1,2,3 --weight 100 --cookie-count 2 &

sleep 15
