// /* Files generated from AADL model */
// #include <request.h>
// #include <deployment.h>
// #include <po_hi_gqueue.h>
 
// #include "simu.h" /* MinePump simulator */

// #define MS_L1 70
// #define MS_L2 100

// #define Normal 0
// #define Alarm1 1
// #define Alarm2 2
// #define LowLevel 0
// #define HighLevel 1

// /*****************************************************************************/
// /* Helper macros to access AADL entities                                     */

// #define LOCAL_PORT(THREAD_INSTANCE_NAME, PORT_NAME) THREAD_INSTANCE_NAME ## _local_ ## PORT_NAME
// #define REQUEST_PORT(THREAD_INSTANCE_NAME, PORT_NAME) THREAD_INSTANCE_NAME ## _global_ ## PORT_NAME
// #define PORT_VARIABLE(THREAD_INSTANCE_NAME, PORT_NAME) vars.REQUEST_PORT(THREAD_INSTANCE_NAME,PORT_NAME).REQUEST_PORT(THREAD_INSTANCE_NAME,PORT_NAME)

// /*****************************************************************************/
// /* WaterLevelMonitoring_Thread is a periodic task, period = 250 ms.
//    It has one output port called "WaterAlarm", of integer type

//    At each dispatch, it reads the HLS and LLS sensors, using the
//    ReadHLS() and RealLLS() functions.

//    - If HLS is true, it sends "HighValue" through its out port;
//    - else, if LLS is false, it sends "LowValue";
//    - otherwise, it sends the previous value.
// */

// void waterlevelmonitoring_body(__po_hi_task_id self) {

//   /* Logic of the thread */
//   static int waterlvl = LowLevel;

//   if (ReadHLS()) {
//     waterlvl = HighLevel;
//   } else if (!ReadLLS()) {
//     waterlvl = LowLevel;
//   }

//   /* Send waterlvl through the wateralarm port */

//   /* NOTE: Sending through an output port requires some discipline in
//      naming conventions */

//   __po_hi_request_t request;  /* AADL request type */

//   /* The name of an output port is built from the thread_instance name
//        and the port name using the REQUEST_PORT macro */

//   request.port = REQUEST_PORT (waterlevelmonitoring_thread, wateralarm);

//   /* The name of the corresponding port variable is built from the
//      port name, following similar pattern. */

//   request.PORT_VARIABLE (waterlevelmonitoring_thread,wateralarm) = waterlvl;

//   /* We send the request through the thread *local* port, built from
//      the instance name and the port name using the LOCAL_PORT macro */

//   __po_hi_gqueue_store_out
//       (self,
//        LOCAL_PORT (waterlevelmonitoring_thread, wateralarm),
//        &request);
// }

// /*****************************************************************************/
// /* MethaneMonitoring_Thread is a periodic task, period = 100 ms.
//    It has one output port called "MethaneLevel", of integer type

//    At each dispatch, it reads the MS sensor. Depending on the methane
//    level (constant MS_L1 and MS_L2), it sends either normal, Alert1 or
//    Alert2 through its output port.
// */

// void methanemonitoring_body (__po_hi_task_id self) {

//   /* Logic of the thread */
//   int level = Normal;
//   BYTE MS;

//   MS=ReadMS();
//   if (MS>MS_L2) {
//     level=Alarm2;
//   } else if (MS>MS_L1) {
//     level=Alarm1;
//   } else {
//     level=Normal;
//   }

//   /* Port management */
//   __po_hi_request_t request;
//   request.port = REQUEST_PORT (methanemonitoring_thread, methanelevel);
//   request.PORT_VARIABLE (methanemonitoring_thread,methanelevel) = level;
//   __po_hi_gqueue_store_out
//     (self,
//      LOCAL_PORT (methanemonitoring_thread, methanelevel),
//      &request);

// }

// /*****************************************************************************/
// /* PumpCtrl_Thread is a sporadic task, with a MIAT of 1 ms It is
//    triggered by incoming events from sensors.

//    It has two input ports (dispatching)
//    * MethaneLevel, of integer type
//    * WaterLevel, of integer type

//    and one output port
//    * WaterAlarm, of integer type

