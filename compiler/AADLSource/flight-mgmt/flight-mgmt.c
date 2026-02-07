// #include <stdio.h>
// #include <request.h>
// #include <deployment.h>
// #include <po_hi_gqueue.h>

// int job = 0;

// void on_req(__po_hi_task_id self);
// void on_dummy(__po_hi_task_id self);
// void on_dummy_in(__po_hi_task_id self);
// void on_stall_warning (__po_hi_task_id self, int i);
// void on_engine_failure(__po_hi_task_id self);
// void on_gear_cmd(__po_hi_task_id self);
// void on_gear_ack(__po_hi_task_id self);
// void on_operator (__po_hi_task_id self);
// void on_sensor_sim(__po_hi_task_id self);
// void on_stall_monitor(__po_hi_task_id self);

// void on_req(__po_hi_task_id self)
// {
//   printf("=== Starting gear op ===\n");
//   fflush (stdout);

//   __po_hi_request_t *request = __po_hi_get_request(invalid_port_t);
//   request->port = landing_gear_global_dummy_out;
//   __po_hi_gqueue_store_out(self,landing_gear_local_dummy_out,request);
// }

// void on_dummy(__po_hi_task_id self)
// {
//   (void) self;
//   printf("=== Starting gear done ===\n");
//   fflush (stdout);
// }

// void on_dummy_in(__po_hi_task_id self)
// {
//   printf("=== Gear op done ===\n");
//   fflush (stdout);
//   __po_hi_request_t *request = __po_hi_get_request(invalid_port_t);
//   __po_hi_gqueue_store_out(self,landing_gear_local_ack,request);
// }

// void on_stall_warning (__po_hi_task_id self, int i)
// {
//   (void) self;
//   if (i==1)
//     {
//       printf("=== STALL ALARM %d ====\n", i);
//       fflush (stdout);
//     }
//   else
//     {
//       printf("=== False Alert %d ====\n", i);
//       fflush (stdout);
//     }
// }

// void on_engine_failure(__po_hi_task_id self)
// {
//   (void) self;
//   printf("=== ENGINE FAILURE ALARM ===\n");
//   fflush (stdout);
// }

// void on_gear_cmd(__po_hi_task_id self)
// {
//   printf("=== %d ===\n", __LINE__); fflush (stdout);
//   __po_hi_request_t *request = __po_hi_get_request(invalid_port_t);
//   __po_hi_gqueue_store_out(self,hci_local_gear_req,request);
// }

// void on_gear_ack(__po_hi_task_id self)
// {
//   (void) self;
//   printf("=== Gear Locked ===\n");
//   fflush (stdout);
// }

// void on_operator (__po_hi_task_id self)
// {
//      printf("=== on_operator ===\n");
//   fflush (stdout);
//  __po_hi_request_t *request = __po_hi_get_request(invalid_port_t);
//   __po_hi_gqueue_store_out(self,operator_local_gear_cmd,request);
// }

// void on_sensor_sim(__po_hi_task_id self)
// {
//   int cr_v = 0;
//   int aoa_v = 4;

