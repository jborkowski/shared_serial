use std::net::{TcpListener, TcpStream};
use std::io::{Read};
use serialport::SerialPort;

fn handle_client(mut stream: TcpStream, serial: &mut dyn SerialPort) {
    let mut buf = [0; 1024];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break, //connection closed
            Ok(n) => {
                serial
                    .write_all(&buf[0..n])
                    .expect("Failed to write to serial port");
                let mut response = vec![0; 1024];
                let bytes_read = serial
                    .read(&mut response)
                    .expect("Failed to read from serial port");
                serial
                    .write_all(&response[0..bytes_read])
                    .expect("Failed to write to stream");
            }
            Err(_) => break,
        }
    }
}


fn main() {
    let mut serial = serialport::new("/dev/cu.usbmodem2101", 9600).open().expect("Failed to open serial port");

    let listener = TcpListener::bind("0.0.0.0:99919").expect("Failed to bind to address");
    println!("Server listening on port 99919");

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New client connected");
                handle_client(stream, &mut *serial);
            },
            Err(e) => {
                eprintln!("Connection failed: {}", e);
            }
        }
    }
}
