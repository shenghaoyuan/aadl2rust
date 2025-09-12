#ifndef USER_JOBS_H
#define USER_JOBS_H

#ifdef __cplusplus
extern "C" {
#endif

#include <stdint.h>
#include <stdlib.h>
#include <stdio.h>

/* 函数声明 */
void compute_during_1ms(int n);

void user_read(int* value);
void user_update(int* value);

void user_gnc_job(void);
void user_tmtc_job(void);

void user_gnc_identity(void);
void user_tmtc_identity(void);

/* 全局变量声明 */
extern int gnc_welcome;
extern int tmtc_welcome;

#ifdef __cplusplus
}
#endif

#endif // USER_JOBS_H