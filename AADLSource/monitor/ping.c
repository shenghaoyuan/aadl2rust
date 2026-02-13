#include <stdio.h>
// #include <po_hi_monitor.h>
#include "ping.h"

int p=0;

void user_do_ping_spg (int *v)
{
  printf ("*** SENDING PING *** %d\n", p);
  *v=p;

  if ((p % 5) == 0)
  {
      // __po_hi_monitor_report_failure_device (device_a_device_id, po_hi_monitor_failure_unknown);
      printf("Simulated Failure Condition (p %% 5 == 0)\n");
  }

  if ((p % 7) == 0)
  {
      // __po_hi_monitor_recover_device (device_a_device_id);
      printf("Simulated Recover Condition (p %% 7 == 0)\n");
  }
  p++;
  fflush (stdout);
}

void user_ping_spg (int i)
{
  printf ("*** PING *** %d\n" ,i);
  fflush (stdout);
}

void recover (void)
{
  printf ("*** RECOVER ACTION ***\n");
  fflush (stdout);
}
