# Rust HTTP Server

A simple HTTP server implemented in Rust, supporting basic GET and POST requests, file operations, and gzip compression.

## Features

- HTTP/1.1 protocol support
- GET and POST request handling
- File upload and download capabilities
- User-Agent echo endpoint
- Gzip compression support
- Multithreaded request handling

## Getting Started

### Installation

1. Clone the repository:

2. Run the server:
   ```
   ./build.sh
   ```

## Usage

The server listens on `127.0.0.1:4221` by default. Here are some example requests you can make:

- Echo endpoint: `GET /echo/<string>`
- User-Agent: `GET /user-agent`
- File download: `GET /files/<filename>`
- File upload: `POST /files/<filename>`

The server supports gzip compression. Include the `Accept-Encoding: gzip` header in your requests to receive compressed responses when applicable.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
