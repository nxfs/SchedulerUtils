#!/mnt/share/bin/bash

set -euxo pipefail

perf sched record -D 0-10000 -- sleep 15 && echo "perf done" &

schtest --task 'stress -c 2' --threads-count 2 --timeout-secs 12 --cgroup g1 --cpuset 1,2 --weight 50 --cookie-count 1 &
schtest --task 'stress -c 2' --threads-count 2 --timeout-secs 12 --cgroup g2 --cpuset 1,2 --weight 100 --cookie-count 1 &
schtest --task 'stress -c 4' --threads-count 4 --timeout-secs 12 --cgroup g3 --cpuset 1,2,3,4 --weight 100 --cookie-count 2 &

sleep 15
