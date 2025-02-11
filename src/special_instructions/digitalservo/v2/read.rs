#[cfg(any(feature="usb-ftdi", feature="raspberrypi"))]
use cands_transport::cyphal::CyphalRxData;

#[cfg(any(feature="usb-ftdi", feature="raspberrypi"))]
use cands_presentation::cyphal::digitalservo::dictionary::Dict;


#[cfg(any(feature="usb-ftdi", feature="raspberrypi"))]
impl crate::CANInterface {

    pub fn clear_rx_complete_fifo(&mut self) {
        self.rx_complete_fifo.clear();
    }

    pub fn clear_rx_incomplete_fifo(&mut self) {
        self.rx_incomplete_fifo.clear();
    }

    pub fn get_digitalservo_general_status(&mut self) -> u8 {
        const TARGET_PORT_ID: u16 = 0x87;
        // Filter data which are to be processed
        let mut target_ids: Vec<usize> = vec![];
        for i in 0..self.rx_complete_fifo.len() {
            let port_id: u16 = self.rx_complete_fifo[i].props.port_id;
            if port_id == TARGET_PORT_ID {
                target_ids.push(i);
            }
        }

        let mut result :u8 = 0xFF;

        // Process target data
        for process_target_id in &target_ids {
            let packet = &self.rx_complete_fifo[*process_target_id];
            // match Dict::deserialize(&packet.payload) {
            //     Ok(data) => v.push(CyphalRxData{data, props: packet.props}),
            //     Err(err) => return Err(err)
            // }
            // v.push(&packet.payload);
            match packet.payload.first() {
                Some(val) => result = *val,
                None => ()
            }

        }

        // Delete processed data from rx_fifo
        for remove_target_id in target_ids.iter().rev() {
            self.rx_complete_fifo.remove(*remove_target_id);
        }

        result
    }


    pub fn get_key_value(&mut self, key: Option<&str>, source_node_id: Option<u8>) -> Result<Option<Vec<CyphalRxData<Dict>>>, Box<dyn std::error::Error>> {
        const TARGET_PORT_ID: [u16; 3] = [128, 129, 1160];

        let mut buffer: Vec<CyphalRxData<Dict>> = Vec::new();

        // Load data from a device FIFO and put RxFrames on a user-space FIFO
        self.load_frames()?;
        
        // Filter data which are to be processed
        let mut target_ids: Vec<usize> = vec![];
        let mut remove_ids: Vec<usize> = vec![];
        for i in 0..self.rx_complete_fifo.len() {
            let port_id: u16 = self.rx_complete_fifo[i].props.port_id;
            if (port_id == TARGET_PORT_ID[0]) | (port_id == TARGET_PORT_ID[1]) | (port_id == TARGET_PORT_ID[2]) {
                target_ids.push(i);
            }
        }

        // Process target data
        for process_target_id in &target_ids {
            let packet = &self.rx_complete_fifo[*process_target_id];

            if let Ok(data) = Dict::deserialize(&packet.payload) {
                let get_flag = if let Some(key) = key { data.key == key } else { true };
                let get_flag = get_flag && if let Some(source_node_id) = source_node_id { packet.props.source_node_id == source_node_id } else { true };

                if get_flag {
                    buffer.push(CyphalRxData{data, props: packet.props});
                    remove_ids.push(*process_target_id);
                }
            }

        }

        // Delete processed data from rx_fifo
        for id in remove_ids.iter().rev() {
            self.rx_complete_fifo.remove(*id);
        }

        match buffer.len() {
            0 => Ok(None),
            _ => Ok(Some(buffer)) 
        }

    }

    pub fn get_result(&mut self, source_node_id: Option<u8>) -> Result<Option<Vec<CyphalRxData<u8>>>, Box<dyn std::error::Error>> {
        const TARGET_PORT_ID: u16 = 0x87;

        let mut buffer: Vec<CyphalRxData<u8>> = Vec::new();

        // Load data from a device FIFO and put RxFrames on a user-space FIFO
        self.load_frames()?;
        
        // Filter data which are to be processed
        let mut target_ids: Vec<usize> = vec![];
        let mut remove_ids: Vec<usize> = vec![];
        for i in 0..self.rx_complete_fifo.len() {
            let port_id: u16 = self.rx_complete_fifo[i].props.port_id;
            if port_id == TARGET_PORT_ID {
                target_ids.push(i);
            }
        }

        // Process target data
        for process_target_id in &target_ids {
            let packet = &self.rx_complete_fifo[*process_target_id];

            let get_flag = if let Some(source_node_id) = source_node_id { packet.props.source_node_id == source_node_id } else { true };

            if get_flag {
                buffer.push(CyphalRxData{data: packet.payload[0], props: packet.props});
                remove_ids.push(*process_target_id);
            }
        }

        // Delete processed data from rx_fifo
        for id in remove_ids.iter().rev() {
            self.rx_complete_fifo.remove(*id);
        }

        match buffer.len() {
            0 => Ok(None),
            _ => Ok(Some(buffer)) 
        }
    }


    pub fn get_error(&mut self, source_node_id: Option<u8>) -> Result<Option<Vec<CyphalRxData<u8>>>, Box<dyn std::error::Error>> {
        const TARGET_PORT_ID: u16 = 0x17C0;

        let mut buffer: Vec<CyphalRxData<u8>> = Vec::new();

        // Load data from a device FIFO and put RxFrames on a user-space FIFO
        self.load_frames()?;
        
        // Filter data which are to be processed
        let mut target_ids: Vec<usize> = vec![];
        let mut remove_ids: Vec<usize> = vec![];
        for i in 0..self.rx_complete_fifo.len() {
            let port_id: u16 = self.rx_complete_fifo[i].props.port_id;
            if port_id == TARGET_PORT_ID {
                target_ids.push(i);
            }
        }

        // Process target data
        for process_target_id in &target_ids {
            let packet = &self.rx_complete_fifo[*process_target_id];

            let get_flag = if let Some(source_node_id) = source_node_id { packet.props.source_node_id == source_node_id } else { true };

            if get_flag {
                buffer.push(CyphalRxData{data: packet.payload[0], props: packet.props});
                remove_ids.push(*process_target_id);
            }
        }

        // Delete processed data from rx_fifo
        for id in remove_ids.iter().rev() {
            self.rx_complete_fifo.remove(*id);
        }

        match buffer.len() {
            0 => Ok(None),
            _ => Ok(Some(buffer)) 
        }
    }
}