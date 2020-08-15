#![allow(non_snake_case)]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

mod mavlink_setup;
use crate::mavlink_setup::*;

mod data_types;
use crate::data_types::*;

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

    // Connect from host computer
    let mavconn = mavlink::connect("udpin:0.0.0.0:14550").unwrap();

    // For cross-compiling and putting onto the Jetson Nano
    // let mut mavconn = mavlink::connect("udpout:0.0.0.0:9000").unwrap();

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
    vehicle.send_default(&system_arm()).unwrap();

    println!("- About to enter loop:");
    // While the program is active:
    // We're going to take all the messages and add them to a queue
    let mut msgs: Vec<mavlink::common::MavMessage> = Vec::new();
    // Starting the mission time now
    let mission_time = Instant::now();

    // While we're under the mission time limit and haven't received a Ctrl^C termination signal
    while mission_time.elapsed().as_secs() < 5 && running.load(Ordering::SeqCst) {

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
                        //no messages currently available to receive -- wait a while
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
        
        // For debugging
        let id = mavlink::common::MavMessage::message_id(last_msg);
        // println!("{:?}", id);
        // let payload = mavlink::common::MavMessage::ser(last_msg);
        // println!("{:?}", payload);

        // Parse the data payload based on the message ID
        /*
        let data = mavlink::common::MavMessage::parse(
            mavlink::MavlinkVersion::V2,
            last_msg.message_id(),
            &last_msg.ser(),
        )
        .expect("Coudln't parse data");
        */

        // At the moment, we only want to look at Attitude and GPS data
        match id {
            30 => {
                let attitude =
                    parse_attitude_message_30(last_msg).expect("Couldn't parse attitude message");
                println!("Attitude: {:?}", attitude);
            }
            33 => {
                let gps = parse_gps_message_33(last_msg).expect("Couldn't parse GPS message");
                println!("GPS: {:?}\n", gps);
            }
            _ => {
                continue;
            }
        }
        
        // Do some processing with the data
        // Process the data {/* ... */}
        // Now send the vehicle a manual command based on the positon data
        // vehicle.send_default(&manual_control(0.0,0.0,0.0,0,0,0)).expect("Couldn't send manual control");

    }
    // Disarm the system as soon as the run loop is disrupted
    vehicle.send_default(&system_disarm()).unwrap();
    
} // End of program
