#ifndef SOURCE_H
#define SOURCE_H

typedef signed char int8_t;
typedef signed int int32_t;
typedef long long int int64_t;
typedef int bool;

void testdpmon_source_component_init(const int64_t *in_arg);
void run_sender(const int64_t *in_arg);
void testdpmon_destination_component_init(const int64_t *in_arg);
void run_receiver(int64_t in_arg);
bool sb_write_port_write(const int64_t *data);
bool sb_read_port_read(int64_t *data);

#endif