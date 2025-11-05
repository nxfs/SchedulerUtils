#!/mnt/share/bin/bash

set -euxo pipefail

# Configuration
test_duration=10

num_groups=6
threads_per_group=4

use_cookies=1
cookie_groups_size=2

perf sched record -D 0-$((test_duration * 1000)) -- sleep $((test_duration + 5)) && echo "perf done" &

if [ $use_cookies -eq 1 ]; then
    cookie_arg="--cookie-groups-size $cookie_groups_size"
else
    cookie_arg=""
fi

for i in $(seq 1 $num_groups); do
    schedulerutils --task "get-cpu-wake -t $threads_per_group -s 25" --threads-count $threads_per_group --timeout-secs $((test_duration + 2)) --cgroup w$i $cookie_arg &
done

sleep $((test_duration + 10))
