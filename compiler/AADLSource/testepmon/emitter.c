/* testepmon/components/emitter/src/emitter.c */

// #include <camkes.h>
#include <stdio.h>
// #include <sb_types.h>
// #include <sb_emitter_t_impl.h>
#include <stdint.h>
#include <stdbool.h>
#include "emitter.h"

static int64_t port_buffer;
static bool buffer_has_data = false;

static int64_t _value;

void testepmon_emitter_component_init(void)
{
    static bool is_initialized = false;
    if (is_initialized) {
        return;
    }
    printf("testepmon_emitter_component_init called\n");
    _value = 0;
    is_initialized = true;
}

/* control thread: keep calling enqueue for thing
 */
void run_emitter(const int64_t *in_arg)
{
    if (sb_enq_enqueue( &_value ) ) {
        printf("[source] Sent %d\n", _value);
        _value = (_value + 1) % 500;
    } else {
        printf("[source] Unable to send\n");
    }
}

void testepmon_consumer_component_init(void)
{
    printf("testepmon_consumer_component_init called\n");
}


/* Handle monsig notification: there is QueuedData
 */
void testepmon_consumer_s_event_handler(int64_t in_arg)
{
    /* keep dequeuing until no more things can be had
     */
    int64_t value;

    if (sb_deq_dequeue(&value)) {
        printf("[destination] value {%d}\n", value);
    } else {
        printf("[destination] no value consumed.\n");
    }
}

bool sb_enq_enqueue(const int64_t *data) {
    // 空指针检查（鲁棒性）
    if (data == NULL) {
        printf("[source] Error: NULL data to write\n");
        return false;
    }
    // 缓冲区有未读数据时，写入失败
    if (buffer_has_data) {
        printf("[source] Warning: port buffer full, skip write\n");
        return false;
    }
    // 写入数据到缓冲区，标记为有数据
    port_buffer = *data;
    buffer_has_data = true;
    return true; // 写入成功
}

bool sb_deq_dequeue(int64_t *data) {
    // 空指针检查（鲁棒性）
    if (data == NULL) {
        printf("[destination] Error: NULL data to read\n");
        return false;
    }
    // 缓冲区无数据时，读取失败
    if (!buffer_has_data) {
        printf("[destination] Warning: port buffer empty, skip read\n");
        return false;
    }
    // 读取缓冲区数据，标记为无数据
    *data = port_buffer;
    buffer_has_data = false;
    return true; // 读取成功
}