//   job++;
//   printf ("== on_sensor_sim %d ==\n", job); fflush(stdout);
//   if ( (job%40) == 0 )
//     {
//       __po_hi_request_t *request1 = __po_hi_get_request(invalid_port_t);
//       __po_hi_request_t *request2 = __po_hi_get_request(invalid_port_t);
//       request1->vars.sensor_sim_global_aoa.sensor_sim_global_aoa = 41;
//       request2->vars.sensor_sim_global_climb_rate.sensor_sim_global_climb_rate = 4;
//       __po_hi_gqueue_store_out(self,sensor_sim_local_aoa,request1);
//       __po_hi_gqueue_store_out(self,sensor_sim_local_climb_rate,request2);
//       printf("=== Sensor Sim setting soft stall ===\n");
//       fflush (stdout);
//     }
//   else
//     {
//       if ( (job%201) == 0 )
//         {
//           __po_hi_request_t *request1 = __po_hi_get_request(invalid_port_t);
//           __po_hi_request_t *request2 = __po_hi_get_request(invalid_port_t);
//           request1->vars.sensor_sim_global_aoa.sensor_sim_global_aoa = 25;
//           request2->vars.sensor_sim_global_climb_rate.sensor_sim_global_climb_rate = 9;
//           __po_hi_gqueue_store_out(self,sensor_sim_local_aoa,request1);
//           __po_hi_gqueue_store_out(self,sensor_sim_local_climb_rate,request2);
//           printf("=== Sensor Sim setting hard stall ===\n");
//           fflush (stdout);
//         }
//       else
//         {
//           if ( (job%401) == 0 )
//             {
//               __po_hi_request_t *request1 = __po_hi_get_request(invalid_port_t);
//               __po_hi_gqueue_store_out(self,sensor_sim_local_engine_failure,request1);
//               printf("=== Sensor Sim raising engine failure ===\n");
//               fflush (stdout);
//             }
//           else
//             {
//               __po_hi_request_t *request1 = __po_hi_get_request(invalid_port_t);
//               __po_hi_request_t *request2 = __po_hi_get_request(invalid_port_t);
//               request1->vars.sensor_sim_global_aoa.sensor_sim_global_aoa = aoa_v;
//               request2->vars.sensor_sim_global_climb_rate.sensor_sim_global_climb_rate = cr_v;
//               __po_hi_gqueue_store_out(self,sensor_sim_local_aoa,request1);
//               __po_hi_gqueue_store_out(self,sensor_sim_local_climb_rate,request2);
//             }
//         }
//     }
// }

// void on_stall_monitor(__po_hi_task_id self)
// {
//   int aoa_v;
//   int cr_v;
//       printf("=== on_stall_monitor ===\n");
//   fflush (stdout);
//   __po_hi_request_t *request_aoa;
//   __po_hi_request_t *request_cr;
// // data port, no need to free
//   __po_hi_gqueue_get_value(self,stall_monitor_local_aoa,&request_aoa);
//   if (request_aoa != NULL) {
//     aoa_v = request_aoa->vars.sensor_sim_global_aoa.sensor_sim_global_aoa;
//   }
//   else
//     aoa_v = 0;
// // data port, no need to free
//   __po_hi_gqueue_get_value(self,stall_monitor_local_climb_rate,&request_cr);
//   if (request_cr != NULL) {
//     cr_v = request_cr->vars.sensor_sim_global_climb_rate.sensor_sim_global_climb_rate;

//   }
//   else
//     cr_v = 0;

//   printf ("AOA: %d %d\n", aoa_v, cr_v);
// request_aoa = NULL;
// request_cr = NULL;
//   if (aoa_v > 40)
//     {
//       printf("=== %d ===\n", __LINE__); fflush (stdout);
//       __po_hi_request_t *request_out = __po_hi_get_request(invalid_port_t);
//       request_out->vars.stall_monitor_global_stall_warn.stall_monitor_global_stall_warn = 2;
//       __po_hi_gqueue_store_out(self,stall_monitor_local_stall_warn,request_out);
//     }
//   else
//     {
//       if ((aoa_v > 22 ) && (cr_v < 10))
//         {
//             printf("=== %d ===\n", __LINE__); fflush (stdout);
//             __po_hi_request_t *request_out = __po_hi_get_request(invalid_port_t);
//             request_out->vars.stall_monitor_global_stall_warn.stall_monitor_global_stall_warn = 2;
//             __po_hi_gqueue_store_out(self,stall_monitor_local_stall_warn,request_out);
//         }
//     }
// }

#include <stdio.h>
#include <stdint.h>
#include "flight-mgmt.h"

// === Global State ===
static int sensor_job = 0;
static int32_t g_aoa = 4;
static int32_t g_climb_rate = 0;
static int32_t g_engine_fail = 0;

