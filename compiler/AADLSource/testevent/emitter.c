/* testepmon/components/emitter/src/emitter.c */

// #include <camkes.h>
#include <stdio.h>
// #include <sb_types.h>
// #include <sb_emitter_t_impl.h>

typedef long long int int64_t;
typedef int bool;
#define true 1
#define false 0

static int64_t _value;
static bool event_queued = false;

void testevent_emitter_component_init(const int64_t *in_arg)
{
    printf("testevent_emitter_component_init called\n");
}

/* control thread: keep calling enqueue for thing
 */
void run_emitter()
{
    sb_e_enqueue();
    printf("[Emitter] Sent event.\n");
}

void testevent_consumer_component_init(const int64_t *in_arg) {
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