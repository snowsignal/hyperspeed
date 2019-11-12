use std::net::TcpStream;
use bytes::{BufMut, BytesMut};
use std::io::{Read, ErrorKind, Write};
use crate::core::ClientView;

#[derive(Deserialize)]
pub struct InputMessage {
    pub keys: Vec<char>,
    pub clicks: Vec<(u32, u32)>
}

fn find_stream_end_chars(msg: String) -> usize {
    let mut sequential_exclamations = 0;
    for character in msg.chars().rev() {
        if character == '!' {
            sequential_exclamations += 1;
        } else {
            sequential_exclamations = 0;
        }
        if sequential_exclamations >= 3 {
            return msg.find(character).unwrap();
        }
    }
    return 0;
}

pub enum StreamReadResult {
    ValidMessage(String),
    InvalidMessage,
    StreamError(String),
    NotReady
}

pub enum StreamWriteResult {
    Ok,
    SocketClosed,
    OtherError(String)
}

use self::StreamReadResult::*;

pub fn read_message_from_stream(stream: &mut TcpStream, buffer: &mut BytesMut) -> StreamReadResult {
    stream.set_nonblocking(false);

    match stream.read(buffer.as_mut()) {
        Ok(_) => {
            let msg = String::from_utf8_lossy(buffer.as_ref());
            let msg_len = find_stream_end_chars(msg.to_string());
            if msg_len <= 0 {
                return InvalidMessage;
            }
            let msg: String = msg.chars().take(msg_len).collect();
            ValidMessage(msg)
        },
        Err(e) => StreamError(e.to_string())
    }
}

pub fn read_from_message_from_stream_nonblocking(stream: &mut TcpStream, buffer: &mut BytesMut) -> StreamReadResult {
    stream.set_nonblocking(true);

    match stream.read(buffer.as_mut()) {
        Ok(_) => {
            let msg = String::from_utf8_lossy(buffer.as_ref());
            let msg_len = find_stream_end_chars(msg.to_string());
            if msg_len <= 0 {
                return InvalidMessage;
            }
            let msg: String = msg.chars().take(msg_len).collect();
            ValidMessage(msg)
        },
        Err(e) => {
            match e.kind() {
                ErrorKind::WouldBlock => NotReady,
                _ => StreamError(e.to_string())
            }
        }
    }
}

pub fn send_view_to_stream(stream: &mut TcpStream, view: ClientView) -> StreamWriteResult {
    // Serialize view
    let ser_view = serde_json::to_string(&view);
    if ser_view.is_err() {
        return StreamWriteResult::OtherError(
            format!("Serialization of view failed: {}", ser_view.unwrap_err().to_string()))
    }
    let ser_view = ser_view.unwrap() + "\n";
    loop {
        match stream.write(ser_view.as_bytes()) {
            Ok(_) => break,
            Err(e) => match e.kind() {
                ErrorKind::WouldBlock => continue,
                ErrorKind::BrokenPipe
                | ErrorKind::ConnectionReset
                | ErrorKind::ConnectionAborted => return StreamWriteResult::SocketClosed,
                _ => return StreamWriteResult::OtherError(e.to_string())
            }
        }
    }
    stream.flush();
    StreamWriteResult::Ok
}
