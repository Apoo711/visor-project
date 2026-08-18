use std::{
    io::{Read, Write},
    time::Duration,
};

use log::{debug, info};
use serialport::SerialPort;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ArduinoResponse {
    Ready,
    AckDispense {
        bandage: bool,
        alcohol: bool,
        gauze: bool,
    },
    StatusDispensing(String),
    StatusHoldAll,
    StatusComplete,
    Pong,
    Error(String),
    Unknown(String),
    Empty,
}

pub fn parse_serial_response(raw: &str) -> ArduinoResponse {
    let clean = raw.trim();
    if clean.is_empty() {
        return ArduinoResponse::Empty;
    }
    if clean == "STATUS:READY" {
        return ArduinoResponse::Ready;
    }
    if clean == "STATUS:HOLD_ALL" {
        return ArduinoResponse::StatusHoldAll;
    }
    if clean == "STATUS:DISPENSE_COMPLETE" {
        return ArduinoResponse::StatusComplete;
    }
    if clean == "PONG" {
        return ArduinoResponse::Pong;
    }
    if let Some(item) = clean.strip_prefix("STATUS:DISPENSING_") {
        return ArduinoResponse::StatusDispensing(item.to_string());
    }
    if let Some(parts) = clean.strip_prefix("ACK:DISP:") {
        let tokens: Vec<&str> = parts.split(',').collect();
        if tokens.len() == 3 {
            return ArduinoResponse::AckDispense {
                bandage: tokens[0].trim() == "1",
                alcohol: tokens[1].trim() == "1",
                gauze: tokens[2].trim() == "1",
            };
        }
    }
    if let Some(err) = clean.strip_prefix("ERR:") {
        return ArduinoResponse::Error(err.to_string());
    }
    ArduinoResponse::Unknown(clean.to_string())
}

pub fn format_dispense_command(bandage: bool, alcohol_pad: bool, gauze_pad: bool) -> String {
    let b = if bandage { 1 } else { 0 };
    let a = if alcohol_pad { 1 } else { 0 };
    let g = if gauze_pad { 1 } else { 0 };
    format!("<DISP:{},{},{}>\n", b, a, g)
}

pub fn format_ping_command() -> String {
    "<PING>\n".to_string()
}

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
        let payload = format_dispense_command(bandage, alcohol_pad, gauze_pad);
        debug!("Sending framed dispense command: {}", payload.trim());
        self.send_bytes(payload.as_bytes())
    }

    pub fn send_hold(&mut self) -> Result<(), std::io::Error> {
        self.send_dispense(false, false, false)
    }

    pub fn send_ping(&mut self) -> Result<(), std::io::Error> {
        let payload = format_ping_command();
        self.send_bytes(payload.as_bytes())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_dispense_command_all_combinations() {
        assert_eq!(
            format_dispense_command(false, false, false),
            "<DISP:0,0,0>\n"
        );
        assert_eq!(
            format_dispense_command(true, false, false),
            "<DISP:1,0,0>\n"
        );
        assert_eq!(
            format_dispense_command(false, true, false),
            "<DISP:0,1,0>\n"
        );
        assert_eq!(
            format_dispense_command(false, false, true),
            "<DISP:0,0,1>\n"
        );
        assert_eq!(
            format_dispense_command(true, true, false),
            "<DISP:1,1,0>\n"
        );
        assert_eq!(
            format_dispense_command(true, false, true),
            "<DISP:1,0,1>\n"
        );
        assert_eq!(
            format_dispense_command(false, true, true),
            "<DISP:0,1,1>\n"
        );
        assert_eq!(
            format_dispense_command(true, true, true),
            "<DISP:1,1,1>\n"
        );
    }

    #[test]
    fn test_format_ping_command() {
        assert_eq!(format_ping_command(), "<PING>\n");
    }

    #[test]
    fn test_parse_serial_response_ready() {
        assert_eq!(parse_serial_response("STATUS:READY"), ArduinoResponse::Ready);
        assert_eq!(
            parse_serial_response("  STATUS:READY\r\n"),
            ArduinoResponse::Ready
        );
    }

    #[test]
    fn test_parse_serial_response_ack() {
        assert_eq!(
            parse_serial_response("ACK:DISP:1,0,1"),
            ArduinoResponse::AckDispense {
                bandage: true,
                alcohol: false,
                gauze: true,
            }
        );
        assert_eq!(
            parse_serial_response("ACK:DISP:0,1,0\n"),
            ArduinoResponse::AckDispense {
                bandage: false,
                alcohol: true,
                gauze: false,
            }
        );
    }

    #[test]
    fn test_parse_serial_response_status_messages() {
        assert_eq!(
            parse_serial_response("STATUS:HOLD_ALL"),
            ArduinoResponse::StatusHoldAll
        );
        assert_eq!(
            parse_serial_response("STATUS:DISPENSING_BANDAGE"),
            ArduinoResponse::StatusDispensing("BANDAGE".to_string())
        );
        assert_eq!(
            parse_serial_response("STATUS:DISPENSING_ALCOHOL"),
            ArduinoResponse::StatusDispensing("ALCOHOL".to_string())
        );
        assert_eq!(
            parse_serial_response("STATUS:DISPENSING_GAUZE"),
            ArduinoResponse::StatusDispensing("GAUZE".to_string())
        );
        assert_eq!(
            parse_serial_response("STATUS:DISPENSE_COMPLETE"),
            ArduinoResponse::StatusComplete
        );
        assert_eq!(parse_serial_response("PONG"), ArduinoResponse::Pong);
    }

    #[test]
    fn test_parse_serial_response_errors_and_edge_cases() {
        assert_eq!(parse_serial_response(""), ArduinoResponse::Empty);
        assert_eq!(parse_serial_response("   \r\n"), ArduinoResponse::Empty);
        assert_eq!(
            parse_serial_response("ERR:UNKNOWN_COMMAND:<FOO>"),
            ArduinoResponse::Error("UNKNOWN_COMMAND:<FOO>".to_string())
        );
        assert_eq!(
            parse_serial_response("SOMETHING_RANDOM_123"),
            ArduinoResponse::Unknown("SOMETHING_RANDOM_123".to_string())
        );
    }
}
