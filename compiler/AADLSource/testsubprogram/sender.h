#ifndef SENDER_H
#define SENDER_H

#include <stdint.h>

typedef struct {
    uint32_t A;
    uint32_t B;
    uint32_t result;
} testsubprogram__operands_impl;

void operations_add(void);
void operations_subtract(void);
void sender_init(void);
void run_sender(void);

#endif