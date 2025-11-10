use clap::{Args, Subcommand};
use std::net::Ipv4Addr;
use uuid::Uuid;

#[derive(Subcommand, Debug)]
pub enum AddCommands {
    #[command(about = "Add a peer to the network")]
    Peer {
        #[command(flatten)]
        options: AddPeerOptions,
    },
    #[command(about = "Add a connection between two peers")]
    Connection {
        #[command(flatten)]
        options: AddConnectionOptions,
    },
}

#[derive(Args, Debug)]
pub struct AddPeerOptions {
    #[arg(long, default_value = None, long_help = "Skip all prompts and exit with error if required options are not provided")]
    pub no_prompt: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set peer name")]
    pub name: Option<String>,

    #[arg(long, default_value = None, long_help = "Set peer IPv4 address")]
    pub address: Option<Ipv4Addr>,

    #[arg(long, default_value = None, long_help = "Enable endpoint")]
    pub endpoint_enabled: Option<bool>,
    
    #[arg(long, default_value = None, long_help = "Set peer endpoint (hostname:port or ipv4:port)")]
    pub endpoint_address: Option<String>,
    
    #[arg(long, default_value = None, long_help = "Set peer kind (e.g., laptop, server, phone)", value_name = "laptop")]
    pub kind: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable icon")]
    pub icon_enabled: Option<bool>,
    
    #[arg(long, default_value = None, long_help = "Set peer icon source (URL or path)")]
    pub icon_src: Option<String>,

    #[arg(long, default_value = None, long_help = "Enable DNS")]
    pub dns_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set DNS address(es). Can be specified multiple times for multiple DNS addresses.", value_name = "1.1.1.1")]
    pub dns_addresses: Vec<Ipv4Addr>,

    #[arg(long, default_value = None, long_help = "Enable MTU")]
    pub mtu_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set MTU value", value_name = "1420")]
    pub mtu_value: Option<u16>,

    #[arg(long, default_value = None, long_help = "Enable PreUp script")]
    pub script_pre_up_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set PreUp script line(s). Can be specified multiple times for multiple script lines."
    )]
    pub script_pre_up_line: Vec<String>,

    #[arg(long, default_value = None, long_help = "Enable PostUp script")]
    pub script_post_up_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set PostUp script line(s). Can be specified multiple times for multiple script lines."
    )]
    pub script_post_up_line: Vec<String>,

    #[arg(long, default_value = None, long_help = "Enable PreDown script"
    )]
    pub script_pre_down_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set PreDown script line(s). Can be specified multiple times for multiple script lines."
    )]
    pub script_pre_down_line: Vec<String>,

    #[arg(long, default_value = None, long_help = "Enable PostDown script"
    )]
    pub script_post_down_enabled: Option<bool>,

    #[arg(long, default_value = None, num_args = 0.., long_help = "Set PostDown script line(s). Can be specified multiple times for multiple script lines."
    )]
    pub script_post_down_line: Vec<String>,
}

#[derive(Args, Debug)]
pub struct AddConnectionOptions {
    #[arg(long, default_value = None, long_help = "Skip all prompts and exit with error if required options are not provided")]
    pub no_prompt: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set first peer UUID")]
    pub first_peer: Option<Uuid>,

    #[arg(long, default_value = None, long_help = "Set second peer UUID")]
    pub second_peer: Option<Uuid>,

    #[arg(long, default_value = None, long_help = "Enable persistent keepalive")]
    pub persistent_keepalive_enabled: Option<bool>,

    #[arg(long, default_value = None, long_help = "Set persistent keepalive period in seconds", value_name = "25")]
    pub persistent_keepalive_period: Option<u16>,

    #[arg(long, default_value = None, long_help = "Set allowed IPs from the first peer to the second peer (comma-separated CIDR blocks)", value_name = "10.0.34.0/24")]
    pub allowed_ips_first_to_second: Option<String>,

    #[arg(long, default_value = None, long_help = "Set allowed IPs from the second peer to the first peer (comma-separated CIDR blocks)", value_name = "10.0.34.0/24")]
    pub allowed_ips_second_to_first: Option<String>,
}
