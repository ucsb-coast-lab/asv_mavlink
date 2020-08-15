#![allow(non_snake_case)]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

mod mavlink_setup;
use crate::mavlink_setup::*;

mod data_types;
use crate::data_types::*;

use mavlink::common::*;
use mavlink::Message;
use simple_signal::{self, Signal};

fn main() {
    // We'll be checking to see if Ctrl^C is called, and if so, will kill the program signal
    let running = Arc::new(AtomicBool::new(true));
    // When a SIGINT (Ctrl-C) or SIGTERM signal is caught, atomically set running to false.
    simple_signal::set_handler(&[Signal::Int, Signal::Term], {
        let running = running.clone();
        move |_| {
            running.store(false, Ordering::SeqCst);
        }
    });

    // Compiling with this option will read published values from the ASV on the HOST COMPUTER
    // let mavconn = mavlink::connect("udpin:0.0.0.0:14550").unwrap();

    // Compile with this option to run directly on the Jetson Nano
    let mavconn = mavlink::connect("udpout:0.0.0.0:9000").unwrap();

    let vehicle = Arc::new(mavconn);
    vehicle
        .send(&mavlink::MavHeader::default(), &request_parameters())
        .unwrap();
    vehicle
        .send(&mavlink::MavHeader::default(), &request_stream())
        .unwrap();

    let mavlink_version = vehicle.get_protocol_version();
    println!("MAVLINK Protocol version: {:?}", mavlink_version);

    // Creates a separate thread that keeps a heartbeat between the topside and vehicle computer
    thread::spawn({
        let vehicle = vehicle.clone();
        move || loop {
            let res = vehicle.send_default(&heartbeat_message());
            if res.is_ok() {
                thread::sleep(Duration::from_secs(1));
            } else {
                println!("send failed: {:?}", res);
            }
        }
    });

    // Arm the system
    println!("- About to arm the system:");
    // vehicle.send_default(&system_arm()).unwrap(); // Arming the system currently spins the motors, both using Mavlink API and QGC

    // This DO_SET_MODE doesn't seem to be doing much...
    /*
    vehicle.send_default(
        &mavlink::common::MavMessage::COMMAND_LONG(mavlink::common::COMMAND_LONG_DATA {
            param1: 16.0,
            param2: 0.0,
            param3: 0.0,
            param4: 0.0,
            param5: 0.0,
            param6: 0.0,
            param7: 0.0,
            command: mavlink::common::MavCmd::MAV_CMD_DO_SET_MODE,
            target_system: 0,
            target_component: 0,
            confirmation: 0,
        })
    ).unwrap();
    */

    println!("- About to enter loop:");
    // While the program is active:
    // We're going to take all the messages and add them to a queue
    let mut msgs: Vec<mavlink::common::MavMessage> = Vec::new();
    // Starting the mission time now
    let mission_time = Instant::now();

    // After deciding which messages we actually want to process, we create empty placeholders here
    let mut gps: mavlink::common::MavMessage =
        mavlink::common::MavMessage::GLOBAL_POSITION_INT(GLOBAL_POSITION_INT_DATA {
            time_boot_ms: 0,
            lat: 0,
            lon: 0,
            alt: 0,
            relative_alt: 0,
            vx: 0,
            vy: 0,
            vz: 0,
            hdg: 0,
        });

    let mut attitude: mavlink::common::MavMessage =
        mavlink::common::MavMessage::ATTITUDE(mavlink::common::ATTITUDE_DATA {
            time_boot_ms: 0,
            roll: 0.0,
            pitch: 0.0,
            yaw: 0.0,
            rollspeed: 0.0,
            pitchspeed: 0.0,
            yawspeed: 0.0,
        });

    // While we're under the mission time limit and haven't received a Ctrl^C termination signal
    while mission_time.elapsed().as_millis() < 2500 && running.load(Ordering::SeqCst) {
        // If the message is received correctly, push that into a queue; else, return the error
        match vehicle.recv() {
            Ok((_header, msg)) => {
                // println!("msg: {:?}\n",msg);
                msgs.push(msg);
            }
            Err(e) => {
                println!("e: {:?}", e);
                match e {
                    _MessageReadError => {
                        //no messages currently available to receive -- wait a little while
                        thread::sleep(Duration::from_millis(10));
                        continue;
                    }
                    _ => {
                        break;
                    }
                }
            }
        }

        // Parse the last message in the queue
        let last_msg = &msgs.last().cloned().unwrap();

        // At the moment, we only want to look at Attitude and GPS data
        match last_msg {
            mavlink::common::MavMessage::ATTITUDE(attitude_data) => {
                attitude = mavlink::common::MavMessage::ATTITUDE(attitude_data.clone());
            }
            mavlink::common::MavMessage::GLOBAL_POSITION_INT(gps_data) => {
                gps = mavlink::common::MavMessage::GLOBAL_POSITION_INT(gps_data.clone());
            }
            _ => {
                continue;
            }
        }
        println!("ATTITUDE_DATA: {:?}",parse_attitude_data_30(&attitude).unwrap());
        println!("GPS_DATA: {:?}\n",parse_gps_data_33(&gps).unwrap());

        // Do some processing with the data
        // Process the data {/* ... */}
        // Now send the vehicle a manual command based on the positon data
        // vehicle.send_default(&manual_control(1.,1.,1.,1.,0,0)).expect("Couldn't send manual control");
    }
    // Disarm the system as soon as the run loop is disrupted
    vehicle.send_default(&system_disarm()).unwrap();
} // End of program
