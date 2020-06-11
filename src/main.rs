use rosc::{OscMessage, OscPacket, OscBundle};
use std::net::{SocketAddr, UdpSocket};
use std::time::{Instant, Duration, SystemTime, UNIX_EPOCH};
use std::thread;

fn main() {
    thread::spawn(move || {
        let args: Vec<String> = std::env::args().collect();
        let target: SocketAddr = format!("{}:8888", args[1]).parse().unwrap();

        let sock = UdpSocket::bind("0.0.0.0:8889").unwrap();
        sock.set_nonblocking(true).unwrap();

        let mut last = Instant::now();

        loop {
            if last.elapsed() > Duration::from_millis(500) {
                let msg = rosc::encoder::encode(&OscPacket::Bundle(OscBundle {
                    timetag: encode_timestamp(),
                    content: vec![OscPacket::Message(OscMessage {
                        addr: "/time".to_string(),
                        args: vec![],
                    })],
                })).unwrap();

                sock.send_to(&msg, target).unwrap();

                last = Instant::now();
            }
        }
    });

    thread::spawn(move || {
        let sock = UdpSocket::bind("0.0.0.0:8888").unwrap();
        sock.set_nonblocking(true).unwrap();

        let mut buf = [0; rosc::decoder::MTU];

        loop {
            match sock.recv_from(&mut buf) {
                Ok((size, addr)) => {
                    let packet = rosc::decoder::decode(&buf[..size]).unwrap();
                    let now = SystemTime::now();
                    match packet {
                        OscPacket::Bundle(bundle) => {
                            let time = decode_timestamp(bundle.timetag);
                            let duration = now.duration_since(time).unwrap();
                            println!("{}: {:?}", addr, duration);
                        },
                        _ => {}
                    }
                }
                _ => {}
            }
        }

    });

    thread::park();
}

fn encode_timestamp() -> (u32, u32) {
    let epoch = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap();

    let secs = epoch.as_secs();
    let nanos = epoch.as_nanos() - secs as u128 * 1_000_000_000;

    (secs as u32, nanos as u32)
}

fn decode_timestamp(timetag: (u32, u32)) -> SystemTime {
    let (secs, nanos) = timetag;
    UNIX_EPOCH + Duration::from_nanos(secs as u64 * 1_000_000_000 + nanos as u64)
}
