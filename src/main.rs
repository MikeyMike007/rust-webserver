#![allow(dead_code)]
#![allow(unused_variables)]

use multi_threaded_web_server::ThreadPool;

use std::{
    fs,
    io::{BufRead, BufReader, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    thread,
    time::Duration,
};

fn main() {
    let addr = [
        SocketAddr::from(([127, 0, 0, 1], 7887)),
        SocketAddr::from(([127, 0, 0, 1], 7888)),
    ];

    let listener = TcpListener::bind(&addr[..]).unwrap();
    let thread_pool = ThreadPool::new(4);

    for stream in listener.incoming().take(10) {
        match stream {
            Ok(stream) => {
                thread_pool.execute(|| handle_stream(stream));
            }
            Err(err) => {
                println!("Error accepting incoming connetion: {}", err);
            }
        }
    }
}

fn handle_stream(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();

    let get = b"GET / HTTP/1.1\r\n";
    let sleep = b"GET /sleep HTTP/1.1\r\n";

    let (status_line, filename) = if buffer.starts_with(get) {
        ("HTTP/1.1 200 OK", "index.html")
    } else if buffer.starts_with(sleep) {
        thread::sleep(Duration::from_secs(5));
        ("HTTP/1.1 200 OK", "index.html")
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let contents = fs::read_to_string(&filename).unwrap();

    let response = format!(
        "{}\r\nContent-Length: {}\r\n\r\n{}",
        status_line,
        contents.len(),
        contents
    );

    stream.write_all(response.as_bytes()).unwrap();
    stream.flush().unwrap();
}
