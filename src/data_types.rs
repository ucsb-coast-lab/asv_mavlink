use mavlink::common::*;
use mavlink::Message;
use std::error::Error;
use std::f32::consts::PI;

#[derive(Debug, Clone, Copy)]
pub struct Attitude {
    pub roll: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub rollspeed: f32,
    pub pitchspeed: f32,
    pub yawspeed: f32,
}

impl Attitude {
    pub fn new(
        roll: f32,
        pitch: f32,
        yaw: f32,
        rollspeed: f32,
        pitchspeed: f32,
        yawspeed: f32,
    ) -> Self {
        Attitude {
            roll: roll,
            pitch: pitch,
            yaw: yaw,
            rollspeed: rollspeed,
            pitchspeed: pitchspeed,
            yawspeed: yawspeed,
        }
    }
}

pub fn parse_attitude_message_30(
    msg: &mavlink::common::MavMessage,
) -> Result<Attitude, Box<dyn Error>> {
    let data = mavlink::common::MavMessage::parse(
        mavlink::MavlinkVersion::V2,
        msg.message_id(),
        &msg.ser(),
    )?;

    let attitude = match data {
        mavlink::common::MavMessage::ATTITUDE(mavlink::common::ATTITUDE_DATA {
            time_boot_ms: _,
            roll,
            pitch,
            yaw,
            rollspeed,
            pitchspeed,
            yawspeed,
        }) => Attitude::new(
            roll,
            pitch,
            yaw * 180.0 / PI, // yaw is now going to be in degrees instead of radians
            rollspeed,        // However, the "speed" variables should stil be in rad/s
            pitchspeed,
            yawspeed * 180.0 / PI, //
        ),
        _ => panic!("Error while parsing message"), // If it's not an Attitude message, then we need to break off
    };
    Ok(attitude)
}

#[derive(Debug, Clone, Copy)]
pub struct GPS {
    pub lat: f32,
    pub lon: f32,
    pub vx: f32,
    pub vy: f32,
    pub hdg: f32,
}

impl GPS {
    pub fn new(lat: i32, lon: i32, vx: i16, vy: i16, hdg: u16) -> Self {
        GPS {
            lat: lat as f32 / 1e7f32,
            lon: lon as f32 / 1e7f32,
            vx: vx as f32,
            vy: vy as f32,
            hdg: hdg as f32 / 100.,
        }
    }
}

pub fn parse_gps_message_33(msg: &mavlink::common::MavMessage) -> Result<GPS, Box<dyn Error>> {
    let data = mavlink::common::MavMessage::parse(
        mavlink::MavlinkVersion::V2,
        msg.message_id(),
        &msg.ser(),
    )?;

    let gps: GPS = match data {
        mavlink::common::MavMessage::GLOBAL_POSITION_INT(GLOBAL_POSITION_INT_DATA {
            time_boot_ms: _,
            lat,
            lon,
            alt: _,
            relative_alt: _,
            vx,
            vy,
            vz: _,
            hdg,
        }) => GPS::new(lat, lon, vx, vy, hdg),
        _ => panic!("Failed to parse"),
    };
    Ok(gps)
}
