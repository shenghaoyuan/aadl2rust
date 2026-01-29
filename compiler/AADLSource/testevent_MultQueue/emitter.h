#ifndef EMITTER_H
#define EMITTER_H

typedef long long int int64_t;
typedef int bool;

void testevent_emitter_component_init(const int64_t *in_arg);
void run_emitter();
void testevent_consumer_component_init(const int64_t *in_arg);
void testevent_consumer_s_event_handler();
void sb_e_enqueue();
bool sb_s_dequeue();

#endif