use serde::{de::DeserializeOwned, Deserialize, Serialize};
use winreg::enums::*;
use winreg::RegKey;
use wmi::*;

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_BIOS")]
#[serde(rename_all = "PascalCase")]
struct BIOS {
    serial_number: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_BaseBoard")]
#[serde(rename_all = "PascalCase")]
struct Motherboard {
    serial_number: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_DiskDrive")]
#[serde(rename_all = "PascalCase")]
struct DiskDrive {
    serial_number: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_Processor")]
#[serde(rename_all = "PascalCase")]
struct Processor {
    processor_id: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_OperatingSystem")]
#[serde(rename_all = "PascalCase")]
struct OperatingSystem {
    serial_number: String,
}

#[derive(Debug, Serialize)]
pub enum HwidError {
    WmiComError,
    WmiConnectionError,
    WmiQueryError,
    WmiNoResult,
    MacAddressError,
    NoMacAddress,
}

pub struct HardwareInfo {
    pub mac_address: String,
    pub bios: String,
    pub motherboard: String,
    pub disk_drive: String,
    pub processor_id: String,
    pub operating_system: String,
    pub computer_hwid: String,
    pub machine_guid: String,
}

pub fn get_hardware_info() -> Result<HardwareInfo, HwidError> {
    let mac_address = mac_address::get_mac_address().map_err(|_| HwidError::MacAddressError)?;
    let mac_address = mac_address.ok_or(HwidError::NoMacAddress)?;

    let bios = query_wmi::<BIOS>()?;
    let motherboard = query_wmi::<Motherboard>()?;
    let disk_drive = query_wmi::<DiskDrive>()?;
    let processor = query_wmi::<Processor>()?;
    let operating_system = query_wmi::<OperatingSystem>()?;
    let computer_hwid = get_reg_key(
        "SYSTEM\\CurrentControlSet\\Control\\SystemInformation",
        "ComputerHardwareId",
    );
    let machine_guid = get_reg_key("Software\\Microsoft\\Cryptography", "MachineGuid");

    Ok(HardwareInfo {
        mac_address: mac_address.to_string(),
        bios: bios.serial_number,
        motherboard: motherboard.serial_number,
        disk_drive: disk_drive.serial_number,
        processor_id: processor.processor_id,
        operating_system: operating_system.serial_number,
        computer_hwid,
        machine_guid,
    })
}

fn query_wmi<T>() -> Result<T, HwidError>
where
    T: DeserializeOwned,
{
    let instance = COMLibrary::without_security().map_err(|_| HwidError::WmiComError)?;
    let wmi_con = WMIConnection::new(instance).map_err(|_| HwidError::WmiConnectionError)?;

    let mut results: Vec<T> = wmi_con.query().map_err(|_| HwidError::WmiQueryError)?;

    if results.first().is_none() {
        return Err(HwidError::WmiNoResult);
    }

    let first = results.remove(0);

    Ok(first)
}

fn get_reg_key(path: &str, key: &str) -> String {
    let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
    let sub_key = hklm.open_subkey(path).unwrap();
    sub_key.get_value(key).unwrap()
}
