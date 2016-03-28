extern crate getopts;
use getopts::Options;
use std::env;

use std::io::prelude::*;
use std::net::{TcpStream, TcpListener};

use std::sync::mpsc::{channel, Sender, Receiver};

fn handle_socket<T: Read + Write>(mut stream: T, rx: Receiver<String>) {
    loop {
        let mut net_buffer = String::new();
        stream.read_to_string(&mut net_buffer);
        print!("{}", net_buffer);

        match rx.try_recv() {
            Ok(m) => {
                let result = stream.write(m.as_bytes());
                stream.flush();
                result
            },
            Err(_) => Ok(0)
        };
    }
}

fn handle_stdin(tx: Sender<String>) {
    let stdin = std::io::stdin();
    let mut handle = stdin.lock();

    loop {
        let mut std_buffer = String::new();
        handle.read_line(&mut std_buffer);
        tx.send(std_buffer);
    }
}

fn create_stream(host: String, port: String, server: bool) -> TcpStream {
    let dst = &*format!("{}:{}", host, port);

    if server {
        let listener = TcpListener::bind(dst).unwrap();
        match listener.accept() {
            Ok((stream, _)) => {
                stream.set_read_timeout(Some(std::time::Duration::new(0, 1000)));
                stream
            },
            Err(e) => panic!(e)
        }
    }
    else {
        let stream = TcpStream::connect(dst).unwrap();
        stream.set_read_timeout(Some(std::time::Duration::new(0, 1000)));
        stream
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut opts = Options::new();
    opts.optflag("l", "", "start as server and listen");
    opts.optopt("h", "", "specify the host", "0.0.0.0");
    opts.optopt("p", "", "specify the port", "1024");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => { m },
        Err(f) => { panic!(f.to_string()); }
    };

    let host = matches.opt_str("h").unwrap();
    let port = matches.opt_str("p").unwrap();
    let as_server = matches.opt_present("l");

    let mut stream = create_stream(host, port, as_server);

    let (tx, rx): (Sender<String>, Receiver<String>) = channel();
    std::thread::spawn(move || {
            handle_socket(stream, rx);
        });

    handle_stdin(tx);
}
