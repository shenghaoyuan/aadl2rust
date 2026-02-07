#include <stdio.h>
#include <unistd.h>
#include <sys/time.h>
#include "functions.h"

static int step1_data = 0;
static int step2_data = 0;
static int step3_data = 0;

void sensor_emulator (int* value)
{
   struct timeval mytime;
   gettimeofday (&mytime, NULL);
   *value = ((mytime.tv_sec % 10) * 1000) + (mytime.tv_usec / 1000);
   printf( "I'm the sensor, I send value %d\n", *value);
}

void actuator_emulator (int value)
{
   struct timeval mytime;
   int val;

   gettimeofday (&mytime, NULL);
   val = ((mytime.tv_sec % 10) * 1000) + (mytime.tv_usec / 1000);
   printf("I'm actuator, I received value %d, current time=%d\n", value, val);
}

// void spg1 (int ined, int* outed)
// {
//    struct timeval mytime;
//    int val;
//    gettimeofday (&mytime, NULL);
//    val = ((mytime.tv_sec % 10) * 1000) + (mytime.tv_usec / 1000);
//    printf ("I'm program 1, I received value %d, current sended time=%d\n", ined, val);
//    *outed = val;
// }
void spg1_in (int ined)
{
   struct timeval mytime;
   int val;
   gettimeofday (&mytime, NULL);
   val = ((mytime.tv_sec % 10) * 1000) + (mytime.tv_usec / 1000);
   printf ("I'm program 1 (in), I received value %d, current time=%d\n", ined, val);
   step1_data = val; // 存储到全局变量，供out函数使用
}

void spg1_out (int* outed)
{
   if (outed == NULL) return;
   *outed = step1_data; // 从全局变量读取数据
   printf ("I'm program 1 (out), I send value %d\n", *outed);
}

// void spg2 (int ined, int* outed)
// {
//    struct timeval mytime;
//    int val;
//    gettimeofday (&mytime, NULL);
//    val = ((mytime.tv_sec % 10) * 1000) + (mytime.tv_usec / 1000);
//    printf ("I'm program 2, I received value %d, current sended time=%d\n", ined, val);
//    *outed = val;
// }
void spg2_in (int ined)
{
   struct timeval mytime;
   int val;
   gettimeofday (&mytime, NULL);
   val = ((mytime.tv_sec % 10) * 1000) + (mytime.tv_usec / 1000);
   printf ("I'm program 2 (in), I received value %d, current time=%d\n", ined, val);
   step2_data = val;
}

void spg2_out (int* outed)
{
   if (outed == NULL) return;
   *outed = step2_data;
   printf ("I'm program 2 (out), I send value %d\n", *outed);
}

// void spg3 (int ined, int* outed)
// {
//    struct timeval mytime;
//    int val;
//    gettimeofday (&mytime, NULL);
//    val = ((mytime.tv_sec % 10) * 1000) + (mytime.tv_usec / 1000);
//    printf ("I'm program 3, I received value %d, current sended time=%d\n", ined, val);
//    *outed = val;
// }
void spg3_in (int ined)
{
   struct timeval mytime;
   int val;
   gettimeofday (&mytime, NULL);
   val = ((mytime.tv_sec % 10) * 1000) + (mytime.tv_usec / 1000);
   printf ("I'm program 3 (in), I received value %d, current time=%d\n", ined, val);
   step3_data = val;
}

void spg3_out (int* outed)
{
   if (outed == NULL) return;
   *outed = step3_data;
   printf ("I'm program 3 (out), I send value %d\n", *outed);
}