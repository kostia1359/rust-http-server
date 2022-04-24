use std::env;
use std::process;

use test_server::Config;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    println!("Starting test server...");
    println!("{:?}", config)
}
