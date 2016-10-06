extern crate polygon;
extern crate rand;

use std::env::{args};
use std::mem::size_of;
use polygon::polygon::World;
use rand::{thread_rng, Rng};

fn main() {
    let arg = args().nth(1).unwrap();
    //println!("Args count = {}", _args.count());
    //println!("Args[1]: {}", arg);
    println!("Hello, polygon!");

    println!("sizeof(World) = {}", size_of::<World>());
    let mut rng = thread_rng();
    println!("rnd = {}", rng.gen_range(0, 10));
    println!("rnd = {}", rng.gen_range(0, 10));
    polygon::view::run(&arg);
}
