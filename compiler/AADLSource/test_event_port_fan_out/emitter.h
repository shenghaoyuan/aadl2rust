#ifndef EMITTER_H
#define EMITTER_H
#include <stdint.h>
#include <stdbool.h>

void test_event_port_emitter_component_init(const int64_t *in_arg);
void run_emitter(void);
void test_event_port_consumer_component_init(const int64_t *in_arg);
void test_event_port_consumer_s_event_handler(void);
bool enqueue(int queue_idx, int64_t data);
bool dequeue(int queue_idx, int64_t *data);

#endif