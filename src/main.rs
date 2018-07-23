#[macro_use]
extern crate serde_derive;

extern crate serde;
extern crate time;
extern crate docopt;

use std::thread;
use std::sync::mpsc::channel;
use std::str;


// Local module imports.
mod metric;
mod cli;
mod server;
mod buckets;
mod backend;
mod management;
mod metric_processor;
mod backends {
    pub mod console;
    pub mod graphite;
    pub mod statsd;
}


fn main() {
    let args = cli::parse_args();

    let mut backends = backend::factory(
        &args.flag_console,
        &args.flag_graphite,
        &args.flag_graphite_host,
        &args.flag_graphite_port,
        &args.flag_statsd,
        &args.flag_statsd_host,
        &args.flag_statsd_port,
        &args.flag_statsd_packet_size
    );

    let (event_send, event_recv) = channel();
    let flush_send = event_send.clone();
    let udp_send = event_send.clone();
    let tcp_send = event_send.clone();

    let mut buckets = buckets::Buckets::new();

    println!("Starting statsd - {}",
             time::at(buckets.start_time()).rfc822().to_string());
    println!("Data server on 0.0.0.0:{}", args.flag_port);
    println!("Admin server on {}:{}",
             args.flag_admin_host,
             args.flag_admin_port);

    // Setup the UDP server which publishes events to the event channel
    let port = args.flag_port;
    thread::spawn(move || {
        server::udp_server(udp_send, port);
    });

    // Setup the TCP server for administration
    let tcp_port = args.flag_admin_port;
    let tcp_host = args.flag_admin_host;
    thread::spawn(move || {
        server::admin_server(tcp_send, tcp_port, &tcp_host);
    });

    // Run the timer that flushes metrics to the backends.
    let flush_interval = args.flag_flush_interval;
    thread::spawn(move || {
        server::flush_timer_loop(flush_send, flush_interval);
    });

    // Main event loop.
    loop {
        let result = match event_recv.recv() {
            Ok(res) => res,
            Err(e) => panic!(format!("Event channel has hung up: {:?}", e)),
        };

        match result {
            server::Event::TimerFlush => {
                buckets.process();
                for backend in backends.iter_mut() {
                    backend.flush_buckets(&buckets);
                }
                buckets.reset();
            }

            server::Event::UdpMessage(buf) => {
                // Create the metric and push it into the buckets.
                str::from_utf8(&buf)
                    .map(|val| {
                        metric::Metric::parse(&val)
                            .and_then(|metrics| {
                                for metric in metrics.iter() {
                                    buckets.add(&metric);
                                }
                                Ok(metrics.len())
                            })
                            .or_else(|err| {
                                buckets.add_bad_message();
                                Err(err)
                            })
                            .ok();
                    })
                    .ok();
            }

            server::Event::TcpMessage(stream) => {
                management::exec(stream, &mut buckets);
            }
        }
    }
}
