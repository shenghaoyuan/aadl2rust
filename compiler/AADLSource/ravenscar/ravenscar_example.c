#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include "ravenscar_example.h"

/* ========================================================= */
/* Production Workload (Simulation)                          */
/* ========================================================= */

void small_whetstone(int n) {
    /* Simulate CPU load */
    volatile double x = 1.0001;
    long i;
    long count = n * 10000; 
    
    for (i = 0; i < count; i++) {
        x = 1.0 / x;
    }
}

/* ========================================================= */
/* Auxiliary Logic (Simulation)                              */
/* ========================================================= */

int aux_counter = 0;

int due_activation(int criterion) {
    aux_counter++;
    return (aux_counter % criterion == 0);
}

int check_due() {
    return (aux_counter % 3 == 0);
}

/* ========================================================= */
/* Event Source Logic                                        */
/* ========================================================= */

static int es_criterion = 0;
static int es_divisors[] = {2, 3, 5, 7};
static int es_divisor_idx = 0;
static ravenscar_example__interrupt_counter es_interrupt_count = 0;

void event_source_init(void) {
    printf("External Events: starting\n");
    es_criterion = 0;
    es_divisor_idx = 0;
    es_interrupt_count = 0;
}

void new_external_event(ravenscar_example__interrupt_counter* interrupt_out) {
    int current_divisor = es_divisors[es_divisor_idx];
    
    if (es_criterion % current_divisor == 0) {
        es_divisor_idx = (es_divisor_idx + 1) % 4;
        es_interrupt_count++;

        printf("External Events: send an new event: %d. Next divisor = %d\n", 
               es_interrupt_count, es_divisors[es_divisor_idx]);
        
        *interrupt_out = es_interrupt_count;
    } else {
        *interrupt_out = es_interrupt_count; 
    }

    es_criterion++;
}

/* ========================================================= */
/* Work Logic                                                */
/* ========================================================= */

#define REGULAR_PRODUCER_WORKLOAD 498
#define ON_CALL_PRODUCER_WORKLOAD 250
#define ACTIVATION_CONDITION 2

void regular_producer_operation(ravenscar_example__workload* add_load) {
    printf("Regular Producer: doing some work.\n");
    
    if (due_activation(ACTIVATION_CONDITION)) {
        printf("Sending extra work to 'On_Call_Producer': %d\n", ON_CALL_PRODUCER_WORKLOAD);
        *add_load = ON_CALL_PRODUCER_WORKLOAD;
    } else {
        *add_load = 0; 
    }

    if (check_due()) {
        printf("Signaling 'Activation Log Reader'\n");
    }

    printf("Regular Producer: end of cyclic activation\n");
}

void on_call_producer_operation(ravenscar_example__workload load_in) {
    if (load_in > 0) {
        printf("On Call Producer: doing some work (%d).\n", load_in);
        printf("On Call Producer: end of sporadic activation.\n");
    }
}

/* ========================================================= */
/* Events Logic (Split)                                      */
/* ========================================================= */

static ravenscar_example__interrupt_counter g_depository = 0;

void delegate_ext_event_set_depository(ravenscar_example__interrupt_counter depository_in) {
    printf("External Event Server: received an external interrupt %d\n", depository_in);
    g_depository = depository_in;
}

void delegate_ext_event_get_interrupt(ravenscar_example__interrupt_counter* interrupt_out) {
    *interrupt_out = g_depository;
    printf("External Event Server: end of sporadic activation.\n");
}

/* ========================================================= */
/* Logs Logic                                                */
/* ========================================================= */

#define LOG_LOAD 125
static ravenscar_example__interrupt_counter old_interrupt_counter = -1;

void on_signal(ravenscar_example__interrupt_counter interrupt_in) {
    printf("Activation Log Reader: do some work.\n");

    if (interrupt_in != old_interrupt_counter) {
        printf("Read external new interruption: %d\n", interrupt_in);
        old_interrupt_counter = interrupt_in;
    } else {
        printf("Activation Log Reader: no new interrupts.\n");
    }

    printf("Activation Log Reader: end of sporadic activation.\n");
}