//    This task manages the alarm logic, and the pump.

//    - if the alarm level is different from Normal, it sends the value 1
//      through its wateralarm output port, otherwise it sends 0;
//    - if the alarm level is Alarm2 then the pump is deactivated (cmd =
//      0 sent to CmdPump); else, if the water level is high, then the
//      pump is activated (cmd = 1 sent to CmdPump), otherwise the pump
//      is left off.
// */

// void pumpctrl_body(__po_hi_task_id self) {
//   int niveau_eau, niveau_alarme, alarme;
//   int cmd=0;

//   /* Read from the MethaneLevel port */
//   __po_hi_request_t request;

//   /* Get the value from the methanelevel port */
//   __po_hi_gqueue_get_value
//     (self,
//      LOCAL_PORT (pumpctrl_thread,methanelevel),
//      &(request));

//   /* Extract it from the port variable */
//   niveau_alarme = request.PORT_VARIABLE (pumpctrl_thread,methanelevel);

//   /* Dequeue the event */
//   __po_hi_gqueue_next_value (self, LOCAL_PORT (pumpctrl_thread,methanelevel));

//   if (niveau_alarme==Normal) {
//     alarme=0;
//   } else {
//     alarme=1;
//   }

//   /* Send alarme value through the WaterAlarm port */

//   request.port = REQUEST_PORT(pumpctrl_thread, wateralarm);
//   request.PORT_VARIABLE(pumpctrl_thread,wateralarm) = alarme;
//   __po_hi_gqueue_store_out
//     (self,
//      LOCAL_PORT (pumpctrl_thread, wateralarm),
//      &request);

//   if (niveau_alarme==Alarm2) {
//     cmd=0;
//   } else {

//     /* Read from the WaterLevel port */
//     __po_hi_gqueue_get_value(self,LOCAL_PORT (pumpctrl_thread,waterlevel),&(request));
//     niveau_eau = request.PORT_VARIABLE(pumpctrl_thread,waterlevel);
//     __po_hi_gqueue_next_value (self, LOCAL_PORT (pumpctrl_thread,waterlevel));

//     if (niveau_eau==HighLevel) {
//       cmd=1;
//     } else if (niveau_eau==LowLevel) {
//       cmd=0;
//     }
//   }

//   CmdPump(cmd);
// }

// /*****************************************************************************/
// /* WaterAlarm_Thread is a sporadic task, with a MIAT of 1 ms
//    It has one input port (dispatching)
//    * WaterAlarm, of integer type

//    It calls CmdAlarm with the value read.
// */

// void wateralarm_body(__po_hi_task_id self) {
//   int value = 1;

//   /* Read from the WaterAlarm port */

//   __po_hi_request_t request;
//   __po_hi_gqueue_get_value(self,LOCAL_PORT (wateralarm_thread,wateralarm),&(request));
//   value = request.PORT_VARIABLE(wateralarm_thread,wateralarm);
//   __po_hi_gqueue_next_value (self, LOCAL_PORT (wateralarm_thread, wateralarm));

//   CmdAlarm(value);
// }

#include <stdio.h>
#include <stdlib.h>
#include <pthread.h>
#include "minepump.h"

/* ==========================================
   Simulator Implementation
   ========================================== */

pthread_mutex_t mutex_simu_minepump;
int CmdPump_Value = 0; /* 0 = off, 1 = on */
BYTE WaterLevel_Value = 50; /* Initial water level */
int simu_initialized = 0;

void InitSimu(void) {
    if (simu_initialized) return;
    pthread_mutex_init(&mutex_simu_minepump, NULL);
    simu_initialized = 1;
    printf("[SIMU] Simulator Initialized.\n");
}

BYTE ReadHLS(void) {
    InitSimu();
    BYTE b;
    pthread_mutex_lock(&mutex_simu_minepump);
    
    WaterLevel_Value = WaterLevel_Value - (CmdPump_Value * 4) + 2;
    
    if (WaterLevel_Value > 200) WaterLevel_Value = 200;

    b = (WaterLevel_Value > 100) ? 1 : 0;
    
    pthread_mutex_unlock(&mutex_simu_minepump);
    return b;
}

