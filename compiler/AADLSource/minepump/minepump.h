#ifndef _MINEPUMP_H_
#define _MINEPUMP_H_

#ifndef BYTE
#define BYTE unsigned char
#endif

void InitSimu(void);
BYTE ReadHLS(void);
BYTE ReadLLS(void);
BYTE ReadMS(void);
void CmdPump(BYTE cmd);
void CmdAlarm(BYTE cmd);

void water_level_monitoring(int* water_alarm);
void methane_monitoring(int* methane_level);

void pump_ctrl_read_methane(int methane_level);
void pump_ctrl_read_water(int water_level);
void pump_ctrl_logic(int* water_alarm);

void water_alarm_task(int water_alarm);

#endif /* _MINEPUMP_H_ */