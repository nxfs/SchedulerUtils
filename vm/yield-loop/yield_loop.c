#ifndef PR_SCHED_CORE
#define PR_SCHED_CORE 62
#define PR_SCHED_CORE_CREATE 1
#define PR_SCHED_CORE_SHARE_TO 2
#define PR_SCHED_CORE_SCOPE_THREAD 0
#define PR_SCHED_CORE_SCOPE_THREAD_GROUP 1
#endif

#include <sched.h>
#include <time.h>
#include <unistd.h>
#include <sys/prctl.h>
#include <stdlib.h>

int main(int argc, char *argv[]) {
    int should_yield = (argc > 1) ? atoi(argv[1]) : 1;
    time_t program_start = time(NULL);
    
    // Create core cookie for current process
    prctl(PR_SCHED_CORE, PR_SCHED_CORE_CREATE, 0, PR_SCHED_CORE_SCOPE_THREAD, 0);
    
    pid_t pid = fork();
    
    if (pid == 0) {
        // Child: yield for 5s then busy loop (if should_yield is 1)
        if (should_yield) {
            time_t start = time(NULL);
            while (time(NULL) - start < 5 && time(NULL) - program_start < 30) {
                sched_yield();
            }
        }
        while (time(NULL) - program_start < 30) {
            // busy loop
        }
    } else {
        // Parent: share cookie with child, then busy loop
        prctl(PR_SCHED_CORE, PR_SCHED_CORE_SHARE_TO, pid, PR_SCHED_CORE_SCOPE_THREAD, 0);
        while (time(NULL) - program_start < 30) {
            // busy loop
        }
    }
    
    return 0;
}
