#include <stdio.h>
#include <stdlib.h>
#include <stdbool.h>
#include <pthread.h>

#include "simu.h"

pthread_mutex_t mutex_simu_minepump;
int CmdPump_Value = 0; /* c = 0, pump is off, c = 1, pump is on */
int WaterLevel_Value = 50;
static int global_methane_level = 0;
static int global_water_level = 0;

/*****************************************************************************/
void InitSimu(void) {
  static bool init_done = false;

  if (!init_done) {
    init_done = true;
    pthread_mutex_init(&mutex_simu_minepump,0);
  }
}

/*****************************************************************************/
void readhls (int *hls) {
  InitSimu();

  int b;
  pthread_mutex_lock (&mutex_simu_minepump);

  WaterLevel_Value = WaterLevel_Value - CmdPump_Value*4 + 2;
  b = (WaterLevel_Value > 100)?1:0;

  pthread_mutex_unlock (&mutex_simu_minepump);
  *hls = b;
}

/*****************************************************************************/
void readlls(int *lls) {
  InitSimu ();

  int b;

  pthread_mutex_lock (&mutex_simu_minepump);
  b = (WaterLevel_Value > 80)?1:0;
  pthread_mutex_unlock (&mutex_simu_minepump);
  *lls = b;
}

/*****************************************************************************/
void readms(int *ms) {
  InitSimu ();

  static int b = 50;
  static int b_increment = 2;

  pthread_mutex_lock (&mutex_simu_minepump);

  /* If there is no GUI, we simply emulate the methane sensor. The methane
     gets up and down */

  b += b_increment;

  if (b == 100 || b == 40)
    b_increment = - b_increment;

  pthread_mutex_unlock (&mutex_simu_minepump);
  *ms = b;
}

/*****************************************************************************/
void cmdpump(int cmd) {
  InitSimu ();

  pthread_mutex_lock (&mutex_simu_minepump);
  CmdPump_Value = cmd ? 1 : 0;
  printf("Pump %d\n",CmdPump_Value);
  fflush(stdout);
  pthread_mutex_unlock (&mutex_simu_minepump);
}

/*****************************************************************************/
void cmdalarm(int cmd) {
  InitSimu ();

  int c=cmd?1:0; /* c = 0, alarm is off, c = 1, alarm is on */

  pthread_mutex_lock (&mutex_simu_minepump);
  printf("Alarm %d\t",c);
  fflush(stdout);
  pthread_mutex_unlock (&mutex_simu_minepump);
}

// 对应 AADL 中 WaterLevelMonitoring 的 BA 逻辑
void water_level_monitoring_step(int *water_alarm_out) {
    int hls = 0;
    int lls = 0;
    int waterlvl = 0; // 默认值

    // 调用内部模拟函数
    readhls(&hls);

    if (hls == 1) {
        waterlvl = 1;
    } else {
        readlls(&lls);
        if (lls == 0) {
            waterlvl = 0;
        }
    }
    *water_alarm_out = waterlvl;
}

// 对应 AADL 中 MethaneMonitoring 的 BA 逻辑
void methane_monitoring_step(int *methane_level_out) {
    int ms = 0;
    int level = 0;

    readms(&ms);
    if (ms > 100) level = 2;
    else if (ms > 70) level = 1;
    else level = 0;

    *methane_level_out = level;
}

// 对应 AADL 中 PumpCtrl 的 BA 逻辑
void pump_ctrl_step(int methane_level_in, int water_level_in, int *water_alarm_out) {
    int cmd = 0;
    int alarme = 0;
    
    if (methane_level_in == 0) alarme = 0;
    else alarme = 1;

    *water_alarm_out = alarme; // 更新输出

    if (methane_level_in == 2) {
        cmd = 0;
    } else {
        if (water_level_in == 1) cmd = 1;
        else if (water_level_in == 0) cmd = 0;
    }
    
    cmdpump(cmd); // 调用模拟执行器
}

// 对应 AADL 中 WaterAlarm 的 BA 逻辑
void water_alarm_step(int water_alarm_in) {
    cmdalarm(water_alarm_in);
}

void set_methane_level(int val) {
    global_methane_level = val;
}

void set_water_level(int val) {
    global_water_level = val;
}

void run_pump_ctrl_logic(int *water_alarm_out) {
    // 调用之前的逻辑函数，传入全局变量
    pump_ctrl_step(global_methane_level, global_water_level, water_alarm_out);
}