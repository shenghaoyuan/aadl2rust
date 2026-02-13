#include <stdio.h>
#include "blink.h"

static bool Blink_On = true;

void All_LEDs_On(void) {
    printf("[STM32] All LEDs ON\n");
}

void All_LEDs_Off(void) {
    printf("[STM32] All LEDs OFF\n");
}

void Do_Blink(void) {
    if (Blink_On) {
        All_LEDs_On();
    } else {
        All_LEDs_Off();
    }
    
    Blink_On = !Blink_On;
}