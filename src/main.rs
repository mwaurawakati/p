mod partition;

use std::process::{Command, Stdio};

use log::{debug, error};
use std::fs::File;


use std::fs;
use std::path::{Path, PathBuf};

use std::io::{self, Read};

fn do_umount(mountpoint: &Path, recursive: bool, private: bool) -> io::Result<bool> {
    let mp = fs::canonicalize(mountpoint)?;
    let mut ret = false;

    // Load mount points from /proc/mounts
    let mountpoints: Vec<PathBuf> = fs::read_to_string("/proc/mounts")?
        .lines()
        .filter_map(|line| line.split_whitespace().nth(1))
        .map(|mp| PathBuf::from(mp))
        .collect();

    if private {
        for curmp in &mountpoints {
            if curmp == &mp || curmp.starts_with(&mp) {
                let _ = Command::new("mount")
                    .args(&["--make-private", curmp.to_str().unwrap()])
                    .output()?;
            }
        }
    }

    for curmp in mountpoints.iter().rev() {
        if curmp == &mp || (recursive && curmp.starts_with(&mp)) {
            let _ = Command::new("umount")
                .arg(curmp.to_str().unwrap())
                .output()?;
            if curmp == &mp {
                ret = true;
            }
        }
    }

    Ok(ret)
}

fn main() -> io::Result<()> {
    let mountpoint = Path::new("/some/mountpoint");
    let was_mounted = do_umount(mountpoint, true, false)?;
    println!("Was mounted: {}", was_mounted);
    Ok(())
}



