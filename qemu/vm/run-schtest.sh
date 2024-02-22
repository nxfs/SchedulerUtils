#!/mnt/share/bin/bash

set -euxo pipefail

perf sched record -k raw -- schtest "$@"
