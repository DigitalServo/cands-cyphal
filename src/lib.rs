pub use cands_interface::{TCAN455xTranceiver, RxData, SIDConfig, XIDConfig};
pub use cands_transport::cyphal::{CyphalMiddleware, CyphalRxFrame, CyphalRxPacketType, CRC_SIZE_BYTES};
pub use cands_presentation::cyphal as serde;

mod special_instructions;
pub use special_instructions::digitalservo;

const MTU_CAN_FD: usize = 64;

#[cfg(any(feature="usb-ftdi", feature="raspberrypi", feature="raspberrypi_cm"))]
const NODE_ID: u8 = 127;

#[cfg(any(feature="usb-ftdi", feature="raspberrypi", feature="raspberrypi_cm"))]
const SIDF1: SIDConfig = SIDConfig { sft: 3, sfec: 0, sidf1: 0x123, sidf2: 0x456 };

#[cfg(any(feature="usb-ftdi", feature="raspberrypi", feature="raspberrypi_cm"))]
const SIDF2: SIDConfig = SIDConfig { sft: 3, sfec: 5, sidf1: 0x123, sidf2: 0x456 };

#[cfg(any(feature="usb-ftdi", feature="raspberrypi", feature="raspberrypi_cm"))]
const XIDF1: XIDConfig = XIDConfig { eft: 0, efec: 0, eidf1: 0x55555, eidf2: 0x77777 };

#[cfg(any(feature="usb-ftdi", feature="raspberrypi", feature="raspberrypi_cm"))]
const SIDF: [SIDConfig; 2] = [SIDF1, SIDF2];

#[cfg(any(feature="usb-ftdi", feature="raspberrypi", feature="raspberrypi_cm"))]
const XIDF: [XIDConfig; 1] = [XIDF1];

#[cfg(any(feature="raspberrypi", feature="raspberrypi_cm"))]
use cands_interface::GPIO_INPUT_PIN_NUM;

#[cfg(all(any(feature="usb-ftdi", feature="raspberrypi", feature="raspberrypi_cm"), feature="drvcan_v2"))]
const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(50);
#[cfg(all(any(feature="usb-ftdi", feature="raspberrypi", feature="raspberrypi_cm"), feature="drvcan_v2"))]
const DEFAULT_RETRY_COUNT: u32 = 20;


pub struct CANInterface {
    pub middleware: CyphalMiddleware<MTU_CAN_FD>,
    pub driver: TCAN455xTranceiver,
    pub rx_complete_fifo: Vec<CyphalRxFrame>,
    pub rx_incomplete_fifo: Vec<CyphalRxFrame>,
    #[cfg(feature="drvcan_v2")]
    pub timeout: std::time::Duration,
    #[cfg(feature="drvcan_v2")]
    pub retry_count: u32,
}


impl CANInterface {
    #[cfg(any(feature="usb-ftdi", feature="raspberrypi", feature="raspberrypi_cm"))]
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let middleware: CyphalMiddleware<MTU_CAN_FD> = CyphalMiddleware::<MTU_CAN_FD>::new(NODE_ID);
        let driver: TCAN455xTranceiver = TCAN455xTranceiver::new()?;
        
        let mut interface: Self = Self {
            middleware,
            driver,
            rx_complete_fifo: vec![],
            rx_incomplete_fifo: vec![],
            #[cfg(feature="drvcan_v2")]
            timeout: DEFAULT_TIMEOUT,
            #[cfg(feature="drvcan_v2")]
            retry_count: DEFAULT_RETRY_COUNT,
        };
        interface.init()?;
        
