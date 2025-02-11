#[cfg(any(feature="usb-ftdi", feature="raspberrypi"))]
use cands_presentation::cyphal::digitalservo::{
    dictionary::Dict,
    string::Str,
    traits::{DigitalServoPrimitiveData, IntoDigitalServoDataType}
};

use futures_lite::FutureExt;
use async_io::{block_on, Timer};


#[cfg(any(feature="usb-ftdi", feature="raspberrypi"))]
impl crate::CANInterface {

    pub fn send_digitalservo_message<T: Clone + IntoDigitalServoDataType + Into<DigitalServoPrimitiveData>>(&mut self, key: &str, value: &[T]) -> Result<(), Box<dyn std::error::Error>> {
        const SUBJECT_ID: u16 = 0x488;
        let payload:Vec<u8> = Dict::serialize(key, value);
        self.send_message(SUBJECT_ID, &payload)
    }

    pub fn send_digitalservo_response<T: Clone + IntoDigitalServoDataType + Into<DigitalServoPrimitiveData>>(&mut self, channel: u8, key: &str, value: &[T]) -> Result<(), Box<dyn std::error::Error>> {
        const SERVICE_ID: u16 = 0x80;
        let payload:Vec<u8> = Dict::serialize(key, &value);
        self.send_response(SERVICE_ID, channel, &payload)
    }

    pub fn send_digitalservo_request(&mut self, channel: u8, key: &str) -> Result<(), Box<dyn std::error::Error>> {
        const SERVICE_ID: u16 = 0x80;
        let payload:Vec<u8> = Dict::serialize(key, &[0.0]);
        self.send_request(SERVICE_ID, channel, &payload)
    }

    pub fn send_digitalservo_set_value<T>(
        &mut self,
        channel: u8,
        key: &str,
        value: &[T],
        timeout: std::time::Duration
    ) -> Result<(), Box<dyn std::error::Error>>
        where T: Clone + IntoDigitalServoDataType + Into<DigitalServoPrimitiveData>
    {
        const SERVICE_ID: u16 = 0x81;
        let payload:Vec<u8> = Dict::serialize(key, &value);
        self.send_request(SERVICE_ID, channel, &payload)?;

        let task = async {
            loop {
                match self.get_result(Some(channel)) {
                    Ok(v) => {
                        match v {
                            Some(x) => {
                                if x.iter().all(|y| y.data == 0) {
                                    println!("ok");
                                    return Ok(())
                                }
                            },
                            None => {}
                        }
                    },
                    Err(_) => {}
                };
                // Timer::after(std::time::Duration::from_millis(2)).await;
            }
        };
    
        let timeout_handler = async {
            Timer::after(timeout).await;
            // Err(std::io::ErrorKind::TimedOut)
            Err("Timeout".into())
        };
    
        block_on(task.or(timeout_handler))
    }

    pub fn send_digitalservo_get_value(&mut self, channel: u8, key: &str) -> Result<(), Box<dyn std::error::Error>> {
        const SERVICE_ID: u16 = 0x82;
        let payload:Vec<u8> = Str::serialize(key);
        self.send_request(SERVICE_ID, channel, &payload)
    }

}