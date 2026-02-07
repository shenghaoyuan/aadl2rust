#include <stdio.h>
#include <stdint.h>
#include "torture.h"

static int32_t counter = 0;

void tick_counter(void) {
    counter++;
    printf("\n[Periodic] Cycle %d\n", counter);
    fflush(stdout);
}

// -----------------------------------------------------
// Periodic Sender Functions
// -----------------------------------------------------

void send_p1(int32_t *val) {
    if (val) {
        *val = counter * 10 + 1;
        printf("[Per] Sending P1 = %d\n", *val);
        fflush(stdout);
    }
}

void send_p2(int32_t *val) {
    if (val) {
        *val = counter * 10 + 2;
        printf("[Per] Sending P2 = %d\n", *val);
        fflush(stdout);
    }
}

void send_p3(int32_t *val) {
    if (val) {
        *val = counter * 10 + 3;
        printf("[Per] Sending Loopback P3 = %d\n", *val);
        fflush(stdout);
    }
}

void send_p5(int32_t *val) {
    if (val) {
        *val = counter * 10 + 5;
        printf("[Per] Sending Loopback P5 = %d\n", *val);
        fflush(stdout);
    }
}

void send_p7(int32_t *val) {
    if (val) {
        *val = counter * 10 + 7;
        printf("[Per] Sending P7 = %d\n", *val);
        fflush(stdout);
    }
}

// -----------------------------------------------------
// Periodic Receiver Functions (Loopback)
// -----------------------------------------------------

void recv_p4(int32_t val) {
    printf("[Per] Received Loopback P4 = %d\n", val);
    fflush(stdout);
}

void recv_p6(int32_t val) {
    printf("[Per] Received Loopback P6 = %d\n", val);
    fflush(stdout);
}

// -----------------------------------------------------
// Sporadic Receiver Functions
// -----------------------------------------------------

void recv_spo_p1(int32_t val) {
    printf("[Spo] Received P1 (from Per.P2) = %d\n", val);
    fflush(stdout);
}

void recv_spo_p2(int32_t val) {
    printf("[Spo] Received P2 (from Per.P1) = %d\n", val);
    fflush(stdout);
}

void recv_spo_p3(int32_t val) {
    printf("[Spo] Received P3 (from Per.P7) = %d\n", val);
    fflush(stdout);
}