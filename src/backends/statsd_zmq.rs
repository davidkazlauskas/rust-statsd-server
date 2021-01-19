use super::super::backend::Backend;
use super::super::buckets::Buckets;
use super::super::statsd_batch_capnp;
use std::sync::mpsc::SyncSender;
use crate::server::Event;
use crate::metric::{Metric, MetricKind};
use lz4::{EncoderBuilder, Decoder};
use crate::server::Event::ZmqBatch;
use std::io::Write;

struct StatsdZmqConnection {
    socket: zmq::Socket,
}

pub struct StatsdZmq {
    connections: Vec<StatsdZmqConnection>,
    uncompressed_data_processed: usize,
    compressed_data_sent: usize,
}

pub fn buckets_to_packed_message(buckets: &Buckets) -> (Vec<u8>, usize) {
    let mut stats: Vec<(&str, &MetricKind, f64)> = vec![];

    let gauge = MetricKind::Gauge;
    let counter = MetricKind::Counter(1.0);
    let timer = MetricKind::Timer;
    {
        stats.push(("statsd.bad_messages", &counter, buckets.bad_messages() as f64));
        stats.push(("statsd.total_messages", &counter, buckets.total_messages() as f64));

        for (key, value) in buckets.counters().iter() {
            stats.push((key, &counter, *value));
        }

        for (key, value) in buckets.gauges().iter() {
            stats.push((key, &gauge, *value));
        }

        for (key, values) in buckets.timers().iter() {
            for value in values {
                stats.push((key, &timer, *value));
            }
        }
    }

    // sort for amazing compression
    stats.sort_by(|&(a, _, _), &(b, _, _)| { a.cmp(b) });
    let batch_size = stats.len() as u32;

    let mut message = capnp::message::Builder::new_default();
    let mut batch = message.init_root::<statsd_batch_capnp::statsd_batch::Builder>();

    {
        let mut metric_kinds = batch.reborrow().init_metric_kinds(batch_size);
        for (pos, &(_, kind, _)) in stats.iter().enumerate() {
            let pos = pos as u32;
            match kind {
                MetricKind::Counter(_) => {
                    metric_kinds.set(
                        pos,
                        statsd_batch_capnp::statsd_batch::MetricKind::Counter
                    );
                }
                MetricKind::Gauge => {
                    metric_kinds.set(
                        pos,
                        statsd_batch_capnp::statsd_batch::MetricKind::Gauge
                    );
                }
                MetricKind::Timer => {
                    metric_kinds.set(
                        pos,
                        statsd_batch_capnp::statsd_batch::MetricKind::Timer
                    );
                }
            }
        }
    }

    {
        let metric_values = batch.reborrow().init_metric_values(batch_size);
        for (pos, &(_, _, value)) in stats.iter().enumerate() {
            let pos = pos as u32;
            metric_values.reborrow().set(pos, value);
        }
    }

    {
        let mut metric_labels = batch.reborrow().init_metric_labels(batch_size);
        for (pos, &(label, _, _)) in stats.iter().enumerate() {
            let pos = pos as u32;
            metric_labels.reborrow().set(pos, label);
        }
    }

    let mut bytes: Vec<u8> = Vec::with_capacity(2048);
    capnp::serialize::write_message(&mut bytes, &message).unwrap();
    let uncompressed_size = bytes.len();

    let mut compressed: Vec<u8> = Vec::with_capacity(2048);
    let mut encoder =
        EncoderBuilder::new().level(9).build(&mut compressed).unwrap();
    encoder.write_all(bytes.as_slice()).unwrap();
    let _ = encoder.finish();

    (compressed, uncompressed_size)
}

pub fn decompress_packed_message(message: &Vec<u8>) -> Option<Vec<u8>> {
    match Decoder::new(message.as_slice()) {
        Ok(mut decoder) => {
            let mut buffer = Vec::with_capacity(2048);
            let res = std::io::copy(&mut decoder, &mut buffer)
                .unwrap_or(0);
            if res > 0 {
                Some(buffer)
            } else { None }
        }
        _ => None
    }
}

pub fn capn_proto_metric_kind_to_domain_metric_kind(
    kind: statsd_batch_capnp::statsd_batch::MetricKind
) -> MetricKind {
    match kind {
        statsd_batch_capnp::statsd_batch::MetricKind::Gauge => MetricKind::Gauge,
        statsd_batch_capnp::statsd_batch::MetricKind::Timer => MetricKind::Timer,
        statsd_batch_capnp::statsd_batch::MetricKind::Counter => MetricKind::Counter(1.0),
    }
}

pub struct UnpackedZmqBatch {
    decompressed_bytes: Vec<u8>
}

