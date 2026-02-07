/* testpdmon/components/source/src/source.c */

// #include <camkes.h>
#include <stdio.h>
// #include <sb_types.h>
// #include <sb_source_t_impl.h>
#include <stdint.h>
#include <stdbool.h>
#include "source.h"

static int64_t port_buffer;
static bool buffer_has_data = false;

static int64_t _value;

void testdpmon_source_component_init(void)
{
    static bool is_initialized = false;
    if (is_initialized) {
        return;
    }
    printf("testdpmon_source_component_init called\n");
    _value = 0;
    is_initialized = true;
}

/* control thread: keep calling enqueue for thing
 */
void run_sender(const int64_t *in_arg)
{
    if (sb_write_port_write( &_value ) ) {
        printf("[source] Sent %d\n", _value );
        _value = (_value + 1) % 500;
    }
}

void testdpmon_destination_component_init(void)
{
    printf("testdpmon_destination_component_init called\n");
}

/* Handle monsig notification: there is QueuedData
 */
void run_receiver(int64_t in_arg)
{
    /* keep dequeuing until no more things can be had
     */
    int64_t value;

    if(sb_read_port_read(&value)){
        printf("[destination] value {%d}\n", value);
    }
}

bool sb_write_port_write(const int64_t *data) {
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

bool sb_read_port_read(int64_t *data) {
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