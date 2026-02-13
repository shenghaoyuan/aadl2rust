#ifndef PINGER_H
#define PINGER_H

#include <stdint.h>

void user_produce_pkts_init(void);
void user_produce_pkts(void);
void user_do_ping_spg(int32_t* data_source);
void user_ping_spg(int32_t i);
void recover(void);

#endif