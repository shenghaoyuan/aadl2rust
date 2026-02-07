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

void test_data_port_periodic_source_component_init(const int64_t *in_arg) {
  printf("[%s] test_data_port_periodic_source_component_init called\n", get_instance_name());
  _value = 0;
}

void test_data_port_periodic_source_component_time_triggered(const int64_t *arg) {
  if (sb_write_port_write( &_value ) ) {
    printf("---------------------------------------\n");
    printf("[Source] Sent %d\n", _value );
    _value = (_value + 1) % 500;
  }
}

void test_data_port_periodic_destination_component_init(const int64_t *arg) {
  printf("[%s] test_data_port_periodic_destination_component_init called\n", get_instance_name());
}

void test_data_port_periodic_destination_component_time_triggered(int64_t arg) {
  int64_t value;

  if(sb_read_port_read(&value)){
    printf("[Destination1] value {%d}\n", value);
  }
}

void test_data_port_periodic_destination_component_time_triggered2(int64_t arg) {
  int64_t value;

  if(sb_read_port_read(&value)){
    printf("[Destination2] value {%d}\n", value);
  }
}

bool sb_write_port_write(const int64_t *data) {
    // 空指针检查（鲁棒性）
    if (data == NULL) {
        return false;
    }
    // 缓冲区有未读数据时，写入失败
    if (buffer_has_data) {
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
        return false;
    }
    // 缓冲区无数据时，读取失败
    if (!buffer_has_data) {
        return false;
    }
    // 读取缓冲区数据，标记为无数据
    *data = port_buffer;
    buffer_has_data = false;
    return true; // 读取成功
}