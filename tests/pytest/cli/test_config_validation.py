import pytest
import yaml
from tests.pytest.conftest import setup_wg_quickrs_folder
from tests.pytest.helpers import get_wg_quickrs_command
import subprocess


def set_nested_value(config, path, value):
    """Helper to set nested dictionary values using dot notation."""
    keys = path.split('.')
    current = config
    for key in keys[:-1]:
        if key not in current:
            current[key] = {}
        current = current[key]
    current[keys[-1]] = value


def create_invalid_config_file(wg_quickrs_config_file, field_path, invalid_value):
    """Create a temporary config file with an invalid field value."""
    with open(wg_quickrs_config_file) as stream:
        conf = yaml.safe_load(stream)
    set_nested_value(conf, field_path, invalid_value)
    yaml.safe_dump(conf, open(wg_quickrs_config_file, 'w'))


@pytest.mark.parametrize(
    "field_path,invalid_value,expected_error_contains",
    [
        # missing fields
        ("agent", None, "invalid config file format"),
        ("network", None, "invalid config file format"),

        # incorrect version
        ("version", "999.999.999", "conf::util::error::version_not_supported"),

        # Agent.web validation
        ("agent.web.address", "", "agent.web.address: address is not IPv4"),
        
        # Network basic fields
        ("network.identifier", "", "network.identifier: identifier cannot be empty"),
        ("network.subnet", "invalid-subnet", "network.subnet: subnet is not in CIDR format"),
        ("network.subnet", "192.168.1.1", "network.subnet: subnet is not in CIDR format"),
        ("network.this_peer", "invalid-uuid", "network.this_peer: peer_id needs to follow uuid4 standards"),
        ("network.this_peer", "", "network.this_peer: peer_id needs to follow uuid4 standards"),
        ("network.updated_at", "invalid-timestamp", "network.updated_at: invalid timestamp"),
        ("network.updated_at", "2023-13-45T99:99:99Z", "network.updated_at: invalid timestamp"),
        
        # Peer field validation
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.name", "", "name cannot be empty"),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.address", "invalid-ip", "address is not IPv4"),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.address", "", "address is not IPv4"),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.endpoint", {"enabled": True, "value": "invalid-endpoint"}, "endpoint is not IPv4 nor an FQDN"),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.icon", {"enabled": True, "value": ""}, "icon cannot be empty"),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.dns", {"enabled": True, "value": "invalid-dns"}, "DNS is invalid"),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.dns", {"enabled": True, "value": ""}, "DNS is invalid"),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.mtu", {"enabled": True, "value": "invalid"}, "MTU is invalid"),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.mtu", {"enabled": True, "value": "0"}, "MTU is invalid"),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.mtu", {"enabled": True, "value": "99999"}, "MTU is invalid"),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.private_key", "invalid-key", "private_key is not base64 encoded"),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.private_key", "", "private_key is not base64 encoded"),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.created_at", "invalid-timestamp", "created_at: invalid timestamp"),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.updated_at", "invalid-timestamp", "updated_at: invalid timestamp"),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.scripts.pre_up", [{"enabled": True, "value": "echo 'test'"}], "script needs to end with a semicolon"),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.scripts.post_up", [{"enabled": True, "value": "invalid-script"}], "script needs to end with a semicolon"),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.scripts.pre_down", [{"enabled": True, "value": "echo 'test' "}], "script needs to end with a semicolon"),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.scripts.post_down", [{"enabled": True, "value": ""}], "script needs to end with a semicolon"),

        # Connection field validation
        ("network.connections.9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e.pre_shared_key", "invalid-key", "pre_shared_key is not base64 encoded"),
        ("network.connections.9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e.pre_shared_key", "", "pre_shared_key is not base64 encoded"),
        ("network.connections.9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e.allowed_ips_a_to_b", "invalid-cidr", "AllowedIPs is not in CIDR format"),
        ("network.connections.9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e.allowed_ips_a_to_b", "192.168.1.1", "AllowedIPs is not in CIDR format"),
        ("network.connections.9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e.allowed_ips_b_to_a", "invalid-cidr", "AllowedIPs is not in CIDR format"),
        ("network.connections.9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e.allowed_ips_b_to_a", "192.168.1.1", "AllowedIPs is not in CIDR format"),
        ("network.connections.9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e.persistent_keepalive", {"enabled": True, "value": "invalid"}, "Persistent Keepalive is invalid"),
        ("network.connections.9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e.persistent_keepalive", {"enabled": True, "value": ""}, "Persistent Keepalive is invalid"),

        # Defaults field validation
        ("network.defaults.peer.endpoint", {"enabled": True, "value": "invalid-endpoint"}, "endpoint is not IPv4 nor an FQDN"),
        ("network.defaults.peer.icon", {"enabled": True, "value": ""}, "icon cannot be empty"),
        ("network.defaults.peer.dns", {"enabled": True, "value": "invalid-dns"}, "DNS is invalid"),
        ("network.defaults.peer.mtu", {"enabled": True, "value": "invalid"}, "MTU is invalid"),
        ("network.defaults.connection.persistent_keepalive", {"enabled": True, "value": "invalid"}, "Persistent Keepalive is invalid"),

        # Leases field validation
        ("network.leases", {"invalid-ip": {"peer_id": "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "valid_until": "2025-12-31T23:59:59Z"}}, "address is not IPv4"),
        ("network.leases", {"10.0.34.100": {"peer_id": "not-a-uuid", "valid_until": "2025-12-31T23:59:59Z"}}, "peer_id needs to follow uuid4 standards"),
        ("network.leases", {"10.0.34.100": {"peer_id": "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "valid_until": "invalid-timestamp"}}, "invalid timestamp"),
    ]
)
def test_config_validation_failures(setup_wg_quickrs_folder, field_path, invalid_value, expected_error_contains):
    """Test that configuration validation fails correctly for invalid field values."""
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = setup_wg_quickrs_folder("no_auth_multi_peer")

    # Create a config file with an invalid value
    create_invalid_config_file(wg_quickrs_config_file, field_path, invalid_value)

    result = subprocess.run(
        get_wg_quickrs_command() + ['agent', 'run'],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)
    assert result.returncode == 1
    assert expected_error_contains in result.stdout

