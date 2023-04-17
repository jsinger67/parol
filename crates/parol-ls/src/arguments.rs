use std::net::IpAddr;

use clap::Parser;

// Parol Language Server
#[derive(Debug, Parser)]
#[clap(author, version, about)]
pub struct Arguments {
    /// Server's IP address
    #[clap(
        short = 'a',
        long = "address",
        group = "tcp",
        default_value = "127.0.0.1"
    )]
    pub ip_address: IpAddr,
    /// Server's port
    #[clap(short = 's', long = "socket", group = "tcp", default_value = "7061")]
    pub port_number: u16,
    /// Use stdio
    #[clap(long = "stdio", conflicts_with = "tcp")]
    pub stdio: bool,
    /// Lookahead limit
    #[clap(short = 'k', long = "lookahead", default_value = "3")]
    pub lookahead: usize,
}
