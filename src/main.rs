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

    // unwrap() is used, which will panic if there's an error
    let request_line = buf_reader.lines().next().unwrap().unwrap();

    // Read the index 1, if it dosent exist, default to "/"
    let url = request_line.split_whitespace().nth(1).unwrap_or("/");

    if url.starts_with("/echo/") {
        // Extract the string afther "/echo/"
        let echo_string = &url[6..];

        let content_len = echo_string.len();

        let response = format!(
            "HTTP/1.1 200 OK\r\n\
            Content-Type: text/plain\r\n\
            Content-Length: {}\r\n\
            \r\n\
            {}",
            content_len, echo_string
        );

        // Write the response back to the TCP stream
        stream.write_all(response.as_bytes()).unwrap();
    } else {
        let status = if url == "/" {
            "200 OK"
        } else {
            "404 Not Found"
        };

        let response = format!("HTTP/1.1 {}\r\n\r\n", status);

        // Write the response back to the TCP stream
        stream.write_all(response.as_bytes()).unwrap();
    }
}
