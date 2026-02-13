#include <stdio.h>
#include "radar.h"

#define FAKE_POSITIONS_COUNT 8
int fake_rho[FAKE_POSITIONS_COUNT]   = {1, 2, 3, 1, 2, 3, 2, 3};
int fake_theta[FAKE_POSITIONS_COUNT] = {1, 2, 3, 1, 2, 3, 2, 3};

int fake_index = 0;

/* Static storage for Analyser State */
static radar_types__target_distance g_dist = 0;
static radar_types__motor_position g_angle = 0;

/* Static storage for Display */
static radar_types__target_distance g_disp_dist = 0;

void receiver_spg(radar_types__target_distance *receiver_out) {
    *receiver_out = fake_rho[fake_index];
    printf("Receiver: target is at distance %d\n", *receiver_out);
    fflush(stdout);
    fake_index = (fake_index + 1) % FAKE_POSITIONS_COUNT;
}

void controller_spg(radar_types__motor_position *controller_out) {
    *controller_out = fake_theta[fake_index];
    printf("Controller: motor is at angular position %d\n", *controller_out);
    fflush(stdout);
}

/* Analyser Split Implementation */

void analyser_spg_set_distance(radar_types__target_distance from_receiver) {
    g_dist = from_receiver;
}

void analyser_spg_set_angle(radar_types__motor_position from_controller) {
    g_angle = from_controller;
}

void analyser_spg_do_compute(void) {
    /* Logic is simple here, just print state */
    printf("Analyser: target is at distance: %d at angular position %d\n", 
           g_dist, g_angle);
    fflush(stdout);
}

void analyser_spg_get_distance(radar_types__target_distance *dist_res) {
    *dist_res = g_dist;
}

void analyser_spg_get_angle(radar_types__motor_position *angle_res) {
    *angle_res = g_angle;
}

/* Transmitter Implementation */

void transmitter_spg(void) {
    printf("Transmitter: Pulse sent\n");
    fflush(stdout);
}

/* Display Panel Implementation */

void display_spg_dist(radar_types__target_distance d_in) {
    g_disp_dist = d_in;
}

void display_spg_angle(radar_types__motor_position a_in) {
    printf("Display_Panel: target is at (%d, %d)\n", 
           g_disp_dist, a_in);
    fflush(stdout);
}