impl UnpackedZmqBatch {
    // Validate everything that might be wrong so that iteration code would be simpler
    pub fn new(compressed_message: &Vec<u8>) -> Option<UnpackedZmqBatch> {
        match decompress_packed_message(compressed_message) {
            Some(decompressed) => {
                let message = capnp::serialize::read_message(
                    decompressed.as_slice(),
                    ::capnp::message::ReaderOptions::new()
                );
                match message {
                    Ok(message_reader) => {
                        let statsd_batch =
                            message_reader.get_root::<statsd_batch_capnp::statsd_batch::Reader>();
                        match statsd_batch {
                            Ok(reader) => {
                                match (reader.get_metric_labels(),
                                       reader.get_metric_values(),
                                       reader.get_metric_kinds())
                                {
                                    (Ok(labels), Ok(values), Ok(kinds)) => {
                                        if labels.len() > 0 &&
                                            labels.len() == values.len()
                                            && labels.len() == kinds.len()
                                        {
                                            let len = labels.len();
                                            let mut all_values_intact = true;
                                            for i in 0..len {
                                                match (labels.get(i), values.get(i), kinds.get(i)) {
                                                    (Ok(_), _, Ok(_)) => {}
                                                    _ => { all_values_intact = false }
                                                }
                                            }
                                            if all_values_intact {
                                                Some(UnpackedZmqBatch {
                                                    decompressed_bytes: decompressed
                                                })
                                            } else { None }
                                        } else { None }
                                    }
                                    _ => { None }
                                }
                            }
                            Err(_e) => None
                        }
                    }
                    Err(_e) => None
                }
            }
            None => None
        }
    }

    #[cfg(test)]
    pub fn access_readers(&self, f: &mut impl FnMut(
        &capnp::text_list::Reader,
        &capnp::primitive_list::Reader<f64>,
        &capnp::enum_list::Reader<statsd_batch_capnp::statsd_batch::MetricKind>
    )) {
        let message = capnp::serialize::read_message(
        self.decompressed_bytes.as_slice(),
        ::capnp::message::ReaderOptions::new()
        ).unwrap();
        let reader = message.get_root::<statsd_batch_capnp::statsd_batch::Reader>().unwrap();
        let labels = reader.get_metric_labels().unwrap();
        let kinds = reader.get_metric_kinds().unwrap();
        let values = reader.get_metric_values().unwrap();
        f(&labels, &values, &kinds);
    }

    // fastest way to iterate currently
    pub fn iterate_optimal(&self, f: &mut impl FnMut(Metric)) {
        let message = capnp::serialize::read_message(
            self.decompressed_bytes.as_slice(),
            ::capnp::message::ReaderOptions::new()
        ).unwrap();
        let reader = message.get_root::<statsd_batch_capnp::statsd_batch::Reader>().unwrap();
        let labels = reader.get_metric_labels().unwrap();
        let kinds = reader.get_metric_kinds().unwrap();
        let values = reader.get_metric_values().unwrap();
        let len = labels.len();
        for i in 0..len {
            let res = Metric::new(
                labels.get(i).unwrap(), values.get(i),
                capn_proto_metric_kind_to_domain_metric_kind(kinds.get(i).unwrap())
            );
            f(res)
        }
    }
}

impl StatsdZmq {
    pub fn new(statsd_zmq_hosts: Vec<String>) -> StatsdZmq {
        let context = zmq::Context::new();
        let connections = statsd_zmq_hosts.iter().map(|host| {
            println!("Opening zmq socket to another statsd server {}", host);
            let socket = context.socket(zmq::SocketType::PUB).unwrap();
            socket.set_sndhwm(4).unwrap();
            socket.connect(&host).unwrap();
            StatsdZmqConnection {
                socket: socket,
            }
        }).collect();
        StatsdZmq {
            connections: connections,
            compressed_data_sent: 0,
            uncompressed_data_processed: 0,
        }
    }
}

impl Backend for StatsdZmq {
    fn flush_buckets(&mut self, buckets: &Buckets) {
        let (stats, uncompressed_size) = buckets_to_packed_message(buckets);
        let compressed_size = stats.len();
        println!("Sending compressed stats batch from {} to {} bytes ({:.2} ratio)",
                 uncompressed_size,
                 compressed_size,
                 (compressed_size as f32 / (uncompressed_size as f32)));
        self.compressed_data_sent += compressed_size;
        self.uncompressed_data_processed += uncompressed_size;
        println!("Total data compressed from {} to {} bytes ({:.2} ratio)",
                self.uncompressed_data_processed,
                self.compressed_data_sent,
                (self.compressed_data_sent as f32 / self.uncompressed_data_processed as f32));
        for i in 0..self.connections.len() {
            let i = &self.connections[i];
            i.socket.send(stats.as_slice(), zmq::DONTWAIT).unwrap_or_default();
        }
    }
}

