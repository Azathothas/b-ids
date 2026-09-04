//! HARNESS-11. The TCP layer below TLS, and the capability question it turns on.
//!
//! ⛔ Every test name starts with `tcp_layer`, because
//! `cargo test -p b-ids-harness tcp_layer -- --nocapture` is the entry's
//! acceptance command.
//!
//! ⚠ **These use a real loopback connection**, because the question is what an
//! ACCEPTED socket can be asked. A fixture would answer about the fixture.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};

use b_ids_harness::tcp::{capability, observe};

/// Accept one loopback connection and hand back both ends' view.
///
/// ⚠ Port 0, so the operating system picks one and two runs of this suite
/// cannot collide.
fn accepted(client_ttl: Option<u32>) -> (std::net::TcpStream, u16) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind a loopback listener");
    let addr = listener.local_addr().expect("the listener's address");

    let joined = std::thread::spawn(move || {
        let mut client = TcpStream::connect(addr).expect("connect to the listener");
        if let Some(ttl) = client_ttl {
            client.set_ttl(ttl).expect("set the client's hop limit");
        }
        client.write_all(b"hello").expect("write");
        // ⚠ Held open until the server has read, so the accepted socket is a
        // live connection rather than one already closed underneath the test.
        let mut buf = [0_u8; 2];
        let _ = client.read(&mut buf);
    });

    let (server, _) = listener.accept().expect("accept");
    let mut reader = server.try_clone().expect("clone the accepted socket");
    let mut buf = [0_u8; 5];
    reader
        .read_exact(&mut buf)
        .expect("read what the client sent");
    let mut writer = server.try_clone().expect("clone again to answer");
    let _ = writer.write_all(b"ok");
    joined.join().expect("the client thread");
    let port = server.peer_addr().expect("the peer address").port();
    (server, port)
}

#[test]
fn tcp_layer_the_source_port_is_recorded_and_it_is_the_peer_s() {
    let (server, expected) = accepted(None);
    let seen = observe(&server);
    assert_eq!(seen.source_port, Some(expected));
    // ⛔ AND IT IS NOT THE LISTENER'S. A port read off the wrong end of the
    // connection is a plausible number that describes this host.
    let local = server.local_addr().expect("the local address").port();
    assert_ne!(
        seen.source_port,
        Some(local),
        "the source port is the peer's, not the listener's"
    );
}

#[test]
fn tcp_layer_an_unavailable_field_is_absent_with_a_reason_rather_than_zero() {
    // ⛔ THE ENTRY'S RULE. A zero window size is a real value a real stack can
    // send, so using it for "not measured" would publish a measurement nobody
    // took.
    let (server, _) = accepted(None);
    let seen = observe(&server);

    assert_eq!(seen.maximum_segment_size, None);
    assert_eq!(seen.window_size, None);
    assert_eq!(seen.window_scale, None);
    assert_eq!(seen.time_to_live, None);
    assert_eq!(seen.option_order, None);

    assert!(
        seen.every_absence_explained(),
        "a None with no reason reads as not-applicable and means nobody looked: {:?}",
        seen.absent
    );
    for entry in &seen.absent {
        assert!(
            entry.why.len() > 30,
            "{} has a reason too short to be one: {}",
            entry.field,
            entry.why
        );
        assert!(
            entry.why.contains("HARNESS-11"),
            "{} does not name the entry that would change the answer",
            entry.field
        );
    }
}

#[test]
fn tcp_layer_the_socket_ttl_is_this_host_s_own_and_is_not_recorded_as_the_peer_s() {
    // ⛔ THE TRAP THIS MODULE EXISTS TO STOP. `TcpStream::ttl` is named `ttl`,
    // returns a plausible number, and is the LOCAL outgoing hop limit. Recording
    // it as the peer's would put this host's configuration into a profile as
    // though it were the browser's.
    //
    // ⚠ MEASURED HERE RATHER THAN ASSERTED: the client sets a distinctive value
    // and the server is asked what it sees.
    let distinctive = 37;
    let (server, _) = accepted(Some(distinctive));
    let local_ttl = server.ttl().expect("the accepted socket's ttl");

    assert_ne!(
        local_ttl, distinctive,
        "if this ever equals the client's value, ttl() has started reporting the peer's and \
         HARNESS-11's premise needs re-measuring"
    );

    let seen = observe(&server);
    assert_eq!(
        seen.time_to_live, None,
        "the peer's hop limit is not readable here, so it is absent"
    );
    let reason = seen
        .absent
        .iter()
        .find(|a| a.field == "time_to_live")
        .expect("time_to_live is listed as absent");
    assert!(
        reason.why.contains("this host's own"),
        "the reason says WHY rather than only that it is missing: {}",
        reason.why
    );

    println!(
        "tcp_layer: the client set ttl={distinctive} and the accepted socket reads {local_ttl}"
    );
}

#[test]
fn tcp_layer_the_capability_is_stated_and_it_is_one_of_six() {
    // ⭐ THE ENTRY ASKED FOR THE CAPABILITY TO BE ESTABLISHED AND RECORDED, so
    // it is something a session runs rather than a paragraph somebody trusts.
    let (server, _) = accepted(None);
    let seen = observe(&server);
    assert_eq!(seen.observed(), 1, "{seen:?}");
    assert_eq!(seen.absent.len(), 5);

    let stated = capability();
    assert!(stated.contains("1 of 6"), "{stated}");
    println!("{stated}");
}
