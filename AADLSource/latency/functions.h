#ifndef FUNCTIONS_H
#define FUNCTIONS_H

#include <stdint.h>

void sensor_emulator (int* value);
void actuator_emulator (int value);

void spg1_in (int ined);
void spg1_out (int* outed);

void spg2_in (int ined);
void spg2_out (int* outed);

void spg3_in (int ined);
void spg3_out (int* outed);

#endif