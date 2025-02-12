
#[cfg(any(feature="usb-ftdi", feature="raspberrypi"))]
use std::{thread, time};

#[cfg(any(feature="usb-ftdi", feature="raspberrypi"))]
use cands_presentation::cyphal::digitalservo::dictionary::DigitalServoPrimitiveData;

#[cfg(any(feature="usb-ftdi", feature="raspberrypi"))]
impl crate::CANInterface {
    pub fn drive_enable(&mut self, channel: u8) -> Result<(), Box<dyn std::error::Error>> {

        self.send_digitalservo_set_value(channel, "cmdval", &[0.0])?;
        thread::sleep(time::Duration::from_millis(50));

        self.send_digitalservo_set_value(channel, "drive", &[true])?;
        thread::sleep(time::Duration::from_millis(50));

        Ok(())
    }

    pub fn drive_disable(&mut self, channel: u8) -> Result<(), Box<dyn std::error::Error>> {

        self.send_digitalservo_set_value(channel, "drive", &[false])?;
        thread::sleep(time::Duration::from_millis(100));

        self.send_digitalservo_set_value(channel, "cmdval", &[0.0])?;
        thread::sleep(time::Duration::from_millis(50));

        Ok(())
    }


    pub fn drive_enable_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {

        self.send_digitalservo_message("cmdval", &[0.0])?;
        thread::sleep(time::Duration::from_millis(50));

        self.send_digitalservo_message("drive", &[true])?;
        thread::sleep(time::Duration::from_millis(50));

        Ok(())
    }
    
    pub fn drive_disable_all(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        
        self.send_digitalservo_message("drive", &[false])?;
        thread::sleep(time::Duration::from_millis(50));

        self.send_digitalservo_message("cmdval", &[0.0])?;
        thread::sleep(time::Duration::from_millis(50));

        Ok(())
    }

    pub fn send_cmdval(&mut self, channel: u8, value: f64) -> Result<(), Box<dyn std::error::Error>> {
        self.send_digitalservo_set_value(channel, "cmdval", &[value])
    }

    pub fn send_cmdarray(&mut self, channel: u8, value: &[f64]) -> Result<(), Box<dyn std::error::Error>> {
        self.send_digitalservo_set_value(channel, "cmdarray", value)
    }


    pub fn get_vector_response(&mut self, channel: u8, key: &str) -> Option<Vec<DigitalServoPrimitiveData>> {

        let ret = match self.send_digitalservo_get_value(channel, key) {
            Ok(ret) => ret,
            Err(_) => return None,
        };

        match ret.last() {
            Some(data) => Some(data.data.value.clone()),
            None => return None,
        }
    }

    pub fn get_scalar_response<T: TryFrom<DigitalServoPrimitiveData>>(&mut self, channel: u8, key: &str) -> Result<Option<T>, Box<dyn std::error::Error>> {

        let res = match self.get_vector_response(channel, key) {
            Some(data) => data,
            None => return Ok(None)
        };

        let data = match res.get(0) {
            Some(data) => data,
            None => return Ok(None)
        };

        match T::try_from(data.clone()) {
            Ok(ret) => Ok(Some(ret)),
            Err(_e) => Err("Type Not Mismatch".into()) 
        }
    }

    pub fn get_scalar_response_from_buffer<T: TryFrom<DigitalServoPrimitiveData>>(&mut self, channel: u8, key: &str) -> Result<Option<T>, Box<dyn std::error::Error>> {
        let data = match self.get_key_value(Some(key), Some(channel))? {
            Some(data) => data,
            None => return Ok(None)
        };

        let last_data = match data.last() {
            Some(data) => data.data.clone().value,
            None => return Ok(None)
        };

        let scalar = match last_data.get(0) {
            Some(data) => data.clone(),
            None => return Ok(None)
        };

        match T::try_from(scalar) {
            Ok(data) => Ok(Some(data)),
            Err(_) => Err("Type Not Mismatch".into()) 
        }

    }

}
