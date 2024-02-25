mod partition;
use partition::fileinfo;
fn main() {
    let fi = fileinfo::get_file_info().unwrap();
    println!("{}", fi);
}

