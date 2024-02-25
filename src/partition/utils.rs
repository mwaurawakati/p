use lazy_static::lazy_static;
use std::{
    any::Any,
    collections::HashMap,
    fs,
    os::unix::fs::FileTypeExt, // This trait provides the is_block_device method
    path::{Path, PathBuf},
};
//use serde::{Serialize, Deserialize};
use serde_json;
use tracing::{debug, error, info};

#[allow(dead_code)]
fn path_exists(path_str: &str) -> bool {
    Path::new(path_str).exists()
}
#[allow(dead_code)]
pub fn check_for_errors(device: &str) -> Result<String, std::io::Error> {
    let output: Result<std::process::Output, std::io::Error> = std::process::Command::new("sudo")
        .arg("e2fsck")
        .arg("-p")
        .arg("-f")
        .arg(device)
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                let s = String::from_utf8_lossy(&output.stdout);
                Ok(s.into())
            } else {
                let s = String::from_utf8_lossy(&output.stdout);
                Err(std::io::Error::new(std::io::ErrorKind::AddrInUse, s))
            }
        }
        Err(_) => todo!(),
    }
}

// Size in bytes
#[allow(dead_code)]
pub fn resize_ext(path: &str, size: i64) -> Result<bool, std::io::Error> {
    let output: Result<std::process::Output, std::io::Error> = std::process::Command::new("sudo")
        .arg("resize2fs")
        .arg(path)
        .arg(format!("{}k", size))
        .output();

    match output {
        Ok(output) => {
            if output.status.success() {
                Ok(true)
            } else {
                Ok(false)
            }
        }
        Err(_) => todo!(),
    }
}

// Size in bytes
#[allow(dead_code)]
pub fn resize_ntfs(path: &str, size: i64) -> Result<bool, std::io::Error> {
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
            } else {
                Ok(false)
            }
        }
        Err(_) => todo!(),
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
lazy_static! {
    pub static ref MKFS_COMMANDS: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("btrfs", "mkfs.btrfs");
        m.insert("ext2", "mkfs.ext2");
        m.insert("ext3", "mkfs.ext3");
        m.insert("ext4", "mkfs.ext4");
        m.insert("f2fs", "mkfs.f2fs");
        m.insert("fat", "mkfs.vfat");
        m.insert("fat12", "mkfs.vfat");
        m.insert("fat16", "mkfs.vfat");
        m.insert("fat32", "mkfs.vfat");
        m.insert("vfat", "mkfs.vfat");
        m.insert("jfs", "jfs_mkfs");
        m.insert("ntfs", "mkntfs");
        m.insert("reiserfs", "mkfs.reiserfs");
        m.insert("swap", "mkswap");
        m.insert("xfs", "mkfs.xfs");
        m
    };
}
lazy_static! {
    pub static ref SPEFIC_TO_FAMILY: HashMap<&'static str, &'static str> = {
        let mut m = HashMap::new();
        m.insert("ext2", "ext");
        m.insert("ext3", "ext");
        m.insert("ext4", "ext");
        m.insert("fat12", "fat");
        m.insert("fat16", "fat");
        m.insert("fat32", "fat");
        m.insert("vfat", "fat");
        m
    };
}
lazy_static! {
    pub static ref LABEL_LENGTH_LIMITS: HashMap<&'static str, i32>  = {
        let mut m = HashMap::new();
        m.insert("btrfs", 256);
        m.insert("ext", 16);
        m.insert("f2fs", 512);  // https://docs.kernel.org/filesystems/f2fs.html
        m.insert("fat", 11);
        m.insert("jfs", 16); // see jfs_tune manpage
        m.insert("ntfs", 32);
        m.insert("reiserfs", 16);
        m.insert("swap", 15);  // not in manpages, found experimentally
        m.insert("xfs", 12);
        m
    };
}

