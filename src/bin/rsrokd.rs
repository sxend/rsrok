extern crate clap;
use clap::{Arg, App, SubCommand};

fn main() {
    let matches = App::new("rsrokd")
        .version("0.1.0")
        .arg(Arg::with_name("host")
            .long("host")
            .value_name("HOST")
            .help("set allowed host")
            .takes_value(true))
        .get_matches();
    matches.value_of("host");
    println!("host: {}", matches.value_of("host").unwrap());
}