// #include<stdio.h>
// #include<types.h>

// void print_spg
// (producer__consumer__alpha_type a_data_in) {

//   printf("%d\n", a_data_in);
//   fflush(stdout);
// }

// void consume_spg (producer__consumer__alpha_type data)
// {
//    printf ("Consume %d\n", data);
//    fflush(stdout);
// }

#include <stdio.h>
#include "producer_consumer.h"

void produce_spg(producer__consumer__alpha_type* data_source) {
    static int counter = 1;
    *data_source = counter;
    print_spg(*data_source);
    counter++;
}

void print_spg(producer__consumer__alpha_type a_data_in) {
  printf("Produced: %d\n", a_data_in);
  fflush(stdout);
}

void consume_spg(producer__consumer__alpha_type data) {
   printf("Consumed: %d\n", data);
   fflush(stdout);
}