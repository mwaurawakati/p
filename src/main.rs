mod partition;
//use std::error::Error;
use tracing::{error, info};
use tracing_subscriber;

fn main() {
    tracing_subscriber::fmt::init();
    let matches = partition::utils::check_for_errors("/dev/sda1");
    match matches {
        Ok(m) => {
            info!(m);
        }
        Err(e) => {
            error!("{}", e.to_string());
        }
    }
}
