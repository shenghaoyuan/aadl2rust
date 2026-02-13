#include <stdio.h>
#include "time_triggered.h"

static time_triggered__simple_type b0_cycles = 0;
static time_triggered__simple_type b1_cycles = 0;

void b0_send(time_triggered__simple_type* out_value) {
    *out_value = b0_cycles;
    printf("B0: sending: %d\n", *out_value);
    fflush(stdout);
    b0_cycles++;
}

void b1_receive(time_triggered__simple_type in_value) {
    printf("B1: received %d ", in_value);
}

void b1_send(time_triggered__simple_type* out_value) {
    *out_value = b1_cycles;
    printf("sending %d\n", *out_value);
    fflush(stdout);
    b1_cycles++;
}

void b2_receive(time_triggered__simple_type in_value) {
    printf("B2: received %d\n", in_value);
    fflush(stdout);
}