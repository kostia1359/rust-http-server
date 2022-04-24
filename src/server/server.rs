use std::fs;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};

use std::path::Path;

use regex::Regex;


use crate::ThreadPool;
use crate::log;
use crate::Config;

pub fn run_server(config: &Config) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.port)).unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| {
            handle_connection(stream);
        });
    }

    log("Shutting down.");
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);
    
    match get_path(&request) {
        Ok(path) => match get_full_path_to_file(&path) {
            Ok(full_path) => {
                
            },
            Err(err) => send_error(stream, err)
        },
        Err(err) => send_error(stream, err),
    }

}

enum ErrorCode {
    NotImplemented,
    NotFound,
    BadRequest,    
}

fn get_error_details(error_code: ErrorCode) -> (i32, String) {
    match error_code {
        ErrorCode::NotImplemented => (501, "Not Implemented".to_owned()),
        ErrorCode::NotFound => (404, "Not Found".to_owned()),
        ErrorCode::BadRequest => (400, "Bad Request".to_owned()),
    }
}

fn get_path(request: &str) -> Result<String, ErrorCode> {
    let re = Regex::new(r"GET (.*?) HTTP/1.1\r\n").unwrap();

    match re.captures(request){
        Some(match_value) => Ok(match_value[1].to_owned()),
        None => Err(ErrorCode:: NotImplemented)
    }
}

fn get_full_path_to_file<'a>(full_path: &'a str) -> Result<&'a str, ErrorCode> {
    let mut path = Path::new(full_path);

    if path.exists() {
        if !path.is_dir() {
            return Ok(full_path);
        }

        return Err(ErrorCode::BadRequest)
    }

    path = Path::new(&full_path[1..]);
    if path.exists() {
        if !path.is_dir() {
            return Ok(&full_path[1..]);
        }
        return Err(ErrorCode::BadRequest)
    }

    return Err(ErrorCode::NotFound);
}

fn send_error(mut stream: TcpStream, error_code: ErrorCode){
    let (status, description) = get_error_details(error_code);
    let response = format!(
        "HTTP/1.1 {} {}\r\n\r\n",
        status, 
        description
    );

    stream.write(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}