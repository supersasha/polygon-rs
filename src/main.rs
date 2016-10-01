extern crate polygon;

use std::env::{args};
use std::mem::size_of;
use polygon::polygon::World;

fn main() {
    let arg = args().nth(1).unwrap();
    //println!("Args count = {}", _args.count());
    //println!("Args[1]: {}", arg);
    println!("Hello, polygon!");

    println!("sizeof(World) = {}", size_of::<World>());
    polygon::view::run(&arg);
}
