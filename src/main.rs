mod partition;
use tracing::{info, error};
use tracing_subscriber;
use std::error::Error;

fn main() {
    tracing_subscriber::fmt::init();
    let matches = partition::utils::check_for_errors("/dev/sda1");
    match matches {
        Ok(m) => {
            info!(m);
        }
        Err(e)=>{
            error!( "{}", e.description());
        }
    }
}
