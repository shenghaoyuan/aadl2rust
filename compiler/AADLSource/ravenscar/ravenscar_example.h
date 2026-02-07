#ifndef __RAVENSCAR_EXAMPLE_H__
#define __RAVENSCAR_EXAMPLE_H__

#include <stdint.h>

/* Type Definitions */
typedef int ravenscar_example__workload;
typedef int ravenscar_example__interrupt_counter;

/* Subprogram Declarations */
void event_source_init(void);
void new_external_event(ravenscar_example__interrupt_counter* interrupt_out);

void regular_producer_operation(ravenscar_example__workload* add_load);
void on_call_producer_operation(ravenscar_example__workload load_in);

/* Split Delegate Functions */
void delegate_ext_event_set_depository(ravenscar_example__interrupt_counter depository_in);
void delegate_ext_event_get_interrupt(ravenscar_example__interrupt_counter* interrupt_out);

void on_signal(ravenscar_example__interrupt_counter interrupt_in);

void small_whetstone(int n);

#endif