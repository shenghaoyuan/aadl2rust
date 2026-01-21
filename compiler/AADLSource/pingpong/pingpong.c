#include <stdio.h>
#include "pingpong.h"

int p = 0;

void user_ping_spg(int *v) {
    printf("*** SENDING PING *** %d\n", p);
    *v = p;
    p++;
    fflush(stdout);
}

void user_pong_spg(int i) {
    printf("*** PING *** %d\n", i);
    fflush(stdout);
}

void recover(void) {
    printf("*** RECOVER ACTION ***\n");
    fflush(stdout);
}