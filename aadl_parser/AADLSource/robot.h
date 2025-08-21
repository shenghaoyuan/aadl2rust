#ifndef __ROBOT_H_
#define __ROBOT_H_
#include <stdint.h>
#include <stdbool.h>

typedef bool Alpha_Type;

void collecte_donnee(Alpha_Type *d_source);
void traite(Alpha_Type d_info, Alpha_Type *d_ordre);
void action(Alpha_Type d_action);

#endif 