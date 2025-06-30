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
                if let Some(removable) = disk.removable {
                    println!(" - Removable: {}", removable);
                } else {
                    println!(" - Removable: Unknown");
                }
            }
        }
        Err(e) => {
            eprintln!("Error retrieving disk information: {}", e);
        }
    }
}
