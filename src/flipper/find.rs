use super::{FlipperError, Result};
use serialport::{SerialPortType, UsbPortInfo};

const FLIPPER_VID: u16 = 0x0483;
const FLIPPER_PID: u16 = 0x5740;

fn looks_like_flipper(info: &UsbPortInfo) -> bool {
    if info.vid == FLIPPER_VID && info.pid == FLIPPER_PID {
        return true;
    }
    let m = info.manufacturer.as_deref().unwrap_or("").to_lowercase();
    let p = info.product.as_deref().unwrap_or("").to_lowercase();
    m.contains("flipper") || p.contains("flipper")
}

pub fn find_flipper_port() -> Result<String> {
    find_all_flipper_ports()?
        .into_iter()
        .next()
        .ok_or(FlipperError::NotFound)
}

pub fn find_all_flipper_ports() -> Result<Vec<String>> {
    let ports = serialport::available_ports()?;
    let mut out = Vec::new();
    for p in ports {
        if let SerialPortType::UsbPort(usb) = &p.port_type
            && looks_like_flipper(usb)
        {
            out.push(p.port_name);
        }
    }
    Ok(out)
}
