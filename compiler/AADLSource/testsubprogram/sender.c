/* testsubprogram/components/sender/src/sender */

// #include <camkes.h>
// #include <sb_types.h>
// #include "../includes/sb_sender_impl.h"
#include <stdio.h>
#include <stdint.h>
#include <stdbool.h>
#include "sender.h"

static testsubprogram__operands_impl global_ops;

void sender_init(void){
   static bool is_init = false;
   if (is_init) 
   {
      return;
   }
   is_init = true;
   printf("Initializer method for sender invoked\n");
}

// void run_sender(void) {
//    uint32_t result;

//    operations_add(10, 5, &result);
//    printf("Result of 'add' call to receiver with arguments 10, 5 : (%d) \n", result);
   
//    operations_subtract(10, 5, &result);  
//    printf("Result of 'subtract' call to receiver with arguments 10, 5 : (%d) \n", result);
// }

void run_sender(void) {
   global_ops.A = 10;
   global_ops.B = 5;
   global_ops.result = 0;

   operations_add();
   
   printf("Result of 'add' call to receiver with arguments %u, %u : (%u) \n", 
          global_ops.A, global_ops.B, global_ops.result);
   
   global_ops.A = 10;
   global_ops.B = 5;
   global_ops.result = 0; 
   
   operations_subtract();
   
   printf("Result of 'subtract' call to receiver with arguments %u, %u : (%u) \n", 
          global_ops.A, global_ops.B, global_ops.result);
}

// void operations_add(uint32_t A, uint32_t B, uint32_t *result) {
// 	*result = A + B;
// }

// void operations_subtract(uint32_t A, uint32_t B, uint32_t *result) {
// 	*result = A - B;
// }

void operations_add(void) {
    global_ops.result = global_ops.A + global_ops.B;
}

void operations_subtract(void) {
    global_ops.result = global_ops.A - global_ops.B;
}