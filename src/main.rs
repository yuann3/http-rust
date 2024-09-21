#[allow(unused_imports)]
use std::io::Write;
use std::{
    io::{BufRead, BufReader},
    net::{TcpListener, TcpStream},
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("accepted new connection");
                handle_connection(stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let buf_reader = BufReader::new(&mut stream);
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let path = request_line.split_whitespace().nth(1).unwrap_or("/");

    let status = if path == "/" {
        "200 OK"
    } else {
        "404 Not Found"
    };

    let response = format!("HTTP/1.1 {}\r\n\r\n", status);
    stream.write_all(response.as_bytes()).unwrap();
}
