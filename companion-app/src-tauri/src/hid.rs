use hidapi::{HidApi, HidDevice};
use std::sync::Mutex;

// KQB Raw HID parameters
// Usage page 0xFF60, usage 0x61 is the standard QMK Raw HID interface
const RAW_HID_USAGE_PAGE: u16 = 0xFF60;
const RAW_HID_USAGE: u16 = 0x61;
const RAW_HID_REPORT_SIZE: usize = 32;

// Companion protocol command IDs (must match firmware companion.h)
const CMD_GET_STATUS: u8 = 0x20;
const CMD_SET_LAYER: u8 = 0x21;
const CMD_SET_OS_OVERRIDE: u8 = 0x22;
const CMD_PING: u8 = 0x23;
const CMD_GET_VERSION: u8 = 0x24;

#[derive(Debug, serde::Serialize, Clone)]
pub struct DeviceStatus {
    pub default_layer: u8,
    pub layer_state: u32,
    pub os_override: u8,
    pub layer_count: u8,
}

#[derive(Debug, serde::Serialize, Clone)]
pub struct DeviceVersion {
    pub protocol_version: u16,
    pub uid: [u8; 8],
}

pub struct HidConnection {
    device: Option<HidDevice>,
}

impl HidConnection {
    pub fn new() -> Self {
        Self { device: None }
    }

    pub fn connect(&mut self) -> Result<(), String> {
        let api = HidApi::new().map_err(|e| format!("Failed to init HID API: {}", e))?;

        for dev_info in api.device_list() {
            if dev_info.usage_page() == RAW_HID_USAGE_PAGE && dev_info.usage() == RAW_HID_USAGE {
                match dev_info.open_device(&api) {
                    Ok(device) => {
                        // Try a ping to verify it's a KQB with companion protocol
                        device
                            .set_blocking_mode(true)
                            .map_err(|e| format!("Failed to set blocking mode: {}", e))?;

                        let mut buf = [0u8; RAW_HID_REPORT_SIZE];
                        buf[0] = CMD_PING;
                        buf[1] = 0xAA;
                        buf[2] = 0x55;

                        if let Err(_) = device.write(&buf) {
                            continue;
                        }

                        let mut resp = [0u8; RAW_HID_REPORT_SIZE];
                        match device.read_timeout(&mut resp, 500) {
                            Ok(n) if n > 0 && resp[0] == CMD_PING && resp[1] == 0xAA => {
                                self.device = Some(device);
                                return Ok(());
                            }
                            _ => continue,
                        }
                    }
                    Err(_) => continue,
                }
            }
        }

        Err("No KQB device with companion protocol found".to_string())
    }

    pub fn disconnect(&mut self) {
        self.device = None;
    }

    pub fn is_connected(&self) -> bool {
        self.device.is_some()
    }

    fn send_recv(&self, data: &[u8]) -> Result<[u8; RAW_HID_REPORT_SIZE], String> {
        let device = self
            .device
            .as_ref()
            .ok_or_else(|| "Not connected".to_string())?;

        let mut buf = [0u8; RAW_HID_REPORT_SIZE];
        let len = data.len().min(RAW_HID_REPORT_SIZE);
        buf[..len].copy_from_slice(&data[..len]);

        device
            .write(&buf)
            .map_err(|e| format!("Write failed: {}", e))?;

        let mut resp = [0u8; RAW_HID_REPORT_SIZE];
        let n = device
            .read_timeout(&mut resp, 1000)
            .map_err(|e| format!("Read failed: {}", e))?;

        if n == 0 {
            return Err("Read timeout".to_string());
        }

        Ok(resp)
    }

    pub fn get_status(&self) -> Result<DeviceStatus, String> {
        let resp = self.send_recv(&[CMD_GET_STATUS])?;

        Ok(DeviceStatus {
            default_layer: resp[1],
            layer_state: u32::from_le_bytes([resp[2], resp[3], resp[4], resp[5]]),
            os_override: resp[6],
            layer_count: resp[7],
        })
    }

    pub fn set_layer(&self, layer: u8) -> Result<(), String> {
        let resp = self.send_recv(&[CMD_SET_LAYER, layer])?;
        if resp[0] == 0xFF {
            return Err(format!("Invalid layer: {}", layer));
        }
        Ok(())
    }

    pub fn set_os_override(&self, override_type: u8) -> Result<(), String> {
        let resp = self.send_recv(&[CMD_SET_OS_OVERRIDE, override_type])?;
        if resp[0] == 0xFF {
            return Err(format!("Invalid override type: {}", override_type));
        }
        Ok(())
    }

    pub fn get_version(&self) -> Result<DeviceVersion, String> {
        let resp = self.send_recv(&[CMD_GET_VERSION])?;

        let mut uid = [0u8; 8];
        uid.copy_from_slice(&resp[3..11]);

        Ok(DeviceVersion {
            protocol_version: u16::from_be_bytes([resp[1], resp[2]]),
            uid,
        })
    }
}

pub type HidState = Mutex<HidConnection>;

pub fn create_hid_state() -> HidState {
    Mutex::new(HidConnection::new())
}
