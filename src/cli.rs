//! Provides the CLI option parser
//!
//! Used to parse the argv/config file into a struct that
//! the server can consume and use as configuration data.

use docopt::Docopt;

static USAGE: &'static str = "
Usage: statsd [options]
       statsd --help

Options:
  -h, --help                Print help information.
  -p, --port=<p>            The UDP port to bind to [default: 8125].
  --flush-interval=<p>      How frequently to flush metrics to the backends in seconds. [default: 10].
  --console                 Enable the console backend.
  --graphite                Enable the graphite backend.
  --graphite-port=<p>       The port graphite/carbon is running on. [default: 2003].
  --graphite-host=<p>       The host graphite/carbon is running on. [default: 127.0.0.1].
  --admin-host=<p>          The host to bind the management server on. [default: 127.0.0.1].
  --admin-port=<p>          The port to bind the management server to. [default: 8126].
  --statsd                  Enable the statsd backend.
  --statsd-port=<p>         The port other statsd is running on. [default: 8125].
  --statsd-host=<p>         The host other statsd is running on. [default: 127.0.0.1].
  --statsd-packet-size=<p>  The maximum statsd packet size that will be sent [default: 16384].
";

/// Holds the parsed command line arguments
#[derive(Deserialize, Debug)]
pub struct Args {
    pub flag_port: u16,
    pub flag_admin_port: u16,
    pub flag_admin_host: String,
    pub flag_flush_interval: u64,
    pub flag_console: bool,
    pub flag_graphite: bool,
    pub flag_graphite_port: u16,
    pub flag_graphite_host: String,
    pub flag_statsd: bool,
    pub flag_statsd_port: u16,
    pub flag_statsd_host: String,
    pub flag_statsd_packet_size: usize,
    pub flag_help: bool,
}

pub fn parse_args() -> Args {
    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());
    args
}
