#ifndef EMITTER_H
#define EMITTER_H

#include <stdint.h>
#include <stdbool.h>

void testevent_emitter_component_init(void);
void run_emitter();
void testevent_consumer_component_init(void);
void testevent_consumer_s_event_handler();
void sb_e_enqueue();
bool sb_s_dequeue();

#endif