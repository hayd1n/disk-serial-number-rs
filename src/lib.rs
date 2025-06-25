#[derive(Debug, Clone)]
pub struct DiskInfo {
    pub name: String,
    pub model: Option<String>,
    pub serial_number: Option<String>,
}

#[cfg(windows)]
use wmi::WMIError;

#[derive(Debug, thiserror::Error)]
pub enum ProviderError {
    #[error("Failed to execute external command: {0}")]
    CommandFailed(#[from] std::io::Error),
    #[error("Command returned non-zero status: {0}")]
    CommandUnsuccessful(String),
    #[error("Failed to parse output: {0}")]
    ParsingFailed(String),
    #[error("JSON processing failed: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Specified device not found")]
    DeviceNotFound,

    #[cfg(windows)]
    #[error("WMI operation failed: {0}")]
    WmiError(#[from] WMIError),
}

pub trait DiskInfoProvider {
    fn get_all_disks() -> Result<Vec<DiskInfo>, ProviderError>;
}

cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        mod linux;
        use linux::LinuxProvider as PlatformProvider;
    } else if #[cfg(target_os = "windows")] {
        mod windows;
        use windows::WindowsProvider as PlatformProvider;
    } else if #[cfg(target_os = "macos")] {
        mod macos;
        use macos::MacosProvider as PlatformProvider;
    } else {
        compile_error!("Unsupported OS");
    }
}

pub fn get_all_disks() -> Result<Vec<DiskInfo>, ProviderError> {
    PlatformProvider::get_all_disks()
}
