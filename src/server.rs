
use std::sync::mpsc::SyncSender;
use std::net::{Ipv4Addr, TcpStream, TcpListener, SocketAddrV4};
use std::net::ToSocketAddrs;
use std::thread::sleep;
use std::time::Duration;
use std::error::Error;
use tokio::net::UdpSocket;
use tokio_util::udp::UdpFramed;
use futures::{StreamExt};
use tokio_util::codec::BytesCodec;
use tokio::runtime::{Builder};
use crate::metric::{Metric, ParseError};
use crate::server::Event::ParsedMetrics;

/// Acceptable event types.
///
pub enum Event {
    ParsedMetrics(Result<Vec<Metric>, ParseError>),
    TcpMessage(TcpStream),
    TimerFlush,
}

/// Setup the UDP socket that listens for metrics and
/// publishes them into the bucket storage.
pub fn udp_server(chan: SyncSender<Event>, port: u16) -> Result<(), Box<dyn Error>> {
    let mut rt =
        Builder::new()
            .threaded_scheduler()
            .core_threads(32)
            .thread_name("statsd-parsers")
            .enable_all()
            .build()
            .unwrap();

    rt.block_on(async {
        let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), port);
        let addr = addr.to_socket_addrs().unwrap().last().unwrap();

        let socket = UdpSocket::bind(&addr).await?;
        let mut framed = UdpFramed::new(socket, BytesCodec::new());

        while let Some(Ok((bytes, _addr))) = framed.next().await {
            let chan = chan.clone();
            tokio::spawn(async move {
                let str = String::from_utf8_lossy(bytes.as_ref());
                let parsed = Metric::parse(&str);
                let _ = chan.try_send(ParsedMetrics(parsed));
            });
        }

        Ok(())
    })
}

/// Setup the TCP socket that listens for management commands.
pub fn admin_server(chan: SyncSender<Event>, port: u16, host: &str) {
    let tcp = TcpListener::bind((host, port)).unwrap();
    for stream in tcp.incoming() {
        match stream {
            Ok(stream) => {
                chan.send(Event::TcpMessage(stream)).unwrap();
            }
            Err(e) => panic!("Unable to establish TCP socket: {}", e),
        }
    }
}


/// Publishes an event on the channel every interval
///
/// This message is used to push data from the buckets to the backends.
pub fn flush_timer_loop(chan: SyncSender<Event>, interval: u64) {
    let duration = Duration::new(interval, 0);
    loop {
        sleep(duration);
        chan.send(Event::TimerFlush).unwrap();
    }
}
