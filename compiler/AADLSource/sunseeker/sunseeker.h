#ifndef SUNSEEKER_H
#define SUNSEEKER_H
#include <stdint.h>

// Controller
void controller_receive(int32_t val);
void controller_compute(void);
void controller_send(int32_t *val);

// Plant
void plant_receive(int32_t val);
void plant_compute(void);
void plant_send(int32_t *val);

#endif