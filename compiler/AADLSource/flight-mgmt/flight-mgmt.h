#ifndef FLIGHT_MGMT_H
#define FLIGHT_MGMT_H

#include <stdint.h>

// Sensor Sim
void sensor_sim_job(void);
void get_aoa(int32_t* val);
void get_climb_rate(int32_t* val);
void get_engine_failure(int32_t* val);

// Stall Monitor
void set_stall_aoa(int32_t val);
void set_stall_climb_rate(int32_t val);
void stall_monitor_job(void);
void get_stall_warn(int32_t* val);

// HCI
void hci_stall_warn_in(int32_t val);
void hci_engine_fail_in(int32_t val);
void hci_gear_cmd_in(int32_t val);
void hci_gear_req_out(int32_t* val);
void hci_gear_ack_in(int32_t val);

// Landing Gear
void gear_req_in(int32_t val);
void gear_dummy_out(int32_t* val);
void gear_dummy_in(int32_t val);
void gear_ack_out(int32_t* val);

// Operator
void operator_cmd_out(int32_t* val);

#endif