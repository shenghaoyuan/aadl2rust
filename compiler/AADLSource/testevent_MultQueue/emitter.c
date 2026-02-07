/* testevent/components/emitter/src/emitter.c */

// #include <camkes.h>
#include <stdio.h>
// #include <sb_types.h>
// #include <sb_emitter_impl.h>
#include <stdint.h>
#include <stdbool.h>
#include "emitter.h"

static int32_t counter = 0;
#define EVENT_QUEUE_MAX_SIZE 5
static int32_t current_event_count = 0;

void testevent_emitter_component_init(void)
{
  static bool is_initialized = false;
    if (is_initialized) {
        return;
    }
  printf("testevent_emitter_component_init called\n");
  is_initialized = true;
}

/* control thread: keep calling enqueue for thing
 */
void run_emitter(){
  int numEvents = counter % 7; // send 0 - 6 events per dispatch, consumer's queue size is 5
  for(int32_t i = 0; i < numEvents; i++) {
    sb_e_enqueue();
  }
  printf("[Emitter] Sent %i events.\n", numEvents);
   
  counter++;
}

void testevent_consumer_component_init(void) {
  static bool is_initialized = false;
    if (is_initialized) {
        return;
    }
  printf("testevent_consumer_component_init called\n");
  is_initialized = true;
}

void testevent_consumer_s_event_handler() {
  int32_t receivedEvents = 0; // 1 for the event that caused handler to be invoked
  while(sb_s_dequeue()) {
    receivedEvents++;
  }
  
  printf("[Consumer] received %i events\n\n", receivedEvents);
}

void sb_e_enqueue() {
    // 检查队列是否已满（不超过最大容量）
    if (current_event_count >= EVENT_QUEUE_MAX_SIZE) {
        printf("[Emitter] Warning: Event queue full, skip enqueue!\n");
        return;
    }
    // 队列未满，事件入队，当前事件数+1
    current_event_count++;
}

bool sb_s_dequeue() {
    // 检查队列是否有事件
    if (current_event_count <= 0) {
        return false; // 无事件可出队，返回false
    }
    // 有事件，出队，当前事件数-1
    current_event_count--;
    return true; // 出队成功，返回true
}