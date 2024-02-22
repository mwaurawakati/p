# p
use std::option::Option;

#[derive(Debug, Clone)]
struct OsProber {
    long: String,
    label: String,
    type_: String,
    subpath: Option<String>,
    version: Option<String>,
}
use std::process::Command;
use std::str;

fn probe_os() -> Vec<OsProber> {
    // This function attempts to run `os-prober` with `sudo`.
    // Ensure that the user running this Rust program has the necessary permissions.
    
    let output = Command::new("sudo")
        .arg("os-prober")
        .output()
        .expect("Failed to execute process");

    // Check for command execution success
    if !output.status.success() {
        eprintln!("Error running os-prober. Make sure you have the necessary permissions.");
        return vec![];
    }

    let output_str = match str::from_utf8(&output.stdout) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to parse output: {}", e);
            return vec![];
        }
    };

    output_str.lines().map(|line| {
        let parts: Vec<&str> = line.split(':').collect();
        OsProber {
            long: parts.get(0).unwrap_or(&"").to_string(),
            label: parts.get(1).unwrap_or(&"").to_string(),
            type_: parts.get(2).unwrap_or(&"").to_string(),
            subpath: parts.get(3).map(|s| s.to_string()),
            version: parts.get(4).map(|s| s.to_string()),
        }
    }).collect()
}

fn main() {
    let os_entries = probe_os();
    for entry in os_entries {
        println!("{:?}", entry);
    }
}
