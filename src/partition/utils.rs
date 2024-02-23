use lazy_static::lazy_static;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
fn path_exists(path_str: &str) -> bool {
    Path::new(path_str).exists()
}
use tracing::{debug};
pub fn check_for_errors(device: &str) -> Result<String, std::io::Error> {
    let output: Result<std::process::Output, std::io::Error> = std::process::Command::new("sudo")
    .arg("e2fsck")
    .arg("-p")
    .arg("-f")
    .arg(device)
    .output();

    match output{
        Ok(output) => {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout);
                Ok(s.into())
            }else {
                let s = String::from_utf8_lossy(&output.stdout);
                Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, s))
            }

        }
        Err(_) =>  todo!()
    }
}

// Size in bytes
pub fn resize_ext(path:&str, size:i64) -> Result<bool, std::io::Error> {
    let output: Result<std::process::Output, std::io::Error> = std::process::Command::new("sudo")
    .arg("resize2fs")
    .arg(path)
    .arg(format!("{}k", size))
    .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(true)
            }else{
                Ok(false)
            }
        }
        Err(_) => todo!()
    }
}

// Size in bytes
pub fn resize_ntfs(path:&str, size:i64) -> Result<bool, std::io::Error> {
    let output: Result<std::process::Output, std::io::Error> = std::process::Command::new("sudo")
    .arg("ntfsresize")
    .arg("-f")
    .arg("-s")
    .arg(format!("{}k", size))
    .arg(path)
    .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(true)
            }else{
                Ok(false)
            }
        }
        Err(_) => todo!()
    }
}
type ResizeFn = fn(&str, i64) -> Result<bool, std::io::Error>;
lazy_static! {
    pub static ref RESIZERS: HashMap<&'static str, ResizeFn> = {
        let mut m = HashMap::new();
        m.insert("ext2", resize_ext as ResizeFn);
        m.insert("ext3", resize_ext as ResizeFn);
        m.insert("ext4", resize_ext as ResizeFn);
        m.insert("ntfs", resize_ntfs as ResizeFn);
        m
    };
}

pub fn perform_resize(kname: &str, resize: HashMap<String, Box<dyn std::any::Any>>){
    let path = kname_to_path(kname);
    let fstype = resize.get("fstype");
    let size = resize.get("'size");
    let direction = resize.get("direction");
    debug!("Resizing {} of type {:#?} {:#?} to {:#?}",  path, fstype, direction, size);
    if let Some(&resize_function) = RESIZERS.get("fstype") {
        // Now you can call resize_function, which is a function pointer
        // Assuming `size` is of type Option<&Box<dyn Any>>
match size {
    Some(size_box) => {
        // Attempt to downcast to i64
        if let Some(size_value) = size_box.downcast_ref::<i64>() {
            // Now you have an i64 and can call the function
            match resize_function(path.as_str(), *size_value) {
                Ok(result) => {
                    // Handle success
                },
                Err(e) => {
                    // Handle error
                }
            }
        } else {
            // Handle case where downcast_ref fails because the actual type wasn't i64
        }
    },
    None => {
        // Handle the case where `size` is None
    }
}

    } else {
        eprintln!("No resize function found for fstype");
    }
    
}

// converts a kname to a path in /dev, taking special devices and unusual
// naming schemes into account

pub fn kname_to_path(kname:&str) -> String {
   let mut path = String::new();
    // if given something that is already a dev path, return it
    if path_exists(kname) && is_valid_device(String::from(kname)){
        path = String::from(kname);
        return construct_real_path(path.as_str())
    }
    // adding '/dev' to path is not sufficient to handle cciss devices and
    // possibly other special devices which have not been encountered yet
    path = construct_real_path(kname);
    // make sure path we get is correct
    // if not (os.path.exists(path) and is_valid_device(path)):
        
    return path
}
/*
def get_dev_name_entry(devname):
    """
    convert device name to path in /dev
    """
    bname = devname.split('/dev/')[-1]
    return (bname, "/dev/" + bname)
*/

fn is_valid_device(devname: String) -> bool {
    /*"""
    check if device is a valid device
    """
    devent = get_dev_name_entry(devname)[1]
    return is_block_device(devent)*/
    true
}
/*
def is_block_device(path):
    """
    check if path is a block device
    """
    try:
        return stat.S_ISBLK(os.stat(path).st_mode)
    except OSError as e:
        if not util.is_file_not_found_exc(e):
            raise
    return False

*/
fn construct_real_path(kname: &str) -> String {
    let parts: Vec<&str> = kname.split('!').collect();
    let mut path = PathBuf::from("/dev");
    for part in parts {
        path.push(part);
    }
    // Canonicalize the path to resolve it to an absolute path
    match path.canonicalize() {
        Ok(real_path) => real_path.to_str().unwrap_or("default value").to_string(),
        Err(_) => "default value".to_string(), // Provide a default value in case of an error
    }
}