        Ok(interface)
    }

    #[cfg(any(feature="usb-ftdi", feature="raspberrypi", feature="raspberrypi_cm"))]
    pub fn init(&mut self) -> Result<(), Box<dyn std::error::Error>> {
 
        self.driver.setup(&SIDF, &XIDF)?;
        self.reset_rx_fifo();

        // Message: dummy transfer_id to make intentional missmatch of current_transfer_id in slaves and that in this system.
        let now: u128 = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH).unwrap()
            .as_millis();
        let id_init: u8 = (now % 32) as u8;
        self.middleware.transfer_id = id_init;
      
        Ok(())
    }

    #[cfg(all(any(feature="usb-ftdi", feature="raspberrypi", feature="raspberrypi_cm"), feature="drvcan_v2"))]
    pub fn set_timeout(&mut self, timeout: std::time::Duration) {
        self.timeout = timeout;
    }

    #[cfg(all(any(feature="usb-ftdi", feature="raspberrypi", feature="raspberrypi_cm"), feature="drvcan_v2"))]
    pub fn set_retry_count(&mut self, retry_count: u32) {
        self.retry_count = retry_count;
    }

    #[cfg(all(any(feature="usb-ftdi", feature="raspberrypi", feature="raspberrypi_cm"), feature="drvcan_v2"))]
    pub fn reset_settings(&mut self) {
        self.timeout = DEFAULT_TIMEOUT;
        self.retry_count = DEFAULT_RETRY_COUNT;
    }

    #[cfg(any(feature="raspberrypi", feature="raspberrypi_cm"))]
    pub fn gpi_read(&mut self, channel: usize) -> bool {
        self.driver.gpi_read(channel)
    }

    #[cfg(any(feature="raspberrypi", feature="raspberrypi_cm"))]
    pub fn gpi_read_all(&mut self) -> [bool; GPIO_INPUT_PIN_NUM] {
        self.driver.gpi_read_all()
    }


    #[cfg(any(feature="usb-ftdi", feature="raspberrypi", feature="raspberrypi_cm"))]
    pub fn reset_rx_fifo(&mut self) {
        self.rx_complete_fifo.clear();
        self.rx_incomplete_fifo.clear();
    }

    #[cfg(any(feature="usb-ftdi", feature="raspberrypi", feature="raspberrypi_cm"))]
    pub fn send_message(&mut self, subject_id: u16, payload: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        match self.middleware.create_message_data(subject_id, &payload, payload.len()) {
            Ok(packets) => {
                for packet in packets {
                    self.driver.transmit(packet.xid, &packet.payload, packet.payload_size)?
                }
            },
            Err(err) => return Err(err)
        }
        Ok(())
    }

    #[cfg(any(feature="usb-ftdi", feature="raspberrypi", feature="raspberrypi_cm"))]
    pub fn send_response(&mut self, service_id: u16, channel: u8, payload: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        match self.middleware.create_response_data(channel, service_id, &payload, payload.len()) {
            Ok(packets) => {
                for packet in packets {
                    self.driver.transmit(packet.xid, &packet.payload, packet.payload_size)?
                }
            },
            Err(err) => return Err(err)
        }
        Ok(())
    }

    #[cfg(any(feature="usb-ftdi", feature="raspberrypi", feature="raspberrypi_cm"))]
    pub fn send_request(&mut self, service_id: u16, channel: u8, payload: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        match self.middleware.create_request_data(channel, service_id, &payload, payload.len()) {
            Ok(packets) => {
                for packet in packets {
                    self.driver.transmit(packet.xid, &packet.payload, packet.payload_size)?
                }
            },
            Err(err) => return Err(err)
        }
        Ok(())
    }

    /// Read received data from a FIFO buffer on a device.
    #[cfg(any(feature="usb-ftdi", feature="raspberrypi", feature="raspberrypi_cm"))]
    pub fn read_device_fifo(&mut self) -> std::io::Result<Option<RxData>>{
        match self.driver.receive() {
            Ok(rx_data) => Ok(rx_data),
            Err(err) => Err(err)
        }
    }

    /// Load cyphal frames from a FIFO buffer on a user space.
    #[cfg(any(feature="usb-ftdi", feature="raspberrypi", feature="raspberrypi_cm"))]
    pub fn load_frames_from_buffer(&mut self, buffer: &[u8]) -> Result<(), Box<dyn std::error::Error>> {
        match self.middleware.try_read(buffer) {
            Ok(packets) => {
                for packet in packets {
                    match packet.status.frame_type {
                        CyphalRxPacketType::SignleFrame => {
                            self.rx_complete_fifo.push(CyphalRxFrame {
                                xid: packet.xid,
                                payload: packet.payload.to_vec(),
                                payload_size: packet.payload_size,
                                props: packet.props
                            });
                        },
                        CyphalRxPacketType::MultiFrameStart => {
                            self.rx_incomplete_fifo.push(CyphalRxFrame {
                                xid: packet.xid,
                                payload: Vec::from(&packet.payload[..packet.payload_size]),
                                payload_size: packet.payload_size,
                                props: packet.props
                            });
                        },
                        CyphalRxPacketType::MultiFrameInProcess => {
                            let target_frame_position: Option<usize> = self.rx_incomplete_fifo
                                .iter()
                                .position(|frame| (frame.xid == packet.xid) & (frame.props.port_id == packet.props.port_id));
                            if let Some(position) = target_frame_position {
                                self.rx_incomplete_fifo[position].payload.extend(&packet.payload[..packet.payload_size]);
                                self.rx_incomplete_fifo[position].payload_size += packet.payload_size;
                            }
                        },
                        CyphalRxPacketType::MultiFrameEnd => {
                            let target_frame_position: Option<usize> = self.rx_incomplete_fifo
                                .iter()
                                .position(|frame: &CyphalRxFrame| (frame.xid == packet.xid) & (frame.props.port_id == packet.props.port_id));

                            if let Some(position) = target_frame_position {
                                self.rx_incomplete_fifo[position].payload.extend(&packet.payload[..(packet.payload_size - CRC_SIZE_BYTES as usize)]);
                                self.rx_incomplete_fifo[position].payload_size += packet.payload_size - CRC_SIZE_BYTES as usize;

                                let crc_bytes: [u8; 2] = self.rx_incomplete_fifo[position].calculate_crc()?;
                                let crc_bytes_expected: [u8; 2] = [packet.payload[packet.payload_size - CRC_SIZE_BYTES as usize], packet.payload[packet.payload_size - CRC_SIZE_BYTES as usize + 1]];

                                if crc_bytes == crc_bytes_expected {
                                    self.rx_complete_fifo.push(self.rx_incomplete_fifo[position].clone());
                                    self.rx_incomplete_fifo.remove(position);
                                }
                                else {
                                    self.rx_incomplete_fifo.remove(position);
                                    return Err("INVALID DATA EXIST: CRC ERROR AT MULTIFRAME CONSTRUCTION".into());
                                }
                            }
                        }
                    }
                }
            },
            Err(err) => return Err(err)
        };

        Ok(())
    }

    /// Load cyphal frames from a FIFO buffer on a device.
    /// It wraps "read_device_fifo" and "load_frames_from_buffer"
    #[cfg(any(feature="usb-ftdi", feature="raspberrypi", feature="raspberrypi_cm"))]
    pub fn load_frames(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let rx_data: Option<RxData> = self.read_device_fifo()?;

        if let Some(rx_data) = rx_data {
            self.load_frames_from_buffer(&rx_data.fifo1)?
        }
        
        Ok(())
    }

}