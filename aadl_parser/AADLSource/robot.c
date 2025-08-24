#include <stdio.h>
#include "robot.h"

bool b = false;

void collecte_donnee(bool *d_source) {
    printf("*** COLLECTE DONNEE *** %d\n", b);
    *d_source = b;
    b = !b;
    fflush(stdout);
}

void traite(bool d_info, bool *d_ordre) {
    printf("*** TRAITE *** info=%d, ordre=%d\n", d_info, *d_ordre);
    fflush(stdout);
}

void action(bool d_action) {
    printf("*** ACTION *** %d\n", d_action);
    fflush(stdout);
} 