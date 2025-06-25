#![cfg(target_os = "linux")]

use crate::{DiskInfo, DiskInfoProvider, ProviderError};
use serde::Deserialize;
use std::process::Command;

#[derive(Deserialize)]
struct LsblkDevice {
    name: String,
    model: Option<String>,
    serial: Option<String>,
    // We only select devices of type "disk"
    #[serde(rename = "type")]
    device_type: String,
}

#[derive(Deserialize)]
struct LsblkOutput {
    blockdevices: Vec<LsblkDevice>,
}

pub struct LinuxProvider;

impl DiskInfoProvider for LinuxProvider {
    fn get_all_disks() -> Result<Vec<DiskInfo>, ProviderError> {
        let output = Command::new("lsblk")
            .args(["-o", "NAME,MODEL,SERIAL,TYPE", "-J", "-b", "-d"]) // -d: only show disks themselves, not partitions
            .output()?;

        if !output.status.success() {
            return Err(ProviderError::CommandUnsuccessful(
                String::from_utf8_lossy(&output.stderr).into(),
            ));
        }

        let json_str = String::from_utf8_lossy(&output.stdout);
        let parsed: LsblkOutput = serde_json::from_str(&json_str)?;

        let disks = parsed
            .blockdevices
            .into_iter()
            .filter(|dev| dev.device_type == "disk") // Filter devices with type "disk"
            .map(|dev| DiskInfo {
                name: dev.name,
                model: dev
                    .model
                    .and_then(|m| if m.trim().is_empty() { None } else { Some(m) }),
                serial_number: dev
                    .serial
                    .and_then(|s| if s.trim().is_empty() { None } else { Some(s) }),
            })
            .collect();

        Ok(disks)
    }
}
