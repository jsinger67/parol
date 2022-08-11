use std::net::IpAddr;

use clap::Parser;

// Parol Language Server
#[derive(Parser)]
#[clap(author, version, about)]
pub struct Config {
    /// Server's IP address
    #[clap(short = 'a', long = "address", default_value = "127.0.0.1")]
    pub ip_address: IpAddr,
    /// Server's port
    #[clap(short = 's', long = "socket", default_value = "7061")]
    pub port_number: u16,
    /// Lookahead limit
    #[clap(short = 'k', long = "lookahead", default_value = "5")]
    pub lookahead: usize,
}
