#ifndef SOURCE_H
#define SOURCE_H
#include <stdint.h>
#include <stdbool.h>

void test_data_port_periodic_domains_source_component_init(const int64_t *in_arg);
void test_data_port_periodic_domains_source_component_time_triggered(const int64_t *in_arg);
void test_data_port_periodic_domains_destination_component_init(const int64_t *in_arg);
void test_data_port_periodic_domains_destination_component_time_triggered(int64_t in_arg);
bool sb_write_port_write(const int64_t *data);
bool sb_read_port_read(int64_t *data);
const char* get_instance_name(void);

#endif