static int32_t g_stall_aoa = 0;
static int32_t g_stall_cr = 0;
static int32_t g_stall_warn = 0;

static int32_t g_gear_req = 0;

// === Sensor Sim ===
void sensor_sim_job(void)
{
  sensor_job++;
  printf ("[Sensor] Job %d running...\n", sensor_job); 
  
  // Default values
  g_aoa = 4;
  g_climb_rate = 0;
  g_engine_fail = 0;

  if ( (sensor_job % 4) == 0 ) 
  {
      g_aoa = 41;
      g_climb_rate = 4;
      printf("[Sensor] Setting SOFT STALL (AoA=41)\n");
  }
  else if ( (sensor_job % 6) == 0 ) 
  {
      g_aoa = 25;
      g_climb_rate = 9;
      printf("[Sensor] Setting HARD STALL (AoA=25, CR=9)\n");
  }
  else if ( (sensor_job % 10) == 0 ) 
  {
      g_engine_fail = 1;
      printf("[Sensor] Raising ENGINE FAILURE\n");
  }
  else
  {
      printf("[Sensor] Normal flight (AoA=4)\n");
  }
  fflush (stdout);
}

void get_aoa(int32_t* val) { *val = g_aoa; }
void get_climb_rate(int32_t* val) { *val = g_climb_rate; }
void get_engine_failure(int32_t* val) { *val = g_engine_fail; }


// === Stall Monitor ===
void set_stall_aoa(int32_t val) { g_stall_aoa = val; }
void set_stall_climb_rate(int32_t val) { g_stall_cr = val; }

void stall_monitor_job(void) {
    g_stall_warn = 0;
    
    printf ("[StallMonitor] Processing AoA: %d, CR: %d\n", g_stall_aoa, g_stall_cr);

    if (g_stall_aoa > 40) {
        printf("[StallMonitor] !!! STALL WARNING (AoA > 40) !!!\n");
        g_stall_warn = 2;
    } else if ((g_stall_aoa > 22 ) && (g_stall_cr < 10)) {
        printf("[StallMonitor] !!! STALL WARNING (AoA > 22 & CR < 10) !!!\n");
        g_stall_warn = 2;
    } else {
        printf("[StallMonitor] Status OK\n");
    }
    fflush (stdout);
}

void get_stall_warn(int32_t* val) { *val = g_stall_warn; }


// === HCI ===
void hci_stall_warn_in(int32_t val) {
    if (val == 2) printf("[HCI] === STALL ALARM RECEIVED ====\n");
    fflush(stdout);
}
void hci_engine_fail_in(int32_t val) {
    if (val != 0) printf("[HCI] === ENGINE FAILURE ALARM RECEIVED ===\n");
    fflush(stdout);
}
void hci_gear_cmd_in(int32_t val) {
    printf("[HCI] Received Gear Cmd: %d\n", val);
    g_gear_req = 1; // Set flag to send request
    fflush(stdout);
}
void hci_gear_req_out(int32_t* val) {
    *val = g_gear_req;
    // Reset after sending (simulation logic)
    // g_gear_req = 0; 
}
void hci_gear_ack_in(int32_t val) {
    printf("[HCI] === Gear Locked (ACK Received: %d) ===\n", val);
    fflush(stdout);
}


// === Landing Gear ===
void gear_req_in(int32_t val) {
    if (val != 0) printf("[LandingGear] Received Request (%d). Starting op...\n", val);
    fflush(stdout);
}
void gear_dummy_out(int32_t* val) { *val = 1; }
void gear_dummy_in(int32_t val) {
    if (val != 0) printf("[LandingGear] Dummy In received. Op done.\n");
    fflush(stdout);
}
void gear_ack_out(int32_t* val) { *val = 1; }


// === Operator ===
void operator_cmd_out(int32_t* val) {
    printf("[Operator] Commanding Gear Down/Up...\n");
    *val = 1;
    fflush(stdout);
}