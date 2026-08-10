use super::Result;
use serialport::{SerialPortType, UsbPortInfo};

const FLIPPER_VID: u16 = 0x0483;
const FLIPPER_PID: u16 = 0x5740;

pub struct FlipperPort {
    pub port: String,
    pub label: String,
}

fn looks_like_flipper(info: &UsbPortInfo) -> bool {
    if info.vid == FLIPPER_VID && info.pid == FLIPPER_PID {
        return true;
    }
    let m = info.manufacturer.as_deref().unwrap_or("").to_lowercase();
    let p = info.product.as_deref().unwrap_or("").to_lowercase();
    m.contains("flipper") || p.contains("flipper")
}

fn label_for(port: &str, usb: &UsbPortInfo) -> String {
    let name = usb
        .product
        .clone()
        .or_else(|| usb.manufacturer.clone())
        .unwrap_or_else(|| "Flipper Zero".to_string());
    match &usb.serial_number {
        Some(sn) => format!("{name}  ({port}, serial {sn})"),
        None => format!("{name}  ({port})"),
    }
}

pub fn find_all_flippers() -> Result<Vec<FlipperPort>> {
    let ports = serialport::available_ports()?;
    let mut out = Vec::new();
    for p in ports {
        if let SerialPortType::UsbPort(usb) = &p.port_type
            && looks_like_flipper(usb)
        {
            let label = label_for(&p.port_name, usb);
            out.push(FlipperPort {
                port: p.port_name,
                label,
            });
        }
    }
    Ok(out)
}
