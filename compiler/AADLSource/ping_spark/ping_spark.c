#include <stdio.h>
#include "ping_spark.h"

static C_Simple_Type global_var = 0;

void user_welcome_pinger(void)
{
    printf("Hello! This is the pinger thread starting\n");
}

void user_recover(void)
{
    printf("Could not send message ! ***\n");
}

void user_do_ping_spg(C_Simple_Type *data_source)
{
    static int initialized = 0;
    if (!initialized) {
        user_welcome_pinger();
        initialized = 1;
    }

    if (global_var > 100) {
        global_var = 0;
    }

    global_var = global_var + 1;

    *data_source = global_var;
    printf("Sending ORDER: %d\n", global_var);
}

void user_pinged_spg(C_Simple_Type data_sink)
{
    printf("*** PING *** %d\n", data_sink);
}