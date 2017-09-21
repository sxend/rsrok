extern crate clap;
extern crate iron;
extern crate router;

use clap::{Arg, App, SubCommand};
use iron::*;
use iron::typemap;
use iron::headers::*;
use router::Router;

fn main() {
    let matches = App::new("rsrok")
        .version("0.1.0")
        .arg(Arg::with_name("host")
            .long("host")
            .value_name("HOST")
            .help("set expose host")
            .takes_value(true))
        .arg(Arg::with_name("src")
            .long("src")
            .value_name("SRC")
            .help("set source server")
            .takes_value(true))
        .get_matches();
    println!("host: {}, src: {}", matches.value_of("host").unwrap(), matches.value_of("src").unwrap());

}