use std::env;
use std::process;

use test_server::Config;
use test_server::run_server;

fn main() {
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Problem parsing arguments: {}", err);
        process::exit(1);
    });

    println!("Server is listening on port {}", config.port);
    run_server(&config);
}
