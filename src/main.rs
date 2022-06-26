
mod em8080;

use crate::em8080::Em8080;

fn main() {

    println!("Hello, world!");
    
    let mut sys = Em8080::new();
    println!("{:#?}", sys);
}
