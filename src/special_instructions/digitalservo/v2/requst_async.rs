#[cfg(any(feature="usb-ftdi", feature="raspberrypi"))]
use cands_presentation::cyphal::digitalservo::{
    dictionary::Dict,
    string::Str,
    traits::{DigitalServoPrimitiveData, IntoDigitalServoDataType}
};
use cands_transport::cyphal::CyphalRxData;
use tokio::time::error::Elapsed;

const CHECK_FIFO_POLLING_MS: u64 = 2;

#[cfg(any(feature="usb-ftdi", feature="raspberrypi"))]
impl crate::CANInterface {

    pub async fn async_send_digitalservo_set_value<T>(
        &mut self,
        channel: u8,
        key: &str,
        value: &[T],
    ) -> Result<(), Box<dyn std::error::Error>>
        where T: Clone + IntoDigitalServoDataType + Into<DigitalServoPrimitiveData>
    {
        const SERVICE_ID: u16 = 0x81;

        let timeout = self.timeout;

        for _ in 0..self.retry_count {
            let ret: Result<Result<(), Box<dyn std::error::Error>>, Elapsed> = {
                
                let task = async {
                    let payload:Vec<u8> = Dict::serialize(key, &value);
                    self.send_request(SERVICE_ID, channel, &payload)?;

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

                        tokio::time::sleep(std::time::Duration::from_millis(CHECK_FIFO_POLLING_MS)).await;
                    }
                };

                tokio::time::timeout(timeout, task).await
            };

            if let Ok(ret) = ret {
                match ret {
                    Ok(success) => return Ok(success),
                    Err(e) => return Err(e)
                }
            }
        }

        let err: std::io::Error = std::io::ErrorKind::TimedOut.into();
        Err(err.into())
    }

    pub async fn async_send_digitalservo_get_value(
        &mut self,
        channel: u8,
        key: &str,
    ) -> Result<Vec<CyphalRxData<Dict>>, Box<dyn std::error::Error>> {
        
        const SERVICE_ID: u16 = 0x82;
        let payload:Vec<u8> = Str::serialize(key);

        let timeout = self.timeout;

        for _ in 0..self.retry_count {
            let ret: Result<Result<Vec<CyphalRxData<Dict>>, Box<dyn std::error::Error>>, Elapsed> = {

                let task = async {
                    
                    self.send_request(SERVICE_ID, channel, &payload)?;

                    loop {
                        let results = match self.get_key_value(Some(key), Some(channel)) {
                            Ok(ret) => ret,
                            Err(_) => continue,
                        };

                        if let Some(results) = results {
                            return Ok(results)
                        }

                        tokio::time::sleep(std::time::Duration::from_millis(CHECK_FIFO_POLLING_MS)).await;
                    }
                };
        
                tokio::time::timeout(timeout, task).await
            };

            if let Ok(ret) = ret {
                match ret {
                    Ok(success) => return Ok(success),
                    Err(e) => return Err(e)
                }
            }
        }

        let err: std::io::Error = std::io::ErrorKind::TimedOut.into();
        Err(err.into())
    }

}