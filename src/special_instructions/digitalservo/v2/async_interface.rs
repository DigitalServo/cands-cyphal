use std::{sync::Arc, time::Duration};
use cands_transport::cyphal::CyphalRxData;
use tokio::sync::Mutex;

use cands_presentation::cyphal::digitalservo::{dictionary::Dict, traits::{DigitalServoPrimitiveData, IntoDigitalServoDataType}};
use crate::CANInterface;

type IoResult<T> = Result<T, std::io::Error>;

pub struct AsyncCANInterface {
    interface: Arc<Mutex<CANInterface>>,
    timeout: std::time::Duration,
    retry_count: usize, 
}

impl AsyncCANInterface {
    pub fn new(
        interface: Arc<Mutex<CANInterface>>,
        timeout: std::time::Duration,
        retry_count: usize, 
    ) -> Self {
        Self {
            interface,
            timeout,
            retry_count,
        }
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = timeout;
    }

    pub fn set_retry_count(&mut self, retry_count: usize) {
        self.retry_count = retry_count;
    }

    pub async fn operate_interface<T, F>(&self, f: F) -> Result<T, Box<dyn std::error::Error>>
    where F: FnOnce(&mut CANInterface) -> Result<T, Box<dyn std::error::Error>>
    {
        let mut interface = self.interface.lock().await;
        f(&mut interface)
    }

    pub async fn send_digitalservo_set_value<T> (
        &self,
        channel: u8,
        key: &str,
        value: &[T],
    ) -> Result<(), std::io::Error>
        where T: Clone + IntoDigitalServoDataType + Into<DigitalServoPrimitiveData> + Send + Sync + 'static
    {    
        let resource = self.interface.clone();

        let key = key.to_string();
        let value = value.to_vec();
        let timeout = self.timeout.clone();
        let retry_count = self.retry_count;

        let task: tokio::task::JoinHandle<IoResult<()>> = tokio::spawn(async move {
            let mut canif = resource.lock().await;

            for _ in 0..retry_count {
                if let Ok(_) = canif.async_send_digitalservo_set_value(channel, &key, &value, timeout).await {
                    return Ok(());
                };
            }

            return Err(std::io::ErrorKind::TimedOut.into());
        });

        task.await?
    }

    pub async fn send_digitalservo_get_value (
        &self,
        channel: u8,
        key: &str,
    ) -> Result<Vec<CyphalRxData<Dict>>, std::io::Error>
    {    
        let resource = self.interface.clone();

        let key = key.to_string();
        let timeout = self.timeout.clone();
        let retry_count = self.retry_count;

        let task: tokio::task::JoinHandle<IoResult<Vec<CyphalRxData<Dict>>>> = tokio::spawn(async move {
            let mut canif = resource.lock().await;

            for _ in 0..retry_count {
                if let Ok(data) = canif.async_send_digitalservo_get_value(channel, &key, Some(channel), timeout).await {
                    return Ok(data)
                };
            }

            return Err(std::io::ErrorKind::TimedOut.into());
        });

        task.await?
    }

}