#[derive(Clone)]
pub enum FlagValue {
    Single(&'static str),
    Double((&'static str, &'static str)),
}

type FlagMapping = HashMap<&'static str, HashMap<&'static str, FlagValue>>;
lazy_static! {
    pub static ref FAMILY_FLAG_MAPPINGS: FlagMapping = {
        let m: FlagMapping = [
            (
                "fatsize",
                HashMap::from([("fat", FlagValue::Double(("-F", "{fatsize}")))]),
            ),
            (
                "force",
                HashMap::from([
                    ("btrfs", FlagValue::Single("--force")),
                    ("ext", FlagValue::Single("-F")),
                    ("f2fs", FlagValue::Single("-f")),
                    ("fat", FlagValue::Single("-I")),
                    ("jfs", FlagValue::Single("-q")),
                    ("ntfs", FlagValue::Single("--force")),
                    ("reiserfs", FlagValue::Single("-f")),
                    ("swap", FlagValue::Single("--force")),
                    ("xfs", FlagValue::Single("-f")),
                ]),
            ),
            (
                "label",
                HashMap::from([
                    ("btrfs", FlagValue::Double(("--label", "{label}"))),
                    ("ext", FlagValue::Double(("-L", "{label}"))),
                    ("f2fs", FlagValue::Double(("-l", "{label}"))),
                    ("fat", FlagValue::Double(("-n", "{label}"))),
                    ("jfs", FlagValue::Double(("-L", "{label}"))),
                    ("ntfs", FlagValue::Double(("--label", "{label}"))),
                    ("reiserfs", FlagValue::Double(("--label", "{label}"))),
                    ("swap", FlagValue::Double(("--label", "{label}"))),
                    ("xfs", FlagValue::Double(("-L", "{label}"))),
                ]),
            ),
            (
                "quiet",
                HashMap::from([
                    ("ext", FlagValue::Single("-q")),
                    ("f2fs", FlagValue::Single("-q")),
                    ("ntfs", FlagValue::Single("-q")),
                    ("reiserfs", FlagValue::Single("-q")),
                    ("xfs", FlagValue::Single("--quiet")),
                ]),
            ),
            (
                "sectorsize",
                HashMap::from([
                    ("btrfs", FlagValue::Double(("--sectorsize", "{sectorsize}"))),
                    ("ext", FlagValue::Double(("-b", "{sectorsize}"))),
                    ("f2fs", FlagValue::Double(("-w", "{sectorsize}"))),
                    ("fat", FlagValue::Double(("-S", "{sectorsize}"))),
                    ("ntfs", FlagValue::Double(("--sector-size", "{sectorsize}"))),
                    (
                        "reiserfs",
                        FlagValue::Double(("--block-size", "{sectorsize}")),
                    ),
                    ("xfs", FlagValue::Double(("-s", "{sectorsize}"))),
                ]),
            ),
            (
                "uuid",
                HashMap::from([
                    ("btrfs", FlagValue::Double(("--uuid", "{uuid}"))),
                    ("ext", FlagValue::Double(("-U", "{uuid}"))),
                    ("f2fs", FlagValue::Double(("-U", "{uuid}"))),
                    ("reiserfs", FlagValue::Double(("--uuid", "{uuid}"))),
                    ("swap", FlagValue::Double(("--uuid", "{uuid}"))),
                    ("xfs", FlagValue::Double(("-m", "uuid={uuid}"))),
                ]),
            ),
        ]
        .iter()
        .cloned()
        .collect();
        m
    };
}

type InnerMap = HashMap<&'static str, Option<HashMap<&'static str, ()>>>;
type OuterMap = HashMap<&'static str, InnerMap>;
lazy_static! {
    pub static ref RELEASE_FLAG_MAPPING_OVERRIDE: OuterMap = {
        let m: OuterMap = [
            (
                "precise",
                [
                    ("force", Some([("btrfs", ())].iter().cloned().collect())),
                    ("uuid", Some([("btrfs", ())].iter().cloned().collect())),
                ]
                .iter()
                .cloned()
                .collect(),
            ),
            (
                "trusty",
                [(
                    "uuid",
                    Some([("btrfs", ()), ("xfs", ())].iter().cloned().collect()),
                )]
                .iter()
                .cloned()
                .collect(),
            ),
        ]
        .iter()
        .cloned()
        .collect();
        m
    };
}
#[allow(dead_code)]
pub fn perform_resize(kname: &str, resize: HashMap<String, Box<dyn Any>>) {
    let path = kname_to_path(kname);
    let fstype = resize.get("fstype");
    let size = resize.get("'size");
    let direction = resize.get("direction");
    debug!(
        "Resizing {} of type {:#?} {:#?} to {:#?}",
        path, fstype, direction, size
    );
    if let Some(&resize_function) = RESIZERS.get("fstype") {
        // Now you can call resize_function, which is a function pointer
        // Assuming `size` is of type Option<&Box<dyn Any>>
        match size {
            Some(size_box) => {
                // Attempt to downcast to i64
                if let Some(size_value) = size_box.downcast_ref::<i64>() {
                    match resize_function(path.as_str(), *size_value) {
                        Ok(_) => {
                            info!("resize successful")
                        }
                        Err(e) => {
                            error!("resize unsuccessful with error:{}", e.to_string())
                        }
                    }
                } else {
                    error!("invalid size")
                }
            }
            None => {
                error!("unknown error")
            }
        }
    } else {
        eprintln!("No resize function found for fstype");
    }
}

// converts a kname to a path in /dev, taking special devices and unusual
// naming schemes into account
#[allow(dead_code)]
pub fn kname_to_path(kname: &str) -> String {
    let path = Path::new(kname);

    // If the path exists and is a valid device, return its realpath
    if path.exists() && is_valid_device(String::from(kname)) {
        return match fs::canonicalize(path) {
            Ok(real_path) => real_path.to_str().unwrap_or("").to_string(),
            Err(_) => "".to_string(), // Provide a default value in case of an error
        };
    };
    // Handling special devices and unusual naming schemes
    let mut dev_path = PathBuf::from("/dev");
    let components: Vec<&str> = kname.split('!').collect();
    for component in components {
        dev_path.push(component);
    }
    match fs::canonicalize(&dev_path) {
        Ok(real_path) => {
            // Make sure the path is correct
            if !real_path.exists() || !is_valid_device(real_path.to_str().unwrap_or("").to_string(),) {
                return String::new();
            } else {
                return String::new();
            }
        }
        Err(_) => return String::new(),
    }
}
#[allow(dead_code)]
fn get_dev_name_entry(devname: &str) -> (String, String) {
    let parts: Vec<&str> = devname.split("/dev/").collect();
    let bname = parts.last().unwrap_or(&"").to_string();
    let path = format!("/dev/{}", bname);

    (bname, path)
}
#[allow(dead_code)]
fn is_valid_device(devname: String) -> bool {
    let devent = get_dev_name_entry(devname.as_str()).1;
    is_block_device(devent.as_str())
}
#[allow(dead_code)]
fn is_block_device(path: &str) -> bool {
    match fs::metadata(path) {
        Ok(metadata) => metadata.file_type().is_block_device(),
        Err(e) => {
            // Check for a "file not found" error; this is platform-specific
            if e.kind() != std::io::ErrorKind::NotFound {
                panic!(
                    "Unexpected error when checking if path is a block device: {:?}",
                    e
                );
            }
            false
        }
    }
}
#[allow(dead_code)]
fn construct_real_path(kname: &str) -> String {
    let parts: Vec<&str> = kname.split('!').collect();
    let mut path = PathBuf::from("/dev");
    for part in parts {
        path.push(part);
    }
    // Canonicalize the path to resolve it to an absolute path
    match path.canonicalize() {
        Ok(real_path) => real_path.to_str().unwrap_or("").to_string(),
        Err(_) => "".to_string(), // Provide a default value in case of an error
    }
}

pub fn unmarshal_json<T: for<'de> serde::Deserialize<'de>>(data: &str, v: &mut Option<T>) -> Result<(), serde_json::Error> {
    *v = Some(serde_json::from_str(data)?);
    Ok(())
}

pub fn marshal_json<T: serde::Serialize>(v: &T) -> Result<String, serde_json::Error> {
    serde_json::to_string(v)
}