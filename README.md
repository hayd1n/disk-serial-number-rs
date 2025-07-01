# Disk Serial Number for Rust

A Rust library to get disk serial numbers across different platforms.

## Supported Platforms

- Linux
- Windows
- macOS

## Supported Information

- Disk name
- Disk model
- Disk serial number
- Is removable?

## Limitations

Due to differences in operating systems and hardware platforms, the method for obtaining the disk serial number may vary. **The same hard drive may have different serial numbers on different systems.**

Due to the lack of testing platforms, macOS currently only supports NVME and Serial ATA (aka SATA) drives.

If you find a better method, please submit a PR.

## Usage

### Add the following to your `Cargo.toml`:

```toml
[dependencies]
disk-serial-number = "*"
```

### Example

See the full example in [examples/simple.rs](examples/simple.rs).

```rust
use disk_serial_number::get_all_disks;

fn main() {
    let disks = get_all_disks();
    match disks {
        Ok(disk_list) => {
            for disk in disk_list {
                println!("Disk Name: {}", disk.name);
                if let Some(model) = &disk.model {
                    println!(" - Model: {}", model);
                }
                if let Some(serial) = &disk.serial_number {
                    println!(" - Serial Number: {}", serial);
                }
            }
        }
        Err(e) => {
            eprintln!("Error retrieving disk information: {}", e);
        }
    }
}
```
