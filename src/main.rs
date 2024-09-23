use std::{
    collections::HashMap,
    error,
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

fn main() -> Result<(), Box<dyn error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:4221")?;
    println!("Server listening on port 4221");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("Accepted new connection");
                handle_connection(stream)?;
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    Ok(())
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn error::Error>> {
    let buf_reader = BufReader::new(&mut stream);
    let request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    let request_line = &request[0];
    let (method, path, _) = parse_request_line(request_line);

    let headers = parse_headers(&request[1..]);

    match (method.as_str(), path.as_str()) {
        ("GET", path) if path.starts_with("/echo/") => {
            let echo_string = &path[6..];
            send_response(&mut stream, "200 OK", "text/plain", echo_string)?;
        }
        ("GET", "/user-agent") => {
            let user_agent = headers.get("User-Agent").map(String::as_str).unwrap_or("");
            send_response(&mut stream, "200 OK", "text/plain", user_agent)?;
        }
        ("GET", "/") => {
            send_response(&mut stream, "200 OK", "text/plain", "")?;
        }
        _ => {
            send_response(&mut stream, "404 Not Found", "text/plain", "404 Not Found")?;
        }
    }

    Ok(())
}

fn parse_request_line(request_line: &str) -> (String, String, String) {
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("").to_string();
    let version = parts.next().unwrap_or("").to_string();
    (method, path, version)
}

fn parse_headers(header_lines: &[String]) -> HashMap<String, String> {
    header_lines
        .iter()
        .filter_map(|line| {
            let mut parts = line.splitn(2, ": ");
            Some((parts.next()?.to_string(), parts.next()?.trim().to_string()))
        })
        .collect()
}

fn send_response(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    content: &str,
) -> Result<(), Box<dyn error::Error>> {
    let response = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\n\r\n{}",
        status,
        content_type,
        content.len(),
        content
    );
    stream.write_all(response.as_bytes())?;
    Ok(())
}
