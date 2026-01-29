/* testsubprogram/components/sender/src/sender */

// #include <camkes.h>
// #include <sb_types.h>
// #include "../includes/sb_sender_impl.h"
#include <stdio.h>
#include <stdint.h>
#include "sender.h"

static int64_t counter = 0;

// void sender_init(const int64_t *arg){
//    printf("Initializer method for sender invoked\n");
// }

// void run_sender(int64_t * arg) {
//    uint32_t result;

//    operations_add(10, 5, &result);
//    printf("Result of 'add' call to receiver with arguments 10, 5 : (%d) \n", result);
   
//    operations_subtract(10, 5, &result);  
//    printf("Result of 'subtract' call to receiver with arguments 10, 5 : (%d) \n", result);
// }

// void operations_add(uint32_t A, uint32_t B, uint32_t *result) {
// 	*result = A + B;
// }

// void operations_subtract(uint32_t A, uint32_t B, uint32_t *result) {
// 	*result = A - B;
// }

void run_sender(int64_t *result) {
   counter++;
   *result = counter;
   printf("[Sender] Calculated value: %ld\n", *result);
   fflush(stdout);
}

void run_receiver(int64_t input) {
   printf("[Receiver] Received value: %ld\n", input);
   int64_t added = input + 5;
   int64_t subbed = input - 2;
   printf("           (Logic: %ld+5=%ld, %ld-2=%ld)\n", input, added, input, subbed);
   
   fflush(stdout);
}