#ifndef __ARDUPILOT_H__
#define __ARDUPILOT_H__

#include <stdint.h>

/* Mapping AADL types - ALL INT NOW */
typedef int   ardupilot__base_types__float; // Mapped to int in AADL
typedef int   ardupilot__base_types__integer;

/* Subprogram declarations */

/* Split GPS functions */
void gps_simulation_get_lat(ardupilot__base_types__float* lat);
void gps_simulation_get_lon(ardupilot__base_types__float* lon);
void gps_simulation_get_alt(ardupilot__base_types__integer* alt);

void gps_backdoor(float yaw); // Internal usage can stay float

void flt_mgmt_init(void);

/* Split Flight Management functions */
void flt_mgmt_set_lat(ardupilot__base_types__float lat);
void flt_mgmt_set_lon(ardupilot__base_types__float lon);
void flt_mgmt_set_alt(ardupilot__base_types__integer alt);

void flt_mgmt_compute(void);

void flt_mgmt_get_speed(ardupilot__base_types__integer* speed);
void flt_mgmt_get_angle(ardupilot__base_types__integer* angle);

void throttle_simulation(ardupilot__base_types__integer speed);
void yaw_simulation(ardupilot__base_types__integer angle);

/* Time helper */
uint64_t millis(void);

#endif