pub fn statsd_zmq_server(
    port: u16,
    with_message: Box<dyn Fn(UnpackedZmqBatch)>
) {
    let context = zmq::Context::new();
    let socket = context.socket(zmq::SocketType::SUB).unwrap();
    socket.set_rcvhwm(1024).unwrap();
    socket.bind(&format!("tcp://*:{}", port)).unwrap();
    socket.set_subscribe("".as_bytes()).unwrap();

    loop {
        let data = socket.recv_bytes(0);
        match data {
            Ok(data) => {
                let unpacked = UnpackedZmqBatch::new(&data);
                match unpacked {
                    Some(batch) => { with_message(batch) }
                    None => {}
                }
            }
            Err(err) => eprintln!("ZeroMQ message receive error: {:?}", err)
        }
    }
}

pub fn statsd_zmq_event_emitter(
    port: u16,
    sender: SyncSender<Event>
) {
    crate::backends::statsd_zmq::statsd_zmq_server(
        port,
        Box::new(move |zmq_batch| {
            sender.send(ZmqBatch(zmq_batch)).unwrap_or_default();
        }));
}

fn benchmarking_rounds() -> i32 { 100000 }

fn benchmark_compression() {
    let mut buckets = Buckets::new(1.0, false);
    for i in 0..100 {
        buckets.add(&Metric::new(format!("hello_some_metric_{}", i),
                                 123.0, MetricKind::Gauge));
        buckets.add(&Metric::new(format!("world_some_metric_{}", i),
                                 321.0, MetricKind::Counter(1.0)));
        buckets.add(&Metric::new(format!("timer_some_metric_{}", i),
                                 100.0, MetricKind::Timer));
    }
    buckets.process();

    let rounds = benchmarking_rounds();
    let before = std::time::Instant::now();
    let mut uncompressed = 0;
    let mut compressed = 0;
    for _ in 0..rounds {
        let (msg, before_comp_size) = buckets_to_packed_message(&buckets);
        compressed += msg.len();
        uncompressed += before_comp_size;
    }
    let after = std::time::Instant::now();
    let res = after - before;
    println!("{} serializations done in {} ms", rounds, res.as_millis());
    println!("Total compressed bytes from {} to {} ({:.2} ratio)", uncompressed, compressed,
             compressed as f32 / uncompressed as f32);
}

fn benchmark_decompression() {
    let mut buckets = Buckets::new(1.0, false);
    for i in 0..100 {
        buckets.add(&Metric::new(format!("hello_some_metric_{}", i),
                                 123.0, MetricKind::Gauge));
        buckets.add(&Metric::new(format!("world_some_metric_{}", i),
                                 321.0, MetricKind::Counter(1.0)));
        buckets.add(&Metric::new(format!("timer_some_metric_{}", i),
                                 100.0, MetricKind::Timer));
    }
    buckets.process();

    let (packed, _uncompressed_size) = buckets_to_packed_message(&buckets);
    let rounds = benchmarking_rounds();
    let before = std::time::Instant::now();
    for _ in 0..rounds {
        let _ = UnpackedZmqBatch::new(&packed);
    }
    let after = std::time::Instant::now();
    let res = after - before;
    println!("{} deserializations done in {} ms", rounds, res.as_millis());
}

fn benchmark_processing() {
    let mut buckets = Buckets::new(1.0, false);
    for i in 0..100 {
        buckets.add(&Metric::new(format!("hello_some_metric_{}", i),
                                 123.0, MetricKind::Gauge));
        buckets.add(&Metric::new(format!("world_some_metric_{}", i),
                                 321.0, MetricKind::Counter(1.0)));
        buckets.add(&Metric::new(format!("timer_some_metric_{}", i),
                                 100.0, MetricKind::Timer));
    }
    buckets.process();

    let (packed, _uncompressed_size) = buckets_to_packed_message(&buckets);
    let unpacked = UnpackedZmqBatch::new(&packed).unwrap();
    let rounds = benchmarking_rounds();
    let before = std::time::Instant::now();
    let mut all_values_sum = 0.0;
    for _ in 0..rounds {
        unpacked.iterate_optimal(&mut |metric: Metric| {
            all_values_sum += metric.value;
        });
    }
    let after = std::time::Instant::now();
    let res = after - before;
    println!("{} deserialization with processing done in {} ms (checksum {})",
             rounds, res.as_millis(), all_values_sum);
}

