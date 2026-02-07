/* testepmon/components/emitter/src/emitter.c */

// #include <camkes.h>
#include <stdio.h>
// #include <sb_types.h>
// #include <sb_emitter_t_impl.h>
#include <stdint.h>
#include <stdbool.h>
#include "emitter.h"

static int64_t _value;
static bool event_queued = false;

void testevent_emitter_component_init(void)
{
    static bool is_initialized = false;
    if (is_initialized) {
        return;
    }
    printf("testevent_emitter_component_init called\n");
    is_initialized = true;
}

/* control thread: keep calling enqueue for thing
 */
void run_emitter()
{
    sb_e_enqueue();
    printf("[Emitter] Sent event.\n");
}

void testevent_consumer_component_init(void) {
  printf("testevent_consumer_component_init called\n");
  _value = 0;
}

void testevent_consumer_s_event_handler() {
  printf("[Consumer] Callback %d fired.\n", _value);
  _value = (_value + 1) % 500;
}

void sb_e_enqueue() {
    event_queued = true;
}