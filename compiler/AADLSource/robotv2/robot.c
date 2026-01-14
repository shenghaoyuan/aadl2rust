#include <stdio.h>
#include "robot.h"

bool b = false;
bool traite_val;

void collecte_donnee(bool *d_source) {
    printf("*** COLLECTE DONNEE *** %d\n", b);
    *d_source = b;
    b = !b;
    fflush(stdout);
}

void traite_in(bool d_info) {
    printf("*** TRAITE *** info=%d\n", d_info);
    traite_val = d_info;
    fflush(stdout);
}

void traite_out(bool *d_ordre) {
    printf("*** TRAITE *** ordre=%d\n", traite_val);
    *d_ordre = traite_val;
    fflush(stdout);
}

void action(bool d_action) {
    printf("*** ACTION *** %d\n", d_action);
    fflush(stdout);
} 