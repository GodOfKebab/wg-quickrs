import pytest
from tests.pytest.conftest import setup_wg_quickrs_folder
from tests.pytest.helpers import get_wg_quickrs_command
import subprocess
from ruamel.yaml import YAML
yaml = YAML()
yaml.preserve_quotes = True


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
        conf = yaml.load(stream)
    set_nested_value(conf, field_path, invalid_value)
    yaml.dump(conf, open(wg_quickrs_config_file, 'w'))


@pytest.mark.parametrize(
    "field_path,invalid_value,expected_errors",
    [
        # missing fields
        ("agent", None, ["invalid config file format"]),
        ("network", None, ["invalid config file format"]),

        # incorrect version
        ("version", "999.999.999", ["conf::util::error::version_not_supported"]),

        # Agent.web validation
        ("agent.web.address", "", ["ipv4"]),
        
        # Network basic fields
        ("network.name", "", ["network name cannot be empty"]),
        ("network.subnet", "invalid-subnet", ["ip"]),
        ("network.subnet", "192.168.1.1", ["ip"]),
        ("network.this_peer", "invalid-uuid", ["uuid"]),
        ("network.this_peer", "", ["uuid"]),
        ("network.updated_at", "invalid-timestamp", []),
        ("network.updated_at", "2023-13-45T99:99:99Z", []),
        
        # Peer field validation
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.name", "", ["peer name cannot be empty"]),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.address", "invalid-ip", ["ipv4"]),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.address", "", ["ipv4"]),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.endpoint", {"enabled": True, "address": { "ipv4_and_port": {"ipv4": "invalid-address", "port": 51111} }}, ["endpoint.address"]),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.icon", {"enabled": True, "src": ""}, ["icon cannot be empty"]),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.dns", {"enabled": True, "addresses": ["invalid-dns"]}, ["ipv4"]),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.dns", {"enabled": True, "addresses": [""]}, ["ipv4"]),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.mtu", {"enabled": True, "value": "invalid"}, ["u16"]),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.mtu", {"enabled": True, "value": "0"}, ["u16"]),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.mtu", {"enabled": True, "value": "1500"}, ["u16"]),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.mtu", {"enabled": True, "value": "99999"}, ["u16"]),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.private_key", "invalid-key", ["base64"]),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.private_key", "", ["32 bytes"]),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.created_at", "invalid-timestamp", []),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.updated_at", "invalid-timestamp", []),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.scripts.pre_up", [{"enabled": True, "script": "echo 'test'"}], ["semicolon"]),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.scripts.post_up", [{"enabled": True, "script": "invalid-script"}], ["semicolon"]),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.scripts.pre_down", [{"enabled": True, "script": "echo 'test' "}], ["semicolon"]),
        ("network.peers.0ed989c6-6dba-4e3c-8034-08adf4262d9e.scripts.post_down", [{"enabled": True, "script": ""}], ["semicolon"]),

        # Connection field validation
        ("network.connections.9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e.pre_shared_key", "invalid-key", ["base64"]),
        ("network.connections.9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e.pre_shared_key", "", ["32 bytes"]),
        ("network.connections.9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e.allowed_ips_a_to_b", "invalid-cidr", ["invalid type"]),
        ("network.connections.9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e.allowed_ips_a_to_b", ["invalid-cidr"], ["ip"]),
        ("network.connections.9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e.allowed_ips_a_to_b", ["192.168.1.1"], ["ip"]),
        ("network.connections.9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e.allowed_ips_b_to_a", "invalid-cidr", ["invalid type"]),
        ("network.connections.9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e.allowed_ips_b_to_a", ["invalid-cidr"], ["ip"]),
        ("network.connections.9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e.allowed_ips_b_to_a", ["192.168.1.1"], ["ip"]),
        ("network.connections.9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e.persistent_keepalive", {"enabled": True, "period": "invalid"}, ["u16"]),
        ("network.connections.9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e.persistent_keepalive", {"enabled": True, "period": ""}, ["u16"]),

        # Defaults field validation
        ("network.defaults.peer.endpoint", {"enabled": True, "address": { "ipv4_and_port": {"ipv4": "invalid-address", "port": 51111} }}, ["endpoint.address"]),
        ("network.defaults.peer.icon", {"enabled": True, "src": ""}, ["icon cannot be empty"]),
        ("network.defaults.peer.dns", {"enabled": True, "addresses": "invalid-dns"}, ["invalid type"]),
        ("network.defaults.peer.mtu", {"enabled": True, "value": "invalid"}, ["u16"]),
        ("network.defaults.connection.persistent_keepalive", {"enabled": True, "period": "invalid"}, ["u16"]),

        # reservations field validation
        ("network.reservations", {"invalid-ip": {"peer_id": "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "valid_until": "2025-12-31T23:59:59Z"}}, ["ipv4"]),
        ("network.reservations", {"10.0.34.100": {"peer_id": "not-a-uuid", "valid_until": "2025-12-31T23:59:59Z"}}, ["uuid"]),
        ("network.reservations", {"10.0.34.100": {"peer_id": "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "valid_until": "invalid-timestamp"}}, ["network.reservations.10.0.34.100.valid_until"]),
    ]
)
def test_config_validation_failures(setup_wg_quickrs_folder, field_path, invalid_value, expected_errors):
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

    # field path should show up in the error message except for private_key and pre_shared_key fields
    if not (("private_key" in field_path) or ("pre_shared_key" in field_path)):
        assert field_path in result.stdout
    for expected_error in expected_errors:
        assert expected_error in result.stdout.lower()

