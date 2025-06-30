#![cfg(target_os = "macos")]

use crate::{DiskInfo, DiskInfoProvider, ProviderError};
use serde::Deserialize;
use std::process::Command;

// Define structures for parsing `system_profiler` JSON ---

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct MacDisk {
    // Usually the name is something like "Apple SSD AP0123Q Media"
    #[serde(rename = "_name")]
    name: String,
    // Volume's BSD name, e.g., "disk3s1"
    bsd_name: String,
    device_model: String,
    device_serial: String,
    removable_media: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct MacDiskItems {
    #[serde(rename = "_name")]
    name: String,
    #[serde(rename = "_items")]
    items: Vec<MacDisk>,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct MacDisksOutput {
    #[serde(rename = "SPNVMeDataType")]
    nvme_items: Vec<MacDiskItems>,
    #[serde(rename = "SPSerialATADataType")]
    sata_items: Vec<MacDiskItems>,
}

pub struct MacosProvider;

impl DiskInfoProvider for MacosProvider {
    fn get_all_disks() -> Result<Vec<DiskInfo>, ProviderError> {
        let profiler_output = Command::new("system_profiler")
            .args(["SPNVMeDataType", "SPSerialATADataType", "-json"]) // TODO: support more types
            .output()?;

        if !profiler_output.status.success() {
            return Err(ProviderError::CommandUnsuccessful(
                String::from_utf8_lossy(&profiler_output.stderr).into(),
            ));
        }

        let json_str = String::from_utf8_lossy(&profiler_output.stdout);
        let parsed: MacDisksOutput = serde_json::from_str(&json_str)?;

        let mut disks_info = Vec::new();

        // Helper function to process disk items
        let process_disk_items = |items: &[MacDiskItems], disks_info: &mut Vec<DiskInfo>| {
            for item in items {
                for disk in &item.items {
                    disks_info.push(DiskInfo {
                        name: disk.bsd_name.trim().to_string(),
                        model: Some(disk.device_model.trim().to_string()),
                        serial_number: Some(disk.device_serial.trim().to_string()),
                        removable: Some(disk.removable_media == "yes"),
                    });
                }
            }
        };

        process_disk_items(&parsed.nvme_items, &mut disks_info);
        process_disk_items(&parsed.sata_items, &mut disks_info);

        Ok(disks_info)
    }
}
