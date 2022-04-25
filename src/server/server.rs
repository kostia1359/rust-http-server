use std::fs::File;

use std::io::ErrorKind;
use std::io::prelude::*;
use std::io;

use std::net::{TcpListener, TcpStream};

use std::path::Path;

use regex::Regex;

use crate::ThreadPool;
use crate::log;
use crate::Config;

struct ResponseStream{
    pub stream: TcpStream,
}

impl ResponseStream {
    pub fn new(stream: TcpStream) -> ResponseStream {
        ResponseStream {
            stream: stream
        }
    }

    pub fn finish(mut self) -> TcpStream {
        self.stream.write(b"0\r\n\r\n").unwrap();
        self.stream
    }
}

impl Write for ResponseStream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let size_str = format!("{:x}\r\n", buf.len());

        self.stream.write(size_str.as_bytes())?;
        self.stream.write(buf)?;
        self.stream.write(b"\r\n")?;

        return Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stream.flush()
    }
}

pub fn run_server(config: &Config) {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.port)).unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let root_dir = config.root_dir.to_owned();

        pool.execute(|| {
            handle_connection(stream, root_dir);
        });
    }

    log("Shutting down.");
}

fn handle_connection(mut stream: TcpStream, root_dir: String) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let request = String::from_utf8_lossy(&buffer[..]);
    
    match get_path(&request) {
        Ok(path) => match get_full_path_to_file(&path, &root_dir) {
            Ok(full_path) => {
                let path = Path::new(&full_path);
                let mut read_file = match File::open(path) {
                    Ok(file) => file,
                    Err(err) => match err.kind() {
                        ErrorKind::PermissionDenied => {
                            log(&format!("Error reading file: {}, error: {}", path.to_string_lossy(), err));
                            send_error(stream, ErrorCode::Unauthorized);
                            return;
                        },
                        _ => {
                            log(&format!("Error reading file: {}, error: {}", path.to_string_lossy(), err));
                            send_error(stream, ErrorCode::InternalServerError);
                            return;
                        }
                    }
                };

                let response = format!(
                    "HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\nContent-Type: application/octet-stream\nContent-Disposition: attachment; filename=\"{filename}\"\r\n\r\n",
                    filename = path.file_name().unwrap().to_string_lossy(),
                );
                stream.write(response.as_bytes()).unwrap();

                let mut response_stream = ResponseStream::new(stream);
                io::copy( &mut read_file, &mut response_stream).unwrap();

                let mut stream = response_stream.finish();
                stream.flush().unwrap()

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
    Unauthorized,
    InternalServerError   
}

fn get_error_details(error_code: ErrorCode) -> (i32, String) {
    match error_code {
        ErrorCode::BadRequest => (400, "Bad Request".to_owned()),
        ErrorCode::Unauthorized => (401, "Unauthorized".to_owned()),
        ErrorCode::NotFound => (404, "Not Found".to_owned()),
        ErrorCode::InternalServerError => (500, "Internal Server Error".to_owned()),
        ErrorCode::NotImplemented => (501, "Not Implemented".to_owned()),
    }
}

fn get_path(request: &str) -> Result<String, ErrorCode> {
    let re = Regex::new(r"(GET|HEAD) (.*?) HTTP/1.1\r\n").unwrap();

    match re.captures(request){
        Some(match_value) => Ok(match_value[2].to_owned()),
        None => Err(ErrorCode:: NotImplemented)
    }
}

fn get_full_path_to_file(full_path: &str, root_dir: &str) -> Result<String, ErrorCode> {
    let path = Path::new(full_path);

    if path.exists() {
        if !path.is_dir() {
            return Ok(full_path.to_owned());
        }

        return Err(ErrorCode::BadRequest)
    }

    let path = Path::new(&root_dir); 
    let path = path.join(&full_path[1..]);
    if path.exists() {
        if !path.is_dir() {
            let path = path.to_str().unwrap().to_owned(); 
            return Ok(path);
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