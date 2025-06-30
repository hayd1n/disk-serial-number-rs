#![cfg(windows)]

use crate::{DiskInfo, DiskInfoProvider, ProviderError};
use serde::Deserialize;
use wmi::{COMLibrary, WMIConnection};

// Define a structure to map to the fields of WMI Class `Win32_DiskDrive`
// Use `serde(rename_all = "PascalCase")` to automatically map WMI naming conventions
#[allow(non_camel_case_types)]
#[derive(Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Win32_DiskDrive {
    // Device system ID, e.g., "\\.\PHYSICALDRIVE0"
    device_id: String,
    // Hard disk model
    model: String,
    // Serial number, may not exist
    serial_number: Option<String>,

    media_type: Option<String>,
}

pub struct WindowsProvider;

impl DiskInfoProvider for WindowsProvider {
    fn get_all_disks() -> Result<Vec<DiskInfo>, ProviderError> {
        // Initialize COM library. This is a necessary prerequisite for communicating with WMI.
        let com_lib = COMLibrary::new()?;

        // Create a WMI connection to the "root\CIMV2" namespace.
        let wmi_con = WMIConnection::new(com_lib.into())?;

        // Execute WQL (WMI Query Language) query
        // wmi.query() will automatically deserialize results into our defined Win32_DiskDrive structure
        let results: Vec<Win32_DiskDrive> = wmi_con.query()?;

        // Convert WMI results to our common DiskInfo structure
        let disks = results
            .into_iter()
            .map(|dev| {
                let serial = dev
                    .serial_number
                    .and_then(|s| if s.trim().is_empty() { None } else { Some(s) });
                let removable = dev.media_type.and_then(|s| {
                    if s.trim().is_empty() {
                        None
                    } else {
                        let s_lower = s.to_lowercase();
                        Some(
                            s_lower.contains("removable media")
                                || s_lower.contains("external hard disk media"),
                        )
                    }
                });
                DiskInfo {
                    // We use DeviceID as name, as it best represents the device in the system
                    name: dev.device_id,
                    model: Some(dev.model),
                    serial_number: serial,
                    removable,
                }
            })
            .collect();

        Ok(disks)
    }
}
