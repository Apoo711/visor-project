use std::{
    io::{Read, Write},
    time::Duration,
};

use log::{debug, info};
use serialport::SerialPort;

pub struct ArduinoBridge {
    port: Box<dyn SerialPort>,
}

impl ArduinoBridge {
    pub fn new(port_name: &str, baud_rate: u32) -> Result<Self, serialport::Error> {
        info!(
            "Connecting to Arduino on {} at {} baud...",
            port_name, baud_rate
        );
        let port = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(1000))
            .open()?;
        Ok(Self { port })
    }

    pub fn send_dispense(
        &mut self,
        bandage: bool,
        alcohol_pad: bool,
        gauze_pad: bool,
    ) -> Result<(), std::io::Error> {
        let b = if bandage { 1 } else { 0 };
        let a = if alcohol_pad { 1 } else { 0 };
        let g = if gauze_pad { 1 } else { 0 };

        let payload = format!("<DISP:{},{},{}>\n", b, a, g);
        debug!("Sending framed dispense command: {}", payload.trim());
        self.send_bytes(payload.as_bytes())
    }

    pub fn send_hold(&mut self) -> Result<(), std::io::Error> {
        self.send_dispense(false, false, false)
    }

    pub fn send_bytes(&mut self, bytes: &[u8]) -> Result<(), std::io::Error> {
        self.port.write_all(bytes)?;
        self.port.flush()?;
        Ok(())
    }

    pub fn read_response(&mut self) -> Result<String, std::io::Error> {
        let mut buffer = [0u8; 128];
        match self.port.read(&mut buffer) {
            Ok(bytes_read) if bytes_read > 0 => {
                let response = String::from_utf8_lossy(&buffer[..bytes_read])
                    .trim()
                    .to_string();
                debug!("Received response from Arduino: {}", response);
                Ok(response)
            }
            Ok(_) => Ok(String::new()),
            Err(e) => Err(e),
        }
    }
}
