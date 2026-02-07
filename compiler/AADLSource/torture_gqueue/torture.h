#ifndef TORTURE_H
#define TORTURE_H

#include <stdint.h>

void tick_counter(void);

void send_p1(int32_t *val);
void send_p2(int32_t *val);
void send_p3(int32_t *val);
void send_p5(int32_t *val);
void send_p7(int32_t *val);

void recv_p4(int32_t val);
void recv_p6(int32_t val);

void recv_spo_p1(int32_t val);
void recv_spo_p2(int32_t val);
void recv_spo_p3(int32_t val);

#endif