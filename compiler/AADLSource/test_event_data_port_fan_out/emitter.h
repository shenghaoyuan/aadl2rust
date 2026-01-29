#ifndef SOURCE_H
#define SOURCE_H
#include <stdint.h>
#include <stdbool.h>

void test_event_data_port_emitter_component_init(const int64_t *in_arg);
void run_emitter(const int64_t *in_arg);
void test_event_data_port_consumer_component_init(const int64_t *in_arg);
void test_event_data_port_consumer_s_event_handler(int64_t in_arg);
bool enqueue(int queue_idx, int64_t data);
bool dequeue(int queue_idx, int64_t *data);

#endif