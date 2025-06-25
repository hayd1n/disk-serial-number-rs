#![cfg(target_os = "macos")]

use crate::{DiskInfo, DiskInfoProvider, ProviderError};
use serde::Deserialize;
use std::collections::HashSet;
use std::process::Command;

// Define structures for parsing `system_profiler` JSON ---

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct PhysicalDriveInfo {
    // Here we only need the model, as serial number is not available here
    #[serde(rename = "device_name")]
    model: String,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct MacVolume {
    // Volume's BSD name, e.g., "disk3s1"
    bsd_name: String,
    // Nested physical drive information
    physical_drive: PhysicalDriveInfo,
}

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct MacStorageOutput {
    #[serde(rename = "SPStorageDataType")]
    volumes: Vec<MacVolume>,
}

pub struct MacosProvider;

impl DiskInfoProvider for MacosProvider {
    fn get_all_disks() -> Result<Vec<DiskInfo>, ProviderError> {
        // Get unique physical disk identifiers ---

        let profiler_output = Command::new("system_profiler")
            .args(["SPStorageDataType", "-json"])
            .output()?;

        if !profiler_output.status.success() {
            return Err(ProviderError::CommandUnsuccessful(
                String::from_utf8_lossy(&profiler_output.stderr).into(),
            ));
        }

        let json_str = String::from_utf8_lossy(&profiler_output.stdout);
        let parsed: MacStorageOutput = serde_json::from_str(&json_str)?;

        // Use HashSet to automatically handle duplicate physical drives
        let mut physical_disk_ids = HashSet::new();
        for volume in parsed.volumes {
            // Extract "disk3" from "disk3s1"

            // Check if the string starts with "disk", if so, get the remaining part
            if let Some(rest) = volume.bsd_name.strip_prefix("disk") {
                // `rest` is now "3s1", "10s2", "4", etc.

                // Count how many consecutive digit characters from the beginning
                let number_part_len = rest.chars().take_while(|c| c.is_ascii_digit()).count();

                // Based on the calculated length, safely rebuild the physical disk ID
                // For example, get "3" from "3s1", then combine with "disk" to form "disk3"
                let disk_id = format!("disk{}", &rest[..number_part_len]);

                physical_disk_ids.insert(disk_id);
            }
        }

        // Iterate through unique disk IDs, use `diskutil` to query detailed information ---
        let mut disks_info = Vec::new();

        for disk_id in physical_disk_ids {
            let diskutil_output = Command::new("diskutil")
                .arg("info")
                .arg(&disk_id)
                .output()?;

            if !diskutil_output.status.success() {
                // If querying a disk fails, we can choose to ignore it or return an error
                // Here we choose to print a warning and continue
                eprintln!(
                    "Warning: Unable to get detailed information for '{}'.",
                    disk_id
                );
                continue;
            }

            let output_str = String::from_utf8_lossy(&diskutil_output.stdout);

            // Manually parse the key-value pair output from `diskutil info`
            let mut model: Option<String> = None;
            let mut serial_number: Option<String> = None;

            for line in output_str.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim();
                    let value = value.trim();

                    if value.is_empty() {
                        continue;
                    }

                    match key {
                        "Device / Media Name" => model = Some(value.to_string()),
                        "Serial Number" => serial_number = Some(value.to_string()),
                        "Disk / Partition UUID" => serial_number = Some(value.to_string()),
                        "Volume UUID" => serial_number = Some(value.to_string()),
                        _ => {}
                    }
                }
            }

            // Only add to list if model was successfully obtained
            if let Some(m) = model {
                disks_info.push(DiskInfo {
                    name: disk_id, // Use BSD names like "disk3" as unique name
                    model: Some(m),
                    serial_number,
                });
            }
        }

        Ok(disks_info)
    }
}
