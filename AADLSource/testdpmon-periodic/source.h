#ifndef SOURCE_H
#define SOURCE_H

#include <stdint.h>
#include <stdbool.h>

void testdpmon_source_component_init(void);
void run_sender(const int64_t *in_arg);
void testdpmon_destination_component_init(void);
void run_receiver(int64_t in_arg);
bool sb_write_port_write(const int64_t *data);
bool sb_read_port_read(int64_t *data);

#endif