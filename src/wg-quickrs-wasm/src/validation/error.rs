use thiserror::Error;
use uuid::Uuid;

#[derive(Error, PartialEq, Debug)]
pub enum ValidationError {
    #[error("address is not IPv4")]
    NotIPv4Address(),
    #[error("port is not a valid number (1-65535)")]
    NotPortNumber(),
    #[error("TLS file is not found")]
    TlsFileNotFound(),
    #[error("TLS path is not a file (it is a directory or a symlink)")]
    TlsFileNotAFile(),
    #[error("gateway {0} is not found (possible options: {1})")]
    InterfaceNotFound(String, String),
    #[error("firewall utility {0} is not found (possible options: [{1}])")]
    FirewallUtilityNotFound(String, String),
    #[error("subnet is not in CIDR format")]
    NotCIDR(),
    #[error("uuid is invalid (not in v4 format)")]
    InvalidUuid(),
    #[error("network name cannot be empty")]
    EmptyNetworkName(),
    #[error("peer name cannot be empty")]
    EmptyPeerName(),
    #[error("address is not in the network subnet")]
    AddressNotInSubnet(),
    #[error("address is the subnet's network address and cannot be assigned")]
    AddressIsSubnetNetwork(),
    #[error("address is the subnet's broadcast address and cannot be assigned")]
    AddressIsSubnetBroadcast(),
    #[error("address is already taken by {0} ({1})")]
    AddressIsTaken(Uuid, String),
    #[error("address is already reserved for another peer")]
    AddressIsReserved(),
    #[error("endpoint is invalid")]
    InvalidEndpoint(),
    #[error("endpoint port is invalid")]
    InvalidEndpointPort(),
    #[error("icon cannot be empty when enabled")]
    EmptyIcon(),
    #[error("MTU is invalid (1-9999)")]
    InvalidMtu(),
    #[error("script missing a semicolon")]
    ScriptMissingSemicolon(),
    #[error("script missing a semicolon at line {0}")]
    ScriptMissingSemicolonAt(usize),
    #[error("key is not a valid WireGuard key (32 bytes, base64 encoded)")]
    NotWireGuardKey(),
    #[error("persistent_keepalive is invalid")]
    InvalidPersistentKeepalive(),
    #[error("allowed_ips is not in CIDR format")]
    InvalidAllowedIPs(),
}
pub type ValidationResult<T> = Result<T, ValidationError>;
