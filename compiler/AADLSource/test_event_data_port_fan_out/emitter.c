// #include <camkes.h>
#include <stdio.h>
// #include <sb_types.h>
// #include <sb_emitter_impl.h>
#include <stdint.h>
#include <stdbool.h>
#include "emitter.h"

#define NUM_CONSUMERS 4

// 为每个消费者定义一个简单的环形队列
typedef struct {
    int64_t buffer[10]; // 假设最大深度为10，覆盖测试用例中的5
    int head;
    int tail;
    int count;
    int max_size;
    const char* name;
} Queue;

// 初始化 4 个队列，对应: default, 2_A, 2_B, 5
static Queue queues[NUM_CONSUMERS] = {
    { .max_size = 10, .name = "consumer_default" }, // Default
    { .max_size = 2,  .name = "consumer_2_A" },     // Size 2
    { .max_size = 2,  .name = "consumer_2_B" },     // Size 2
    { .max_size = 5,  .name = "consumer_5" }        // Size 5
};

// 辅助函数：入队
bool enqueue(int queue_idx, int64_t data) {
    Queue *q = &queues[queue_idx];
    if (q->count >= q->max_size) {
        return false; // 队列满，丢弃
    }
    q->buffer[q->tail] = data;
    q->tail = (q->tail + 1) % 10; // 物理大小固定为10
    q->count++;
    return true;
}

// 辅助函数：出队
bool dequeue(int queue_idx, int64_t *data) {
    Queue *q = &queues[queue_idx];
    if (q->count == 0) {
        return false; // 队列空
    }
    *data = q->buffer[q->head];
    q->head = (q->head + 1) % 10;
    q->count--;
    return true;
}

static int64_t counter = 0;

// 0: emitter, 1: default, 2: 2_A, 3: 2_B, 4: 5
static int current_consumer_id = 1; 

void test_event_data_port_emitter_component_init(const int64_t *in_arg) {
  printf("[emitter] Init called\n");
}

void run_emitter(const int64_t *in_arg) {
  printf("---------------------------------------\n");
  printf("[emitter] Sending sequence of %ld events...\n", counter);

  for(int64_t val = 1; val <= counter; val++) {
    // 模拟广播：向所有4个队列写入数据
    for (int i = 0; i < NUM_CONSUMERS; i++) {
        enqueue(i, val);
    }
  }
  
  counter = (counter + 1) % 7; // 0 到 6 循环
}

void test_event_data_port_consumer_component_init(const int64_t *in_arg) {
    printf("[consumer] Init called\n");
}

void test_event_data_port_consumer_s_event_handler(int64_t in_arg)
{
    // 获取当前模拟的消费者 ID (1~4)
    int q_idx = current_consumer_id - 1;
    Queue *q = &queues[q_idx];

    printf("[%s] Event Handler Triggered\n", q->name);

    int64_t value;
    int received_count = 0;

    // 尝试从自己的队列中读取所有数据
    while(dequeue(q_idx, &value)) {
        printf("[%s] Dequeued value: %ld\n", q->name, value);
        received_count++;
    }

    if (received_count == 0) {
        printf("[%s] No data (Queue empty or not scheduled)\n", q->name);
    } else {
        printf("[%s] Total processed: %d\n", q->name, received_count);
    }

    // 轮转到下一个消费者，以便下次调用时模拟另一个线程
    current_consumer_id++;
    if (current_consumer_id > NUM_CONSUMERS) {
        current_consumer_id = 1;
    }
}