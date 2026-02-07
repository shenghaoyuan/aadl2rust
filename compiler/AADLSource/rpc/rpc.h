#ifndef RPC_H
#define RPC_H

#include <stdint.h>

// Type definition compatible with AADL Integer
typedef int32_t rpc__alpha_type;

// Client functions
void client_send_request(rpc__alpha_type* out_parameter);
void client_read_response(rpc__alpha_type in_parameter);

// Server functions
void server_read_request(rpc__alpha_type in_parameter);
void server_send_response(rpc__alpha_type* return_value);

#endif