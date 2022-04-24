#[derive(Debug)]
pub struct Config {
    pub root_dir: String,
    pub port: i32,
}

impl Config {
    pub fn new(mut args: std::env::Args) -> Result<Config, &'static str> {
        args.next();

        let port = match args.next() {
            Some(arg) => match arg.parse::<i32>() {
                Ok(val) => val,
                Err(_) => return Err("Unable to parse port")
            },
            None => return Err("Didn't get port"),
        };

        let root_dir = match args.next() {
            Some(arg) => arg,
            None => "/home".to_string(),
        };

        if port < 0 || port > 65535 {
            return Err("Invalid port")
        }

        Ok(Config {
            port,
            root_dir
        })
    }
}

