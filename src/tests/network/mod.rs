//! The tests here involve networking. Since these tests cannot actually create another computer with a client,
//! it will replicate a client's interactions with the server.

#[test]
fn can_connect_with_dummy_client() {

}

#[test]
fn can_send_message_and_get_response() {

}

#[test]
fn server_handles_invalid_address() {

}

#[test]
fn server_handles_invalid_port() {

}

#[test]
fn can_connect_with_multiple_clients() {

}

#[test]
fn can_disconnect_and_have_updated_client_list() {

}

use std::time::Duration;


// 5 microseconds is pretty fast, so adjust this if your computer is slow.
const LATENCY_CAP: Duration = 5 * DURATION::MICROSECOND;

#[test]
fn server_latency_below_threshold() {

}
