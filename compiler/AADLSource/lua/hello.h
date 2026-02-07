#ifndef HELLO2_H
#define HELLO2_H
#define _POSIX_C_SOURCE 199309L
#include <stdint.h>
#include <time.h>

void time_get(time_t *sec, long *nsec);
void time_delay_until(time_t target_sec, long target_nsec);
void lua_sample();
void hello_func();

#endif