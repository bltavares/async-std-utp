#![feature(test)]
extern crate test;

use async_std::task;
use async_std_utp::UtpSocket;
use std::sync::Arc;
use test::Bencher;

macro_rules! iotry {
    ($e:expr) => {
        match $e.await {
            Ok(e) => e,
            Err(e) => panic!("{}", e),
        }
    };
}

fn next_test_port() -> u16 {
    use std::sync::atomic::{AtomicUsize, Ordering};
    static NEXT_OFFSET: AtomicUsize = AtomicUsize::new(0);
    const BASE_PORT: u16 = 9600;
    BASE_PORT + NEXT_OFFSET.fetch_add(1, Ordering::Relaxed) as u16
}

fn next_test_ip4<'a>() -> (&'a str, u16) {
    ("127.0.0.1", next_test_port())
}

#[bench]
fn bench_connection_setup_and_teardown(b: &mut Bencher) {
    let server_addr = next_test_ip4();

    b.iter(|| {
        let mut server = task::block_on(async { iotry!(UtpSocket::bind(server_addr)) });
        let mut buf = vec![0; 1500];

        task::spawn(async move {
            let mut client = iotry!(UtpSocket::connect(server_addr));
            iotry!(client.close());
        });

        task::block_on(task::spawn(async move {
            loop {
                match server.recv_from(&mut buf).await {
                    Ok((0, _src)) => break,
                    Ok(_) => (),
                    Err(e) => panic!("{}", e),
                }
            }
            iotry!(server.close());
        }));
    });
}

#[bench]
fn bench_transfer_one_packet(b: &mut Bencher) {
    let len = 1024;
    let server_addr = next_test_ip4();
    let data = (0..len).map(|x| x as u8).collect::<Vec<u8>>();
    let data_arc = Arc::new(data);

    b.iter(|| {
        let mut server = task::block_on(async { iotry!(UtpSocket::bind(server_addr)) });
        let data = data_arc.clone();
        let mut buf = vec![0; 1500];

        task::spawn(async move {
            let mut client = iotry!(UtpSocket::connect(server_addr));
            iotry!(client.send_to(&data[..]));
            iotry!(client.close());
        });

        task::block_on(task::spawn(async move {
            loop {
                match server.recv_from(&mut buf).await {
                    Ok((0, _src)) => break,
                    Ok(_) => (),
                    Err(e) => panic!("{}", e),
                }
            }
            iotry!(server.close());
        }));
    });
    b.bytes = len as u64;
}

#[bench]
fn bench_transfer_one_megabyte(b: &mut Bencher) {
    let len = 1024 * 1024;
    let server_addr = next_test_ip4();
    let data = (0..len).map(|x| x as u8).collect::<Vec<u8>>();
    let data_arc = Arc::new(data);

    b.iter(|| {
        let mut server = task::block_on(async { iotry!(UtpSocket::bind(server_addr)) });
        let data = data_arc.clone();
        let mut buf = vec![0; 1500];

        task::spawn(async move {
            let mut client = iotry!(UtpSocket::connect(server_addr));
            iotry!(client.send_to(&data[..]));
            iotry!(client.close());
        });

        task::block_on(task::spawn(async move {
            loop {
                match server.recv_from(&mut buf).await {
                    Ok((0, _src)) => break,
                    Ok(_) => (),
                    Err(e) => panic!("{}", e),
                }
            }
            iotry!(server.close());
        }));
    });
    b.bytes = len as u64;
}
