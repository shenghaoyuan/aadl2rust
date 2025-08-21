#include <stdio.h>
#include "robot.h"

void collecte_donnee(Alpha_Type *d_source) {
    printf("*** COLLECTE DONNEE *** %d\n", *d_source);
    fflush(stdout);
}

void traite(Alpha_Type d_info, Alpha_Type *d_ordre) {
    printf("*** TRAITE *** info=%d, ordre=%d\n", d_info, *d_ordre);
    fflush(stdout);
}

void action(Alpha_Type d_action) {
    printf("*** ACTION *** %d\n", d_action);
    fflush(stdout);
} 