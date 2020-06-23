//! Shared code between client & server

use std::io::{self, BufRead};

pub const DEFAULT_SERVER_ADDR: &str = "127.0.0.1:4000";
const MESSAGE_BUFFER_SIZE: usize = 32;

/// Given a buffer (in this case, TcpStream), attempt to read
/// an unknown stream of bytes and decode to a String
pub fn extract_string_unbuffered(buf: &mut impl io::Read) -> io::Result<String> {
    let mut received: Vec<u8> = vec![];

    // Use a statically sized array buffer
    // Picking a size is tricky:
    // - A large array can waste stack space for bytes we may never need
    // - A small array results in more syscalls (for this unbuffered approach)
    let mut rx_bytes = [0u8; MESSAGE_BUFFER_SIZE];
    loop {
        // Read from the current data in the TcpStream
        // !NOTE: Each time this is called it can be a syscall
        let bytes_read = buf.read(&mut rx_bytes)?;

        // However many bytes we read, extend the `received` string bytes
        received.extend_from_slice(&rx_bytes[..bytes_read]);

        // And if we didn't fill the array,\
        // stop reading because there's no more data (we hope!)
        if bytes_read < MESSAGE_BUFFER_SIZE {
            break;
        }
    }

    String::from_utf8(received).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Couldn't parse received string as utf8",
        )
    })
}

/// Given a buffer (in this case, TcpStream), use `BufReader` and `BufRead` trait
/// to read the pending bytes in the stream
pub fn extract_string_buffered(mut buf: &mut impl io::Read) -> io::Result<String> {
    let mut reader = io::BufReader::new(&mut buf);

    // `fill_buf` will return a ref to the bytes pending (received by TCP)
    // This is still a lower-level call, so we have to follow it up with a call to consume
    let received: Vec<u8> = reader.fill_buf()?.to_vec();

    // Mark the bytes read as consumed so the buffer will not return them in a subsequent read
    reader.consume(received.len());

    String::from_utf8(received).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "Couldn't parse received string as utf8",
        )
    })
}
