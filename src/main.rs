extern crate polygon;

use std::env::{args};

fn main() {
    let arg = args().nth(1).unwrap();
    //println!("Args count = {}", _args.count());
    //println!("Args[1]: {}", arg);
    println!("Hello, polygon!");
    //polygon::polygon::run(1000000000);
    polygon::view::run(&arg);
}
