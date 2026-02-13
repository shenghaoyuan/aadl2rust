// #include <camkes.h>
#include <stdio.h>
#include <assert.h>
// #include <sb_Comp_A_Impl.h>
#include "comp.h"

typedef signed int int32_t;

int32_t t = 0;

void Comp_A_time_triggered(int32_t *arg){
  *arg = t;
  printf("Comp_A_time_triggered invoked.  Sending %i to Comp_B\n", t);
  t++;  
}

void Comp_B_input(int32_t in_arg){
  printf("Comp_B_input received event %i\n", in_arg);
}