BYTE ReadLLS(void) {
    InitSimu();
    BYTE b;
    pthread_mutex_lock(&mutex_simu_minepump);
    
    b = (WaterLevel_Value > 80) ? 1 : 0;
    
    pthread_mutex_unlock(&mutex_simu_minepump);
    return b;
}

BYTE ReadMS(void) {
    InitSimu();
    static BYTE b = 50;
    static int b_increment = 2;

    pthread_mutex_lock(&mutex_simu_minepump);
    
    b += b_increment;
    if (b >= 150 || b <= 40)
        b_increment = -b_increment;

    pthread_mutex_unlock(&mutex_simu_minepump);
    return b;
}

void CmdPump(BYTE cmd) {
    InitSimu();
    pthread_mutex_lock(&mutex_simu_minepump);
    CmdPump_Value = cmd ? 1 : 0;
    printf("[HARDWARE] WL=%3d | Pump is %s\n", WaterLevel_Value, CmdPump_Value ? "ON" : "OFF");
    fflush(stdout);
    pthread_mutex_unlock(&mutex_simu_minepump);
}

void CmdAlarm(BYTE cmd) {
    InitSimu();
    int c = cmd ? 1 : 0;
    pthread_mutex_lock(&mutex_simu_minepump);
    printf("[HARDWARE] Alarm is %s\n", c ? "ON" : "OFF");
    fflush(stdout);
    pthread_mutex_unlock(&mutex_simu_minepump);
}

/* ==========================================
   User Logic Implementation (AADL mapped)
   ========================================== */

#define MS_L1 70
#define MS_L2 100

#define Normal 0
#define Alarm1 1
#define Alarm2 2

#define LowLevel 0
#define HighLevel 1

/* 
 * Task: WaterLevelMonitoring
 * Reads sensors and outputs High/Low status
 */
void water_level_monitoring(int* water_alarm) {
    static int waterlvl = LowLevel;

    if (ReadHLS()) {
        waterlvl = HighLevel;
    } else if (!ReadLLS()) {
        waterlvl = LowLevel;
    }
    *water_alarm = waterlvl;
}

/* 
 * Task: MethaneMonitoring
 * Reads Methane sensor and outputs Alarm level
 */
void methane_monitoring(int* methane_level) {
    int level = Normal;
    BYTE MS;

    MS = ReadMS();
    if (MS > MS_L2) {
        level = Alarm2;
    } else if (MS > MS_L1) {
        level = Alarm1;
    } else {
        level = Normal;
    }

    *methane_level = level;
}

/* 
 * Task: PumpCtrl - Split Implementation
 * Storage for inputs between subprogram calls within the thread loop
 */
static int g_methane_level = Normal;
static int g_water_level = LowLevel;

/* Step 1: Read Methane Level */
void pump_ctrl_read_methane(int methane_level) {
    g_methane_level = methane_level;
}

/* Step 2: Read Water Level */
void pump_ctrl_read_water(int water_level) {
    g_water_level = water_level;
}

/* Step 3: Execute Logic and Output Alarm */
void pump_ctrl_logic(int* water_alarm) {
    int alarme;
    int cmd = 0;

    /* Use stored values: g_methane_level and g_water_level */

    /* Alarm Logic */
    if (g_methane_level == Normal) {
        alarme = 0;
    } else {
        alarme = 1;
    }
    
    *water_alarm = alarme;

    /* Pump Logic */
    if (g_methane_level == Alarm2) {
        /* Dangerous Methane levels, force pump OFF */
        cmd = 0;
    } else {
        if (g_water_level == HighLevel) {
            cmd = 1;
        } else if (g_water_level == LowLevel) {
            cmd = 0;
        } else {
            /* Keep previous state (hysteresis), use simulation state */
             cmd = CmdPump_Value; 
        }
    }

    CmdPump((BYTE)cmd);
}

/* 
 * Task: WaterAlarm
 * Input: WaterAlarm
 * Controls the Alarm Hardware directly
 */
void water_alarm_task(int water_alarm) {
    CmdAlarm((BYTE)water_alarm);
}