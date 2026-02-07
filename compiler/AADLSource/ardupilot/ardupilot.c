#include <stdio.h>
#include <stdlib.h>
#include <math.h>
#include <time.h>
#include "ardupilot.h"

#ifdef _WIN32
#include <windows.h>
#else
#include <sys/time.h>
#endif

#define SCALING_FACTOR 1000000.0f

/* ========================================================
   Helper: Time
   ======================================================== */
uint64_t millis()
{
#ifdef _WIN32
    SYSTEMTIME st;
    GetSystemTime(&st);
    return (uint64_t)(st.wSecond * 1000 + st.wMilliseconds);
#else
    struct timeval tv;
    gettimeofday(&tv, NULL);
    return (uint64_t)(tv.tv_sec * 1000 + tv.tv_usec / 1000);
#endif
}

/* ========================================================
   Component: GPS
   ======================================================== */
float glob_longitude = 1.4;
float glob_latitude = 2.5;
int glob_altitude = 3;

void gps_simulation_get_lat(ardupilot__base_types__float* lat) {
    /* Convert float to int with scaling */
    *lat = (int)(glob_latitude * SCALING_FACTOR);
    printf("[GPS] Simulate latitude %f (encoded: %d)\n", glob_latitude, *lat);
}

void gps_simulation_get_lon(ardupilot__base_types__float* lon) {
    /* Convert float to int with scaling */
    *lon = (int)(glob_longitude * SCALING_FACTOR);
    printf("[GPS] Simulate longitude %f (encoded: %d)\n", glob_longitude, *lon);
}

void gps_simulation_get_alt(ardupilot__base_types__integer* alt) {
    *alt = glob_altitude;
}

void gps_backdoor(float yaw)
{
  glob_longitude += sin(yaw)/20.0;
  glob_latitude += cos(yaw)/20.0;
}

/* ========================================================
   Component: Throttle
   ======================================================== */
void throttle_simulation(ardupilot__base_types__integer speed)
{
   printf("[THROTTLE] received speed=%d\n", speed);
}

/* ========================================================
   Component: YAW
   ======================================================== */
void yaw_simulation(ardupilot__base_types__integer angle)
{
   printf("[YAW] received angle %d\n", angle);
}

/* ========================================================
   Component: Flight Management
   ======================================================== */

#define NB_WAYPOINTS 6

#define HEADING_MAX 15
#define HEADING_MIN -15

#define ALTITUDE_MAX 40
#define ALTITUDE_MIN -45

#define REVERSE_YAW 1 

#define DISTANCE_LIMIT 4000

#define KP_HEADING 10.0
#define KI_HEADING .01
#define KD_HEADING 0.001

#define KP_ALTITUDE 4.0
#define KI_ALTITUDE 0.001
#define KD_ALTITUDE 2.0

#define LAUNCH_ALTITUDE 0 

#define RAD_TO_DEG 57.295779513082320876798154814105
#define degrees(rad) ((rad)*RAD_TO_DEG)
#define constrain(amt,low,high) ((amt)<(low)?(low):((amt)>(high)?(high):(amt)))

uint8_t current_wp=1;

float wp_lat[NB_WAYPOINTS+1];
float wp_lon[NB_WAYPOINTS+1];
int   wp_alt[NB_WAYPOINTS+1];

float wp_distance=0.0;   
int   middle_yaw=90;     
int   middle_thr=90;     

int heading_previous_error; 
float heading_I;             
int altitude_previous_error;
float altitude_I;            

float wp_bearing=0.0;       
int course=0;            

const float  kp[]={KP_HEADING,KP_ALTITUDE};	
const float  ki[]={KI_HEADING,KI_ALTITUDE};	 
const float  kd[]={KD_HEADING,KD_ALTITUDE};

/* Static storage for split execution */
static float g_mgmt_lat;
static float g_mgmt_lon;
static int g_mgmt_alt;

static int g_computed_speed = 0;
static int g_computed_angle = 0;

int PID_altitude(int PID_set_Point, int PID_current_Point)
{
  static unsigned int altitude_PID_timer;
  static float altitude_D;
  static int altitude_output;

  int PID_error=0;

  float dt=(float)(millis()-altitude_PID_timer)/1000; 

  PID_error=PID_set_Point-PID_current_Point;

  altitude_I+= (float)PID_error*dt; 
  altitude_I=constrain(altitude_I,20,-20); 

  altitude_D=(float)((float)PID_error-(float)altitude_previous_error)/((float)dt);

  altitude_output= (kp[1]*PID_error);  
  altitude_output+=(ki[1]*altitude_I); 
  altitude_output+= (kd[1]*altitude_D);

  altitude_output = constrain(altitude_output,ALTITUDE_MIN,ALTITUDE_MAX);

  altitude_previous_error=PID_error;
  altitude_PID_timer=millis();

  return altitude_output;
}