fn benchmark_processing_wbuckets() {
    let mut buckets = Buckets::new(1.0, false);
    for i in 0..100 {
        buckets.add(&Metric::new(format!("hello_some_metric_{}", i),
                                 123.0, MetricKind::Gauge));
        buckets.add(&Metric::new(format!("world_some_metric_{}", i),
                                 321.0, MetricKind::Counter(1.0)));
        buckets.add(&Metric::new(format!("timer_some_metric_{}", i),
                                 100.0, MetricKind::Timer));
    }
    buckets.process();

    let (packed, _uncompressed_size) = buckets_to_packed_message(&buckets);
    let unpacked = UnpackedZmqBatch::new(&packed).unwrap();
    let rounds = benchmarking_rounds();
    let before = std::time::Instant::now();
    let mut message_count = 0;
    for _ in 0..rounds {
        unpacked.iterate_optimal(&mut |metric: Metric| {
            buckets.add(&metric);
            message_count += 1;
        });
    }
    let after = std::time::Instant::now();
    let res = after - before;
    println!("{} deserialization with processing with buckets done in {} ms, messages done: {}",
             rounds, res.as_millis(), message_count);
}


fn benchmark_processing_wbuckets_wdecomp() {
    let mut buckets = Buckets::new(1.0, false);
    for i in 0..100 {
        buckets.add(&Metric::new(format!("hello_some_metric_{}", i),
                                 123.0, MetricKind::Gauge));
        buckets.add(&Metric::new(format!("world_some_metric_{}", i),
                                 321.0, MetricKind::Counter(1.0)));
        buckets.add(&Metric::new(format!("timer_some_metric_{}", i),
                                 100.0, MetricKind::Timer));
    }
    buckets.process();

    let (packed, _uncompressed_size) = buckets_to_packed_message(&buckets);
    let rounds = benchmarking_rounds();
    let before = std::time::Instant::now();
    let mut message_count = 0;
    for _ in 0..rounds {
        let unpacked = UnpackedZmqBatch::new(&packed).unwrap();
        unpacked.iterate_optimal(&mut |metric: Metric| {
            buckets.add(&metric);
            message_count += 1;
        });
    }
    let after = std::time::Instant::now();
    let res = after - before;
    println!("{} deserialization with decompression and processing with buckets done in {} ms, messages done: {}",
             rounds, res.as_millis(), message_count);
}

pub fn benchmarks() {
    benchmark_processing();
    benchmark_processing_wbuckets();
    benchmark_processing_wbuckets_wdecomp();
    benchmark_compression();
    benchmark_decompression();
}

#[cfg(test)]
mod tests {
    use crate::buckets::Buckets;
    use crate::metric::{Metric, MetricKind};
    use crate::statsd_batch_capnp::statsd_batch;
    use crate::backends::statsd_zmq;

    #[test]
    fn encode_and_decode_works() {
        let mut buckets = Buckets::new(1.0, false);
        buckets.add_bad_message();
        buckets.add(&Metric::new("hello", 123.0, MetricKind::Gauge));
        buckets.add(&Metric::new("world", 321.0, MetricKind::Counter(1.0)));
        buckets.add(&Metric::new("timer", 100.0, MetricKind::Timer));
        buckets.process();

        let (packed, _uncompressed_size) = statsd_zmq::buckets_to_packed_message(&buckets);
        let unpacked = statsd_zmq::UnpackedZmqBatch::new(&packed);
        assert!(unpacked.is_some());
        let unpacked = unpacked.unwrap();
        unpacked.access_readers(&mut |labels, values, kinds| {
            let extra_count = 3; // total messages, bad messages and processing time
            let other_metrics_count = 3;
            assert_eq!(labels.len(), extra_count + other_metrics_count);
            assert_eq!(labels.len(), values.len());
            assert_eq!(labels.len(), kinds.len());

            // labels are sorted for compression
            assert_eq!(labels.get(0).unwrap(), "hello");
            assert_eq!(labels.get(1).unwrap(), "statsd.bad_messages");
            assert_eq!(labels.get(2).unwrap(), "statsd.processing_time");
            assert_eq!(labels.get(3).unwrap(), "statsd.total_messages");
            assert_eq!(labels.get(4).unwrap(), "timer");
            assert_eq!(labels.get(5).unwrap(), "world");

            assert_eq!(values.get(0), 123.0);
            assert_eq!(values.get(1), 1.0);
            assert_eq!(values.get(3), 5.0);
            assert_eq!(values.get(4), 100.0);
            assert_eq!(values.get(5), 321.0);

            assert!(kinds.get(0).unwrap() == statsd_batch::MetricKind::Gauge);
            assert!(kinds.get(1).unwrap() == statsd_batch::MetricKind::Counter);
            assert!(kinds.get(3).unwrap() == statsd_batch::MetricKind::Counter);
            assert!(kinds.get(4).unwrap() == statsd_batch::MetricKind::Timer);
            assert!(kinds.get(5).unwrap() == statsd_batch::MetricKind::Counter);
        });
    }
}
