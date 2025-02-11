#[cfg(any(feature="usb-ftdi", feature="raspberrypi"))]
use cands_presentation::cyphal::digitalservo::{
    dictionary::Dict,
    string::Str,
    traits::{DigitalServoPrimitiveData, IntoDigitalServoDataType}
};

use cands_transport::cyphal::CyphalRxData;
use futures_lite::FutureExt;
use async_io::{block_on, Timer};

const CHECK_FIFO_POLLING_MS: u64 = 2;

#[cfg(any(feature="usb-ftdi", feature="raspberrypi"))]
impl crate::CANInterface {

    pub fn send_digitalservo_message<T: Clone + IntoDigitalServoDataType + Into<DigitalServoPrimitiveData>>(&mut self, key: &str, value: &[T]) -> Result<(), Box<dyn std::error::Error>> {
        const SUBJECT_ID: u16 = 0x488;
        let payload:Vec<u8> = Dict::serialize(key, value);
        self.send_message(SUBJECT_ID, &payload)
    }

    #[deprecated]
    pub fn send_digitalservo_response<T: Clone + IntoDigitalServoDataType + Into<DigitalServoPrimitiveData>>(&mut self, channel: u8, key: &str, value: &[T]) -> Result<(), Box<dyn std::error::Error>> {
        const SERVICE_ID: u16 = 0x80;
        let payload:Vec<u8> = Dict::serialize(key, &value);
        self.send_response(SERVICE_ID, channel, &payload)
    }

    #[deprecated]
    pub fn send_digitalservo_request(&mut self, channel: u8, key: &str) -> Result<(), Box<dyn std::error::Error>> {
        const SERVICE_ID: u16 = 0x80;
        let payload:Vec<u8> = Dict::serialize(key, &[0.0]);
        self.send_request(SERVICE_ID, channel, &payload)
    }

    pub fn send_digitalservo_set_value <T>(
        &mut self,
        channel: u8,
        key: &str,
        value: &[T],
    ) -> Result<(), Box<dyn std::error::Error>>
        where T: Clone + IntoDigitalServoDataType + Into<DigitalServoPrimitiveData>
    {
        const SERVICE_ID: u16 = 0x81;
        let payload:Vec<u8> = Dict::serialize(key, &value);

        let timeout = self.timeout;

        for _ in 0..self.retry_count {

            self.send_request(SERVICE_ID, channel, &payload)?;

            let ret: Result<(), ()> = {
                let task = async {
                    loop {
                        let _ = self.load_frames();
                        let results = match self.get_result(Some(channel)) {
                            Ok(ret) => ret,
                            Err(_) => continue,
                        };

                        if let Some(results) = results {
                            if results.iter().all(|y| y.data == 0) {
                                return Ok(())
                            }
                        }
                        Timer::after(std::time::Duration::from_millis(CHECK_FIFO_POLLING_MS)).await;
                    }

                };
                
                let timeout_handler = async {
                    Timer::after(timeout).await;
                    Err(())
                };
            
                block_on(task.or(timeout_handler))
            };

            if let Ok(success) = ret {
                return Ok(success)
            }
        }

        let err: std::io::Error = std::io::ErrorKind::TimedOut.into();
        Err(err.into())

    }

    pub fn send_digitalservo_get_value(
        &mut self,
        channel: u8,
        key: &str,
    ) -> Result<Vec<CyphalRxData<Dict>>, Box<dyn std::error::Error>> {

        const SERVICE_ID: u16 = 0x82;
        let payload:Vec<u8> = Str::serialize(key);

        let timeout = self.timeout;

        for _ in 0..self.retry_count {

            self.send_request(SERVICE_ID, channel, &payload)?;

            let ret: Result<Vec<CyphalRxData<Dict>>, ()> = {
                let task = async {
                    loop {
                        let results = match self.get_key_value(Some(key), Some(channel)) {
                            Ok(ret) => ret,
                            Err(_) => continue,
                        };

                        if let Some(results) = results {
                            return Ok(results)
                        }
                        Timer::after(std::time::Duration::from_millis(CHECK_FIFO_POLLING_MS)).await;
                    }
                };

                let timeout_handler = async {
                    Timer::after(timeout).await;
                    Err(())
                };
            
                block_on(task.or(timeout_handler))
            };
            
            if let Ok(success) = ret {
                return Ok(success)
            }
        }

        let err: std::io::Error = std::io::ErrorKind::TimedOut.into();
        Err(err.into())

    }

}