void flt_mgmt_init()
{
   wp_distance = 0;
   middle_thr  = 90;
   middle_yaw  = 90;

   wp_lat[1]=  34.982613;
   wp_lon[1]= -118.443357; 
   wp_alt[1]=50;

   wp_lat[2]= 34.025136;
   wp_lon[2]=-118.445254; 
   wp_alt[2]=100;

   wp_lat[3]=34.018287;
   wp_lon[3]=-118.456048; 
   wp_alt[3]=100;

   wp_lat[4]= 34.009332;
   wp_lon[4]=-118.467672; 
   wp_alt[4]=50;

   wp_lat[5]=  34.006476;
   wp_lon[5]=-118.465413; 
   wp_alt[5]=50;

   wp_lat[6]=  34.009927;
   wp_lon[6]= -118.458320; 
   wp_alt[6]= 20;
}

float PID_heading(int PID_error)
{ 
  static unsigned int heading_PID_timer;
  static float heading_D; 
  static float heading_output;

  float dt=(float)(millis()-heading_PID_timer)/1000;

  heading_I+= (float)PID_error*(float)dt; 
  heading_I=constrain(heading_I,HEADING_MIN,HEADING_MAX); 

  heading_D=((float)PID_error-(float)heading_previous_error)/(float)dt;

  heading_output=0.0;
  heading_output=(kp[0]*PID_error);
  heading_output+= (ki[0]*heading_I);
  heading_output+= (kd[0]*heading_D);

  heading_output = constrain(heading_output,HEADING_MIN,HEADING_MAX);

  heading_previous_error=PID_error;
  heading_PID_timer=millis();

  // printf("PID_heading %f\n", heading_output);
  if(REVERSE_YAW == 1)
    return (-1*heading_output); 
  else
    return (heading_output);
}

int compass_error(int PID_set_Point, int PID_current_Point)
{
   float PID_error=0;
   
   if(fabs(PID_set_Point-PID_current_Point) > 180) 
   {
      if(PID_set_Point-PID_current_Point < -180)
         PID_error=(PID_set_Point+360)-PID_current_Point;
      else
         PID_error=(PID_set_Point-360)-PID_current_Point;
   }
   else
   {
      PID_error=PID_set_Point-PID_current_Point;
   }
   // printf("compass_error %f\n", PID_error);
   return (int)PID_error;
}

float get_gps_dist(float flat1, float flon1, float flat2, float flon2)
{
  float x = 69.1 * (flat2 - flat1); 
  float y = 69.1 * (flon2 - flon1) * cos(flat1/57.3);

  return (float) sqrt((float)(x*x) + (float)(y*y))*1609.344; 
}

int get_gps_course(float flat1, float flon1, float flat2, float flon2)
{
  float calc;
  float bear_calc;

  float x = 69.1 * (flat2 - flat1); 
  float y = 69.1 * (flon2 - flon1) * cos(flat1/57.3);

  calc = atan2(y,x);
  bear_calc = degrees(calc);

  if(bear_calc<=1){
    bear_calc=360+bear_calc; 
  }
  return (int)bear_calc;
}

/* Split implementation */

void flt_mgmt_set_lat(ardupilot__base_types__float lat) {
    /* Restore float from int */
    g_mgmt_lat = (float)lat / SCALING_FACTOR;
}

void flt_mgmt_set_lon(ardupilot__base_types__float lon) {
    /* Restore float from int */
    g_mgmt_lon = (float)lon / SCALING_FACTOR;
}

void flt_mgmt_set_alt(ardupilot__base_types__integer alt) {
    g_mgmt_alt = alt;
}

void flt_mgmt_compute(void)
{
    int altitude = g_mgmt_alt; 
    float latitude = g_mgmt_lat;
    float longitude = g_mgmt_lon;

    wp_bearing = get_gps_course(latitude, longitude, wp_lat[current_wp], wp_lon[current_wp]);
    wp_distance=get_gps_dist(latitude, longitude, wp_lat[current_wp], wp_lon[current_wp]);

    printf("bearing %f distance %f\n", wp_bearing, wp_distance);

    g_computed_angle = middle_yaw + PID_heading(compass_error(45, (int)wp_bearing)); 
  
    if(get_gps_dist(latitude, longitude, wp_lat[0], wp_lon[0]) > DISTANCE_LIMIT) 
        current_wp=0; 

    gps_backdoor((float)wp_bearing);

    g_computed_speed = PID_altitude(wp_alt[current_wp], (altitude-LAUNCH_ALTITUDE));
    g_computed_speed = constrain(g_computed_speed, 45, 100);

    printf("[MGMT] received lat=%f, long=%f, simulate speed=%d, angle=%d\n", latitude, longitude, g_computed_speed, g_computed_angle);
    printf("Go to WP %d: lat %f lon %f\n", current_wp, wp_lat[current_wp], wp_lon[current_wp]);
  
    if(wp_distance<30)
    {
        printf("Touchdown !!!!!\n");
        current_wp++; 
        if(current_wp>NB_WAYPOINTS)
        {
            current_wp=0; 
        }
    }
}

void flt_mgmt_get_speed(int* speed)
{
    *speed = g_computed_speed;
}

void flt_mgmt_get_angle(int* angle)
{
    *angle = g_computed_angle;
}