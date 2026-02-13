#ifndef EMITTER_H
#define EMITTER_H
#include <stdint.h>
#include <stdbool.h>

void test_event_port_emitter_component_init(const int64_t *in_arg);
void run_emitter(void);
void test_event_port_consumer_component_init(const int64_t *in_arg);
void test_event_port_consumer_time_triggered_handler(void);
bool sb_write_port_enqueue(const int64_t *data);
bool sb_read_port_dequeue(int64_t *data);
const char* get_instance_name(void);

#endif