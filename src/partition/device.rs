use serde::{Serialize, Deserialize};
#[derive(PartialEq, Deserialize, Serialize, Debug)]
struct Device {
    use_percentage: Option<String>,
}

impl Default for Device {
    fn default() -> Device {
        Device{
        use_percentage: None
        }
    }
}