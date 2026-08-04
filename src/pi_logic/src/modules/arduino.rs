use serialport::SerialPort;
use std::time::Duration;

pub struct ArduinoBridge {
    port: Box<dyn SerialPort>,
}

impl ArduinoBridge {
    pub fn new(port_name: &str, baud_rate: u32) -> Result<Self, serialport::Error> {
        let port = serialport::new(port_name, baud_rate)
            .timeout(Duration::from_millis(1000))
            .open()?;
        Ok(Self { port })
    }

    // Send a single command byte to the Arduino
    pub fn send_command(&mut self, cmd: u8) -> Result<(), std::io::Error> {
        self.port.write_all(&[cmd])?;
        self.port.flush()?;
        Ok(())
    }
}