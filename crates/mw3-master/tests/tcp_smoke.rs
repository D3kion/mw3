//! Integration: BOOB register then COKE list over TCP.

use mw3_master::conn::handle_connection;
use mw3_master::MasterState;
use mw3_protocol::{ClientListRequest, ServerRegisterRequest};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};
use std::thread;

#[test]
fn register_then_list_roundtrip() {
    let listener = Arc::new(TcpListener::bind("127.0.0.1:0").unwrap());
    let addr = listener.local_addr().unwrap();
    let state = Arc::new(Mutex::new(MasterState::default()));
    let st = Arc::clone(&state);

    let l1 = Arc::clone(&listener);
    let server = thread::spawn(move || {
        let (stream, _) = l1.accept().unwrap();
        handle_connection(stream, Arc::clone(&st)).unwrap();
    });

    let mut c = TcpStream::connect(addr).unwrap();
    let reg = ServerRegisterRequest {
        version: 7,
        q_port: 27016,
    };
    c.write_all(&reg.encode()).unwrap();
    drop(c);
    server.join().unwrap();

    let st2 = Arc::clone(&state);
    let l2 = Arc::clone(&listener);
    let server2 = thread::spawn(move || {
        let (stream, _) = l2.accept().unwrap();
        handle_connection(stream, st2).unwrap();
    });

    let mut c2 = TcpStream::connect(addr).unwrap();
    let req = ClientListRequest { version: 7 };
    c2.write_all(&req.encode()).unwrap();
    let mut out = Vec::new();
    c2.read_to_end(&mut out).unwrap();
    drop(c2);
    server2.join().unwrap();

    let dec = mw3_protocol::ClientListResponse::decode(&out).unwrap();
    assert_eq!(dec.entries.len(), 1);
    assert_eq!(dec.entries[0].q_port, 27016);
    assert_eq!(dec.entries[0].ip_address, 0x0100_007F); // 127.0.0.1 host dword on LE
}

#[test]
fn empty_list_for_unknown_version() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let state = Arc::new(Mutex::new(MasterState::default()));
    let st = Arc::clone(&state);

    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        handle_connection(stream, st).unwrap();
    });

    let mut c = TcpStream::connect(addr).unwrap();
    c.write_all(&ClientListRequest { version: 99 }.encode())
        .unwrap();
    let mut out = Vec::new();
    c.read_to_end(&mut out).unwrap();
    server.join().unwrap();

    assert_eq!(out, vec![0, 0, 0, 0]);
}
