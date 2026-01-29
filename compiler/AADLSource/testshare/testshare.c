/* testshare/components/publisher/src/publisher.c */

#include <stdio.h>
#include <stdint.h>
// #include <camkes.h>
// #include <sb_types.h>
// #include <sb_publisher_impl.h>
// #include <sb_subscriber_impl.h>
#include "testshare.h"

static int8_t _value;

// 定义数据结构 (对应 AADL 中的 Thing_t.impl)
typedef struct {
    int8_t lepht;
    int8_t right;
    int8_t top;
    int8_t bottom;
} Thing_t;

// 模拟共享内存
// 在实际系统中，这块内存由系统分配并映射。
// 在这里，我们声明一个静态实例，让两个指针都指向它。
static Thing_t shared_memory_storage;

// 模拟 AADL features 中的 data access 端口
// b1 是 publisher 的接口，b2 是 subscriber 的接口
static Thing_t *b1 = &shared_memory_storage;
static Thing_t *b2 = &shared_memory_storage;

void testshare_publisher_component_init(void) {
  printf("testshare_publisher_component_init called\n");
  _value = 0;
}

void run_publisher(void)
{
    printf("[publisher] starting\n");
    b1->lepht = _value;
    b1->right = _value + 1;
    b1->top   = _value + 2;
    b1->bottom = _value + 3;
    b1_release(); /* release memory fence */
    printf("[publisher] wrote b1={%d,%d,%d,%d}\n",
           b1->lepht, b1->right, b1->top, b1->bottom );
    _value = (_value + 4) % 500;
}

void testshare_subscriber_component_init(void) {
    printf("testshare_subscriber_component_init called\n");
    printf("[subscriber] starting--poll for nonzero thing_t\n");
}

void run_subscriber(void)
{
    b2_acquire();  /* acquire memory fence */
    if (b2->bottom) {
        printf("[subscriber] b2={%d,%d,%d,%d}\n",
               b2->lepht, b2->right, b2->top, b2->bottom );
    }
}

// b1_release 通常用于在写入数据后刷新缓存或确保顺序
void b1_release(void) {
    // 在简单的本地模拟中，这里什么都不用做，或者可以打印日志
    // printf("[System] Memory fence: b1 release\n");
}

// b2_acquire 通常用于在读取数据前确保缓存一致性
void b2_acquire(void) {
    // 在简单的本地模拟中，这里什么都不用做
    // printf("[System] Memory fence: b2 acquire\n");
}