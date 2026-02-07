#define _POSIX_C_SOURCE 199309L
#include <stdio.h>
#include <time.h>
#include <unistd.h>
#include "hello.h"

typedef struct {
    int val1;
    int val2;
} TotoResult;

TotoResult toto() {
    TotoResult res;
    res.val1 = 1;
    res.val2 = 2;
    return res;
}

void time_get(time_t *sec, long *nsec) {
    struct timespec ts;
    clock_gettime(CLOCK_MONOTONIC, &ts);
    *sec = ts.tv_sec;
    *nsec = ts.tv_nsec;
}

void time_delay_until(time_t target_sec, long target_nsec) {
    struct timespec target_ts, remaining_ts;
    target_ts.tv_sec = target_sec;
    target_ts.tv_nsec = target_nsec;
    struct timespec current_ts;
    clock_gettime(CLOCK_MONOTONIC, &current_ts);
    long sec_diff = target_ts.tv_sec - current_ts.tv_sec;
    long nsec_diff = target_ts.tv_nsec - current_ts.tv_nsec;
    
    if (nsec_diff < 0) {
        sec_diff -= 1;
        nsec_diff += 1000000000;
    }
    
    if (sec_diff > 0 || nsec_diff > 0) {
        struct timespec sleep_ts = {sec_diff, nsec_diff};
        while (nanosleep(&sleep_ts, &remaining_ts) == -1) {
            sleep_ts = remaining_ts;
        }
    }
}

void lua_sample() {
    printf("HELLO MAXIME\n");
    time_t sec;
    long nsec;
    time_get(&sec, &nsec);
    sec = sec + 5;
    time_delay_until(sec, nsec);
    TotoResult res = toto();
    printf("r1  : %dr2 : %d\n", res.val1, res.val2);
}

void hello_func() {
    printf("HELLO1\n");
}