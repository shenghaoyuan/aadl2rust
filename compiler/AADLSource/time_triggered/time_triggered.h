#ifndef __TIME_TRIGGERED_H__
#define __TIME_TRIGGERED_H__

/* Mapping Simple_Type to int */
typedef int time_triggered__simple_type;

/* Subprogram declarations */
void b0_send(time_triggered__simple_type* out_value);

/* Split B1 functions */
void b1_receive(time_triggered__simple_type in_value);
void b1_send(time_triggered__simple_type* out_value);

void b2_receive(time_triggered__simple_type in_value);

#endif