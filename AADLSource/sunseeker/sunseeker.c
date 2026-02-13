#include <stdio.h>
#include <stdint.h>
#include "sunseeker.h"

// float controller_transfer = 0.0;
// float reference_input = 0.0;

// float clock = 0.0;
// const float period = 0.01;

// float plant_integrator = 0.0;
// float plant_transfer_fcn = 0.0;

// float plant_period = 0.01;

// void user_sunseekercontroller(float *controllerinput, float outputfeedback)
// {
//         float error;
//         float gain_error_1;
//         float gain_error;
//         float transfer_fcn_update;

//         printf("CONTROLLER INPUT %f\n", outputfeedback);

//         if (clock < 1.0)
//         {
//                 reference_input = 0.0;
//         }
//         else
//         {
//                 reference_input = clock - 1.0;
//         }

//         error = reference_input - outputfeedback;

//         gain_error_1 = error * 0.1;
//         gain_error = gain_error_1 * (-10000.0);

//         transfer_fcn_update = gain_error - 170.0 * controller_transfer;

//         *controllerinput = 29.17 * controller_transfer + transfer_fcn_update;

//         controller_transfer = controller_transfer + period * transfer_fcn_update;

//         clock = clock + period;

//         printf("CONTROLLER OUTPUT %f\n", *controllerinput);
// }

// void user_sunseekerplant(float controller_input, float *outputfeedback)
// {
//         float feedback_error;
//         float feedback;
//         float integrator_output;
//         float plant_output;
//         float preamp_output;
//         float transfer_fcn_update;

//         printf("PLANT INPUT: %f\n", controller_input);
//         fflush(stdout);
//         preamp_output = controller_input * (-2.0);
//         integrator_output = plant_integrator;
//         plant_output = 0.002 * plant_transfer_fcn;
//         feedback = plant_output * 0.0125;

//         *outputfeedback = integrator_output * 0.00125;
//         plant_integrator = plant_integrator + 0.001 * plant_output;

//         feedback_error = preamp_output - feedback;
//         transfer_fcn_update = 1000000.0 * feedback_error;

//         plant_transfer_fcn = plant_transfer_fcn + plant_period * transfer_fcn_update;

//         printf("PLANT OUTPUT: %f ERROR : %f\n", *outputfeedback, feedback_error);
// }

// Controller Globals
static float c_input = 0.0;
static float c_output = 0.0;
static float controller_transfer = 0.0;
static float clock_time = 0.0;
const float period = 0.01;

// Plant Globals
static float p_input = 0.0;
static float p_output = 0.0;
static float plant_integrator = 0.0;
static float plant_transfer_fcn = 0.0;
static float plant_period = 0.01;

// --- Controller Functions ---

// Step 1: Receive Input
void controller_receive(int32_t val) {
    c_input = val;
    printf("CONTROLLER INPUT %f\n", c_input);
}

// Step 2: Compute Logic
void controller_compute(void) {
    float error;
    float gain_error_1;
    float gain_error;
    float transfer_fcn_update;
    float reference_input;

    if (clock_time < 1.0) {
        reference_input = 0.0;
    } else {
        reference_input = clock_time - 1.0;
    }

    error = reference_input - c_input;
    gain_error_1 = error * 0.1;
    gain_error = gain_error_1 * (-10000.0);
    transfer_fcn_update = gain_error - 170.0 * controller_transfer;

    c_output = 29.17 * controller_transfer + transfer_fcn_update;

    controller_transfer = controller_transfer + period * transfer_fcn_update;
    clock_time = clock_time + period;
}

// Step 3: Send Output
void controller_send(int32_t *val) {
    *val = c_output;
    printf("CONTROLLER OUTPUT %f\n", c_output);
    fflush(stdout);
}

// --- Plant Functions ---

// Step 1: Receive Input
void plant_receive(int32_t val) {
    p_input = val;
    printf("PLANT INPUT: %f\n", p_input);
    fflush(stdout);
}

// Step 2: Compute Logic
void plant_compute(void) {
    float feedback_error;
    float feedback;
    float integrator_output;
    float plant_output;
    float preamp_output;
    float transfer_fcn_update;

    preamp_output = p_input * (-2.0);
    integrator_output = plant_integrator;
    plant_output = 0.002 * plant_transfer_fcn;
    feedback = plant_output * 0.0125;

    p_output = integrator_output * 0.00125;
    
    plant_integrator = plant_integrator + 0.001 * plant_output;
    feedback_error = preamp_output - feedback;
    transfer_fcn_update = 1000000.0 * feedback_error;
    plant_transfer_fcn = plant_transfer_fcn + plant_period * transfer_fcn_update;
    printf("PLANT OUTPUT: %f ERROR : %f\n", p_output, feedback_error);
}

// Step 3: Send Output
void plant_send(int32_t *val) {
    *val = p_output;
    fflush(stdout);
}