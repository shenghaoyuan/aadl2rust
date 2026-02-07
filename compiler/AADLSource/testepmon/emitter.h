#ifndef EMITTER_H
#define EMITTER_H

#include <stdint.h>
#include <stdbool.h>

void testepmon_emitter_component_init(void);
void run_emitter(const int64_t *in_arg);
void testepmon_consumer_component_init(void);
void testepmon_consumer_s_event_handler(int64_t in_arg);
bool sb_enq_enqueue(const int64_t *data);
bool sb_deq_dequeue(int64_t *data);

#endif