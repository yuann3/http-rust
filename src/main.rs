use flate2::write::GzEncoder;
use flate2::Compression;
use std::{
    collections::HashMap,
    env, error,
    fs::{self, File},
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    path::Path,
};

fn main() -> Result<(), Box<dyn error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:4221")?;
    println!("Server listening on port 4221");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(move || {
                    if let Err(e) = handle_connection(stream) {
                        eprintln!("Error handling connection: {}", e);
                    }
                });
            }
            Err(e) => eprintln!("Error: {}", e),
        }
    }
    Ok(())
}

fn client_accept_gzip(headers: &HashMap<String, String>) -> bool {
    headers
        .get("Accept-Encoding")
        .map(|encodings| encodings.contains("gzip"))
        .unwrap_or(false)
}

fn handle_connection(mut stream: TcpStream) -> Result<(), Box<dyn error::Error>> {
    let mut buf_reader = BufReader::new(&stream);
    let mut request_line = String::new();
    buf_reader.read_line(&mut request_line)?;

    let (method, path, _) = parse_request_line(&request_line);
    let headers = parse_headers(&mut buf_reader);

    let mut body = Vec::new();
    if method == "POST" {
        if let Some(content_length) = headers.get("Content-Length") {
            let content_length: usize = content_length.parse()?;
            body = vec![0; content_length];
            buf_reader.read_exact(&mut body)?;
        }
    }

    let support_gzip = client_accept_gzip(&headers);

    match (method.as_str(), path.as_str()) {
        ("GET", path) if path.starts_with("/echo/") => {
            let echo_string = &path[6..];
            send_response_with_encoding(
                &mut stream,
                "200 OK",
                "text/plain",
                echo_string,
                support_gzip,
            )?;
        }
        ("GET", path) if path.starts_with("/files/") => {
            handle_get_file(&mut stream, &path)?;
        }
        ("POST", path) if path.starts_with("/files/") => {
            handle_post_file(&mut stream, &path, &body)?;
        }
        ("GET", "/user-agent") => {
            let user_agent = headers.get("User-Agent").map(String::as_str).unwrap_or("");
            send_response_with_encoding(
                &mut stream,
                "200 OK",
                "text/plain",
                user_agent,
                support_gzip,
            )?;
        }
        ("GET", "/") => {
            send_response_with_encoding(&mut stream, "200 OK", "text/plain", "", support_gzip)?;
        }
        _ => {
            send_response_with_encoding(
                &mut stream,
                "404 Not Found",
                "text/plain",
                "404 Not Found",
                support_gzip,
            )?;
        }
    }
    Ok(())
}

fn handle_get_file(stream: &mut TcpStream, path: &str) -> Result<(), Box<dyn error::Error>> {
    let filename = &path[7..];
    let directory = env::args().nth(2).unwrap_or_else(|| ".".to_string());
    let file_path = Path::new(&directory).join(filename);
    if file_path.is_file() {
        match fs::read(&file_path) {
            Ok(content) => {
                send_response(
                    stream,
                    "200 OK",
                    "application/octet-stream",
                    &String::from_utf8_lossy(&content),
                )?;
            }
            Err(e) => {
                eprintln!("Error reading file: {}", e);
                send_response(stream, "500 Internal Server Error", "text/plain", "")?;
            }
        }
    } else {
        send_response(stream, "404 Not Found", "text/plain", "")?;
    }
    Ok(())
}

fn handle_post_file(
    stream: &mut TcpStream,
    path: &str,
    body: &[u8],
) -> Result<(), Box<dyn error::Error>> {
    let filename = &path[7..];
    let directory = env::args().nth(2).unwrap_or_else(|| ".".to_string());
    let file_path = Path::new(&directory).join(filename);

    let mut file = File::create(file_path)?;
    file.write_all(body)?;

    send_response(stream, "201 Created", "text/plain", "")?;
    Ok(())
}

fn parse_request_line(request_line: &str) -> (String, String, String) {
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("").to_string();
    let path = parts.next().unwrap_or("").to_string();
    let version = parts.next().unwrap_or("").to_string();
    (method, path, version)
}

fn parse_headers(buf_reader: &mut BufReader<&TcpStream>) -> HashMap<String, String> {
    buf_reader
        .lines()
        .map(|line| line.unwrap())
        .take_while(|line| !line.is_empty())
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
        content,
    );

    stream.write_all(response.as_bytes())?;
    Ok(())
}

fn send_response_with_encoding(
    stream: &mut TcpStream,
    status: &str,
    content_type: &str,
    content: &str,
    use_gzip: bool,
) -> Result<(), Box<dyn error::Error>> {
    let mut response = format!("HTTP/1.1 {}\r\nContent-Type: {}\r\n", status, content_type);

    let body: Vec<u8>;
    if use_gzip {
        let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
        encoder.write_all(content.as_bytes())?;
        body = encoder.finish()?;
        response.push_str("Content-Encoding: gzip\r\n");
    } else {
        body = content.as_bytes().to_vec();
    }

    response.push_str(&format!("Content-Length: {}\r\n\r\n", body.len()));

    stream.write_all(response.as_bytes())?;
    stream.write_all(&body)?;
    Ok(())
}
