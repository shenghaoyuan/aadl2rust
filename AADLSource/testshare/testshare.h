#ifndef TESTSHARE_H
#define TESTSHATE_H
#include <stdint.h>

void testshare_publisher_component_init(void);
void run_publisher(void);
void testshare_subscriber_component_init(void);
void run_subscriber(void);
void b1_release(void);
void b2_acquire(void);

#endif