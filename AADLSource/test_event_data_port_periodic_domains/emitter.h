#ifndef EMITTER_H
#define EMITTER_H
#include <stdint.h>
#include <stdbool.h>

void test_event_data_port_emitter_component_init(const int64_t *in_arg);
void test_event_data_port_emitter_time_triggered_handler(const int64_t *in_arg);
void test_event_data_port_consumer_component_init(const int64_t *in_arg);
void test_event_data_port_consumer_time_triggered_handler(int64_t in_arg);
bool sb_write_port_enqueue(const int64_t *data);
bool sb_read_port_dequeue(int64_t *data);
const char* get_instance_name(void);

#endif