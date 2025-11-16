#ifndef __ROBOT_H_
#define __ROBOT_H_
#include <stdint.h>
#include <stdbool.h>

void collecte_donnee(bool *d_source);
void traite(bool d_info, bool *d_ordre);
void action(bool d_action);

#endif 