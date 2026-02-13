#ifndef __PING_SPARK_H__
#define __PING_SPARK_H__

typedef int C_Simple_Type;
void user_do_ping_spg(C_Simple_Type *data_source);
void user_pinged_spg(C_Simple_Type data_sink);
void user_welcome_pinger(void);
void user_recover(void);

#endif