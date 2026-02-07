// #include <aadl.h>
// #include <stdio.h>

// /* Files generated from AADL model */
// #include <request.h>
// #include <deployment.h>
// #include "types.h"
// #include <po_hi_transport.h>
// #include <po_hi_gqueue.h>

// /******************************************************************************/
// /* In the case of the client-side of the RPC: the corresponding AADL
//  * subprogram is *NOT* connected to AADL thread ports. We interact
//  * directly with the port variables to emulate a RPC synchronous call:
//  * 1/ we send the parameter through the "out_parameter" port
//  * 2/ we block and wait for return parameters through the return_value
//  */

// void rpc_client (__po_hi_task_id self) {

//   static int i = 0;
//   __po_hi_request_t req;
//   __po_hi_local_port_t return_value_port;

//   printf ("[START of RPC]\n");

//   req.port = REQUEST_PORT (client_t, out_parameter);
//   req.PORT_VARIABLE (client_t,out_parameter) = i;

//   __po_hi_gqueue_store_out
//   (self,
//    LOCAL_PORT (client_t,out_parameter),
//    &req);
//   __po_hi_send_output (self,REQUEST_PORT(client_t, out_parameter));

//   printf ("Client thread: sending parameter %d\n", i);

//   __po_hi_gqueue_wait_for_incoming_event(self, &return_value_port);
//   __po_hi_gqueue_get_value(self,return_value_port,&req);
//   __po_hi_gqueue_next_value(self,return_value_port);

//   printf ("Client received: %d\n",
//           req.PORT_VARIABLE(client_t,return_value));

//   i++;
//   printf ("[END of RPC]\n\n");
// }

// /******************************************************************************/
// /* In the case of the server-side RPC: the server function is
//  * connected to the input parameter and return value ports, we take
//  * advantage of port-to-parameter conenctions to hook the server code
//  * directly
//  */

// void rpc_server
//     (rpc__alpha_type in_parameter,
//      rpc__alpha_type* return_value)
// {
//   printf ("Server thread: received %d\n", in_parameter);
//   *return_value = in_parameter + 1;
//   printf ("Server thread: sending return value %d\n", *return_value);
// }

#include <stdio.h>
#include <stdint.h>
#include <unistd.h>
#include "rpc.h"

typedef int32_t rpc__alpha_type;

// Static storage to hold data between Read and Write phases
static rpc__alpha_type server_store = 0;
static rpc__alpha_type client_counter = 0;

/******************************************************************************/
/* Client Side */

// 1. Send Request: Generates a value and outputs it
void client_send_request(rpc__alpha_type* out_parameter) {
    sleep(1);
    client_counter++;
    *out_parameter = client_counter;
    printf("[Client] 1. Sending Request: %d\n", (int)*out_parameter);
}

// 2. Read Response: Receives the processed value
void client_read_response(rpc__alpha_type in_parameter) {
    printf("[Client] 2. Received Response: %d\n", (int)in_parameter);
    printf("----------------------------------------\n");
}

/******************************************************************************/
/* Server Side */

// 1. Read Request: Receives value and computes
void server_read_request(rpc__alpha_type in_parameter) {
    printf("[Server] 1. Received Request: %d\n", (int)in_parameter);
    // Simple computation: Add 1
    server_store = in_parameter + 1;
}

// 2. Send Response: Outputs the computed value
void server_send_response(rpc__alpha_type* return_value) {
    *return_value = server_store;
    printf("[Server] 2. Sending Response: %d\n", (int)*return_value);
}