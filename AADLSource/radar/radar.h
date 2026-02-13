#ifndef __RADAR_H__
#define __RADAR_H__

typedef int radar_types__motor_position;
typedef int radar_types__target_distance;

void receiver_spg(radar_types__target_distance *receiver_out);
void controller_spg(radar_types__motor_position *controller_out);

/* Split Analyser functions (Fully Flattened) */
void analyser_spg_set_distance(radar_types__target_distance from_receiver);
void analyser_spg_set_angle(radar_types__motor_position from_controller);
void analyser_spg_do_compute(void);
void analyser_spg_get_distance(radar_types__target_distance *dist_res);
void analyser_spg_get_angle(radar_types__motor_position *angle_res);

void display_spg_dist(radar_types__target_distance d_in);
void display_spg_angle(radar_types__motor_position a_in);

void transmitter_spg(void);

#endif