use super::buckets::Buckets;
use super::metric::{Metric, MetricKind};
use std::collections::HashMap;
use time;

/// Creates derived values from metric data.
///
/// Creates:
///
/// - timer percentile data.
/// - internal processing metrics
pub fn process(buckets: &mut Buckets) {
    let start_time = time::get_time();

    let mut timer_data = HashMap::new();

    // Add the various derived values for timers.
    for (key, values) in buckets.timers().iter() {
        if values.len() > 0 {
            let mut v = values.clone();
            v.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let len = v.len() as f64;
            let sum = v.iter().fold(0.0, |sum, x| sum + x);
            let mean = sum / len;

            // Get population standard deviation
            let sum_diff = v.iter().fold(0.0, |sum, x| sum + (x - mean).powi(2));
            let stddev = (sum_diff / len).sqrt();

            let median = percentile(&v, 0.5);
            let upper_90 = percentile(&v, 0.90);
            let upper_95 = percentile(&v, 0.95);
            let upper_99 = percentile(&v, 0.99);
            let count_per_second = len as f64 / buckets.flush_interval();

            timer_data.insert(format!("{}.min", key), v[0]);
            timer_data.insert(format!("{}.max", key), v[v.len() - 1]);
            timer_data.insert(format!("{}.count", key), len);
            timer_data.insert(format!("{}.count_ps", key), count_per_second as f64);
            timer_data.insert(format!("{}.mean", key), mean);
            timer_data.insert(format!("{}.median", key), median);
            timer_data.insert(format!("{}.stddev", key), stddev);
            timer_data.insert(format!("{}.upper_90", key), upper_90);
            timer_data.insert(format!("{}.upper_95", key), upper_95);
            timer_data.insert(format!("{}.upper_99", key), upper_99);
        }
    }
    buckets.set_timer_data(timer_data);

    let duration = time::get_time() - start_time;
    let process_duration = Metric::new("statsd.processing_time",
                                       duration.num_milliseconds() as f64,
                                       MetricKind::Counter(1.0));
    buckets.add(&process_duration);
}


/// Extract the value at the given percentile.
/// If vector has an even length, two values will be
/// averaged together.
fn percentile(values: &Vec<f64>, tile: f64) -> f64 {
    let len = values.len() as f64;
    let index = (len * tile) as usize;
    if (values.len() % 2) == 0 {
        return (values[index - 1] + values[index]) / 2.0;
    } else {
        return values[index];
    }
}



#[cfg(test)]
mod test {
    use super::*;
    use super::super::buckets::Buckets;
    use super::super::metric::{Metric, MetricKind};

    fn make_buckets() -> Buckets {
        let mut buckets = Buckets::new(0., true);

        let metrics = [Metric::new("some.timer", 13.1, MetricKind::Timer),
                       Metric::new("some.timer", 33.7, MetricKind::Timer),
                       Metric::new("some.timer", 34.7, MetricKind::Timer),
                       Metric::new("some.timer", 3.4, MetricKind::Timer),
                       Metric::new("some.timer", 1.4, MetricKind::Timer),
                       Metric::new("some.timer", 0.7, MetricKind::Timer),
                       Metric::new("some.timer", 5.6, MetricKind::Timer),
                       Metric::new("some.timer", 1.4, MetricKind::Timer),
                       Metric::new("some.timer", 24.5, MetricKind::Timer),
                       Metric::new("some.timer", 0.7, MetricKind::Timer),
                       Metric::new("some.timer", 5.6, MetricKind::Timer),
                       Metric::new("some.timer", 9.4, MetricKind::Timer),
                       Metric::new("some.timer", 0.7, MetricKind::Timer),
                       Metric::new("some.timer", 3.6, MetricKind::Timer),
                       Metric::new("some.timer", 6.7, MetricKind::Timer),
                       Metric::new("some.timer", 6.3, MetricKind::Timer),
                       Metric::new("some.timer", 4.3, MetricKind::Timer),
                       Metric::new("some.timer", 6.1, MetricKind::Timer),
                       Metric::new("some.timer", 0.7, MetricKind::Timer),
                       Metric::new("some.timer", 3.7, MetricKind::Timer),
                       Metric::new("some.timer", 3.4, MetricKind::Timer),
                       Metric::new("some.timer", 1.4, MetricKind::Timer),
                       Metric::new("some.timer", 0.7, MetricKind::Timer),
                       Metric::new("some.timer", 5.6, MetricKind::Timer),
                       Metric::new("some.timer", 1.4, MetricKind::Timer),
                       Metric::new("some.timer", 24.5, MetricKind::Timer),
                       Metric::new("some.timer", 0.7, MetricKind::Timer),
                       Metric::new("some.timer", 5.6, MetricKind::Timer),
                       Metric::new("some.timer", 9.4, MetricKind::Timer),
                       Metric::new("some.timer", 0.7, MetricKind::Timer),
                       Metric::new("some.timer", 3.6, MetricKind::Timer),
                       Metric::new("some.timer", 6.7, MetricKind::Timer),
                       Metric::new("some.timer", 6.3, MetricKind::Timer),
                       Metric::new("some.timer", 4.3, MetricKind::Timer),
                       Metric::new("some.timer", 6.1, MetricKind::Timer),
                       Metric::new("some.timer", 0.7, MetricKind::Timer),
                       Metric::new("some.timer", 3.7, MetricKind::Timer),
                       Metric::new("some.timer", 12.1, MetricKind::Timer)];
        for m in metrics.iter() {
            buckets.add(&m);
        }
        buckets
    }

    fn assert_float(expected: &str, value: &f64) {
        assert_eq!(expected, format!("{:.*}", 3, value));
    }

    #[test]
    fn test_process_timer_data() {
        let mut buckets = make_buckets();
        process(&mut buckets);

        assert_eq!(Some(&0.7), buckets.timer_data().get("some.timer.min"));
        assert_eq!(Some(&34.7), buckets.timer_data().get("some.timer.max"));
        assert_eq!(Some(&38.0), buckets.timer_data().get("some.timer.count"));
        assert_float("6.926",
                     buckets.timer_data().get("some.timer.mean").unwrap());
        assert_float("4.300",
                     buckets.timer_data().get("some.timer.median").unwrap());
        assert_float("8.439",
                     buckets.timer_data().get("some.timer.stddev").unwrap());
        assert_float("18.800",
                     buckets.timer_data().get("some.timer.upper_90").unwrap());
        assert_float("29.100",
                     buckets.timer_data().get("some.timer.upper_95").unwrap());
        assert_float("34.200",
                     buckets.timer_data().get("some.timer.upper_99").unwrap());
    }

    #[test]
    fn test_set_internal_metrics() {
        let mut buckets = make_buckets();
        process(&mut buckets);

        assert_eq!(Some(&0.0), buckets.counters().get("statsd.processing_time"));
    }
}
