extern crate lazy_static;
extern crate libc;

use libc::{
    c_char, c_int, c_void, epoll_create1, epoll_ctl, epoll_event, epoll_wait, size_t, ssize_t,
    EPOLLERR, EPOLLHUP, EPOLLIN, EPOLLOUT,
};
use std::net::TcpStream;
use std::{
    ffi::CStr,
    io::{Read, Write},
};
use std::{
    os::{fd::RawFd, unix::io::AsRawFd},
    sync::Once,
};

static INIT: Once = Once::new();
static mut CLIENT_SOCKET: Option<TcpStream> = None;
static mut EPOLL_FD: RawFd = -1;

fn init() {
    unsafe {
        let epoll_fd = epoll_create1(0);
        if epoll_fd == -1 {
            panic!("Failed to create epoll file descriptor");
        }
        EPOLL_FD = epoll_fd;
    }
}

fn ensure_initialized() {
    INIT.call_once(init);
}

#[no_mangle]
pub extern "C" fn open(pathname: *const c_char, flags: c_int) -> c_int {
    let path = unsafe { CStr::from_ptr(pathname).to_str().unwrap_or("") };
    if path.contains("/dev/tty") {
        ensure_initialized();
        unsafe {
            if CLIENT_SOCKET.is_none() {
                match TcpStream::connect("100.98.67.49:12121") {
                    Ok(stream) => {
                        let fd = stream.as_raw_fd();

                        let mut event = epoll_event {
                            events: (EPOLLIN | EPOLLOUT | EPOLLERR | EPOLLHUP) as u32,
                            u64: fd as u64,
                        };

                        if epoll_ctl(EPOLL_FD, libc::EPOLL_CTL_ADD, fd, &mut event) == -1 {
                            eprintln!("Failed to add file descriptor to epoll");
                            std::process::abort();
                        }

                        CLIENT_SOCKET = Some(stream);
                    }
                    Err(e) => {
                        eprintln!("Connection failed: {}", e);
                        return -1;
                    }
                }
            }
            CLIENT_SOCKET.as_ref().unwrap().as_raw_fd()
        }
    } else {
        unsafe { libc::open(pathname, flags) }
    }
}

// #[no_mangle]
// pub extern "C" fn read(fd: c_int, buf: *mut c_void, count: size_t) -> ssize_t {
//     ensure_initialized();
//     unsafe {
//         if let Some(ref mut stream) = CLIENT_SOCKET {
//             if fd == stream.as_raw_fd() {
//                 let mut events = [epoll_event { events: 0, u64: 0 }; 10];
//                 let nfds = epoll_wait(EPOLL_FD, events.as_mut_ptr(), 10, -1);

//                 if nfds == -1 {
//                     return -1;
//                 }

//                 for i in 0..nfds {
//                     if events[i as usize].u64 == fd as u64 {
//                         let mut buffer = vec![0u8; count];

//                         let read = stream.read(&mut buffer);
//                         return match read {
//                             Ok(bytes_read) => {
//                                 std::ptr::copy_nonoverlapping(
//                                     buffer.as_ptr(),
//                                     buf as *mut u8,
//                                     bytes_read,
//                                 );
//                                 bytes_read as ssize_t
//                             }
//                             Err(_) => -1,
//                         };
//                     }
//                 }
//             }
//         }

//         libc::read(fd, buf, count)
//     }
// }

#[no_mangle]
pub extern "C" fn write(fd: c_int, buf: *const c_void, count: size_t) -> ssize_t {
    ensure_initialized();
    unsafe {
        if let Some(ref mut stream) = CLIENT_SOCKET {
            if fd == stream.as_raw_fd() {
                let buffer = std::slice::from_raw_parts(buf as *const u8, count);

                let mut events = [epoll_event { events: 0, u64: 0 }; 10];
                let nfds = epoll_wait(EPOLL_FD, events.as_mut_ptr(), 10, -1);

                if nfds == -1 {
                    return -1;
                }

                for i in 0..nfds {
                    if events[i as usize].u64 == fd as u64 {
                        let read = stream.write(buffer);
                        return match read {
                            Ok(written_bytes) => written_bytes as ssize_t,
                            Err(_) => -1,
                        };
                    }
                }
            }
        }
        libc::write(fd, buf, count)
    }
}
