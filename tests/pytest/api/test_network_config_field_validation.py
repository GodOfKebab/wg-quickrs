from tests.pytest.conftest import setup_wg_quickrs_agent
from tests.pytest.helpers import get_this_peer_id, get_test_peer_data, get_test_connection_data, deep_get, get_paths
import pytest
import requests
from ruamel.yaml import YAML
yaml = YAML()
yaml.preserve_quotes = True


@pytest.mark.parametrize(
    "field_name,field_value,expected_status,test_description",
    [
        # Basic string fields
        ("name", "updated-host-name", 200, "peer name change"),
        ("name", "", 400, "empty peer name validation"),
        ("address", "10.0.34.50", 200, "peer address change"),
        ("address", "", 400, "empty peer address validation"),
        ("address", "invalid-ip", 400, "invalid peer address format"),
        ("address", "999.999.999.999", 400, "out of range IP address"),
        ("address", "10.0.0", 400, "incomplete IP address"),
        ("address", "10.0.34.0", 400, "network address (boundary)"),
        ("address", "10.0.34.255", 400, "broadcast address (boundary)"),
        ("address", "192.168.1.1", 400, "address not in subnet"),
        ("kind", "laptop", 200, "peer kind change to laptop"),

        # EnabledValue fields - Icon
        ({"icon": {"enabled": True, "src": "data:image/png;base64,..."}}, None, 200, "peer icon enabled with a fake-base64 icon"),
        ({"icon": {"enabled": False, "src": ""}}, None, 200, "peer icon disabled"),

        # EnabledValue fields - DNS
        ({"dns": {"enabled": True, "addresses": ["8.8.8.8"]}}, None, 200, "peer DNS enabled with Google DNS"),
        ({"dns": {"enabled": False, "addresses": []}}, None, 200, "peer DNS disabled"),
        ({"dns": {"enabled": True, "addresses": []}}, None, 400, "empty DNS value when enabled"),
        ({"dns": {"enabled": True, "addresses": ["invalid-dns"]}}, None, 400, "invalid DNS format"),

        # EnabledValue fields - MTU
        ({"mtu": {"enabled": True, "value": 1420}}, None, 200, "peer MTU enabled with 1420"),
        ({"mtu": {"enabled": True, "value": 10000}}, None, 200, "peer MTU maximum valid value"),
        ({"mtu": {"enabled": True, "value": 10001}}, None, 400, "peer MTU above maximum"),
        ({"mtu": {"enabled": True, "value": -1}}, None, 400, "peer MTU negative value"),
        ({"mtu": {"enabled": False, "value": ""}}, None, 400, "peer MTU disabled parse error"),
        ({"mtu": {"enabled": True, "value": ""}}, None, 400, "empty MTU value when enabled parse error"),
        ({"mtu": {"enabled": True, "value": "invalid"}}, None, 400, "invalid MTU format"),

        # Scripts - individual script types
        ({"scripts": {"pre_up": [{"enabled": True, "script": "echo 'pre up script';"}]}}, None, 200, "peer pre_up script"),
        ({"scripts": {"post_up": [{"enabled": True, "script": "echo 'post up script';"}]}}, None, 200, "peer post_up script"),
        ({"scripts": {"pre_down": [{"enabled": True, "script": "echo 'pre down script';"}]}}, None, 200, "peer pre_down script"),
        ({"scripts": {"post_down": [{"enabled": True, "script": "echo 'post down script';"}]}}, None, 200, "peer post_down script"),

        # Scripts - multiple commands
        ({"scripts": {"pre_up": [
            {"enabled": True, "script": "echo 'command 1';"},
            {"enabled": True, "script": "echo 'command 2';"}
        ]}}, None, 200, "peer multiple pre_up scripts"),

        # Scripts - disabled
        ({"scripts": {"pre_up": [{"enabled": False, "script": ""}]}}, None, 200, "peer disabled pre_up script"),

        # Scripts - all types together
        ({"scripts": {
            "pre_up": [{"enabled": True, "script": "echo 'pre up';"}],
            "post_up": [{"enabled": True, "script": "echo 'post up';"}],
            "pre_down": [{"enabled": True, "script": "echo 'pre down';"}],
            "post_down": [{"enabled": True, "script": "echo 'post down';"}]
        }}, None, 200, "peer all script types"),

        # Scripts - validation errors
        ({"scripts": {"pre_up": [{"enabled": True, "script": "echo 'missing semicolon'"}]}}, None, 400, "script missing semicolon"),
        ({"scripts": {"pre_up": [{"enabled": True, "script": ""}]}}, None, 400, "script empty with enabled"),
        ({"scripts": {"pre_up": [{"enabled": True, "script": "echo ok"}]}}, None, 400, "script without semicolon"),
        ({"scripts": {"pre_up": [{"enabled": True, "script": "   "}]}}, None, 400, "script whitespace only"),
        ({"scripts": {
            "pre_up": [{"enabled": True, "script": "echo 'pre up';"}],
            "post_up": [{"enabled": True, "script": "echo 'post up'"}],
            "pre_down": [{"enabled": True, "script": "echo 'pre down';"}],
            "post_down": [{"enabled": True, "script": "echo 'post down';"}]
        }}, None, 400, "script without semicolon"),

        ("private_key", "kL+YuwGKNS8bNnPUVdnGDp7jF5BZs1vp1UxK2Xv+JX0=", 200, "peer private key change"),
        ("private_key", "", 400, "empty peer private key validation"),
        ("private_key", "invalid-key-format", 400, "invalid peer private key format"),

        # Peer Amnezia parameters - Jc
        ({"amnezia_parameters": {"jc": 0}}, None, 200, "peer amnezia jc valid value"),
        ({"amnezia_parameters": {"jc": -1}}, None, 200, "peer amnezia jc minimum value"),
        ({"amnezia_parameters": {"jc": 128}}, None, 200, "peer amnezia jc maximum value"),
        ({"amnezia_parameters": {"jc": -2}}, None, 400, "peer amnezia jc below minimum"),
        ({"amnezia_parameters": {"jc": 129}}, None, 400, "peer amnezia jc above maximum"),

        # Peer Amnezia parameters - Jmin
        ({"amnezia_parameters": {"jmin": 100}}, None, 200, "peer amnezia jmin valid value"),
        ({"amnezia_parameters": {"jmin": 0}}, None, 400, "peer amnezia jmin zero"),
        ({"amnezia_parameters": {"jmin": 1280}}, None, 400, "peer amnezia jmin at boundary"),

        # Peer Amnezia parameters - Jmax
        ({"amnezia_parameters": {"jmax": 200}}, None, 200, "peer amnezia jmax valid value"),
        ({"amnezia_parameters": {"jmax": 1280}}, None, 200, "peer amnezia jmax maximum value"),
        ({"amnezia_parameters": {"jmax": 0}}, None, 400, "peer amnezia jmax zero"),
        ({"amnezia_parameters": {"jmax": 1281}}, None, 400, "peer amnezia jmax above maximum"),

        # Peer Amnezia parameters - Jmin/Jmax relationship (test together)
        ({"amnezia_parameters": {"jmin": 50, "jmax": 100}}, None, 200, "peer amnezia jmin and jmax valid"),
        ({"amnezia_parameters": {"jmin": 1, "jmax": 100}}, None, 200, "peer amnezia jmin minimum with valid jmax"),
        ({"amnezia_parameters": {"jmin": 100, "jmax": 1280}}, None, 200, "peer amnezia jmax maximum with valid jmin"),
        ({"amnezia_parameters": {"jmin": 1, "jmax": 1280}}, None, 200, "peer amnezia jmin min and jmax max"),
        ({"amnezia_parameters": {"jmin": 100, "jmax": 100}}, None, 400, "peer amnezia jmin equals jmax"),
        ({"amnezia_parameters": {"jmin": 150, "jmax": 100}}, None, 400, "peer amnezia jmin greater than jmax"),

        # Multiple fields combination
        ({"name": "multi-field-test", "dns": {"enabled": True, "addresses": ["8.8.8.8"]}, "mtu": {"enabled": True, "value": 1420}}, None, 200, "multiple peer fields"),
        ({"amnezia_parameters": {"jc": 10, "jmin": 50, "jmax": 150}}, None, 200, "peer amnezia all fields"),
    ],
)
def test_patch_peer_field_changes(setup_wg_quickrs_agent, field_name, field_value, expected_status, test_description):
    """Parameterized test for all peer field changes."""
    base_url = setup_wg_quickrs_agent("no_auth_multi_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    # Change other-peer1
    peer_id = "6e9a8440-f884-4b54-bfe7-b982f15e40fd"

    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.load(stream)

    # Handle different parameter formats
    if isinstance(field_name, dict):
        changed_fields = field_name
    else:
        changed_fields = {field_name: field_value}

    change_sum = {
        "changed_fields": {
            "peers": {
                peer_id: changed_fields
            }
        }
    }

    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == expected_status

    # yaml validation
    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.load(stream)

    if response.status_code == 200:
        if isinstance(field_name, str):
            assert new_conf['network']['peers'][peer_id][field_name] == field_value
        else:
            for field_name_key, field_name_value in field_name.items():
                if field_name_key == 'scripts':
                    for script_type, script_value in field_name_value.items():
                        assert new_conf['network']['peers'][peer_id][field_name_key][script_type] == script_value
                elif field_name_key == 'amnezia_parameters':
                    for amnezia_field_key, amnezia_field_value in field_name_value.items():
                        assert new_conf['network']['peers'][peer_id]['amnezia_parameters'][amnezia_field_key] == amnezia_field_value
                else:
                    assert new_conf['network']['peers'][peer_id][field_name_key] == field_name_value


@pytest.mark.parametrize(
    "field_name,field_value,expected_status,test_description",
    [
        # Connection enabled/disabled
        ("enabled", True, 200, "connection enable"),
        ("enabled", False, 200, "connection disable"),

        # Pre-shared key variations
        ("pre_shared_key", "iF9xlxiI3W/p9LSZ5QhT/4Rk6IHi8v5NzA/UTUdPOVI=", 200, "pre-shared key change"),
        ("pre_shared_key", "uPd20wN+DtKipuxso46CmA7nY+rVQiWMnTK190e48FA=", 200, "different pre-shared key"),
        ("pre_shared_key", "", 400, "empty pre-shared key"),

        # Allowed IPs variations
        ("allowed_ips_a_to_b", ["0.0.0.0/0"], 200, "allowed_ips_a_to_b all traffic"),
        ("allowed_ips_a_to_b", ["192.168.1.0/24"], 200, "allowed_ips_a_to_b local network"),
        ("allowed_ips_a_to_b", ["not-a-subnet"], 400, "allowed_ips_a_to_b validation error"),
        ("allowed_ips_a_to_b", ["10.0.0.0/33"], 400, "allowed_ips_a_to_b invalid prefix"),
        ("allowed_ips_a_to_b", ["10.0.0"], 400, "allowed_ips_a_to_b incomplete CIDR"),
        ("allowed_ips_a_to_b", ["999.999.999.999/24"], 400, "allowed_ips_a_to_b invalid IP in CIDR"),

        ("allowed_ips_b_to_a", ["0.0.0.0/0"], 200, "allowed_ips_b_to_a all traffic"),
        ("allowed_ips_b_to_a", ["192.168.1.0/24"], 200, "allowed_ips_b_to_a local network"),
        ("allowed_ips_b_to_a", ["not-a-subnet"], 400, "allowed_ips_b_to_a validation error"),
        ("allowed_ips_b_to_a", ["10.0.0.0/33"], 400, "allowed_ips_b_to_a invalid prefix"),
        ("allowed_ips_b_to_a", ["10.0.0"], 400, "allowed_ips_b_to_a incomplete CIDR"),
        ("allowed_ips_b_to_a", ["999.999.999.999/24"], 400, "allowed_ips_b_to_a invalid IP in CIDR"),

        # Persistent keepalive variations
        ("persistent_keepalive", {"enabled": True, "period": 25}, 200, "persistent keepalive 25 seconds"),
        ("persistent_keepalive", {"enabled": True, "period": 1}, 200, "persistent keepalive minimum value"),
        ("persistent_keepalive", {"enabled": True, "period": 65535}, 200, "persistent keepalive maximum value"),
        ("persistent_keepalive", {"enabled": True, "period": 0}, 400, "persistent keepalive zero with enabled"),
        ("persistent_keepalive", {"enabled": False, "period": ""}, 400, "persistent keepalive validation error"),
        ("persistent_keepalive", {"enabled": True, "period": ""}, 400, "persistent keepalive validation error"),
        ("persistent_keepalive", {"enabled": True, "period": -1}, 400, "persistent keepalive negative value"),
        ("persistent_keepalive", {"enabled": True, "period": 70000}, 400, "persistent keepalive above maximum"),

        # Multiple fields combination
        ({"pre_shared_key": "iF9xlxiI3W/p9LSZ5QhT/4Rk6IHi8v5NzA/UTUdPOVI=", "allowed_ips_a_to_b": ["0.0.0.0/0"]}, None, 200, "multiple peer fields"),
    ],
)
def test_patch_connection_field_changes(setup_wg_quickrs_agent, field_name, field_value, expected_status, test_description):
    """Parameterized test for all connection field changes."""
    base_url = setup_wg_quickrs_agent("no_auth_multi_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.load(stream)

    # Setup: Create a test connection
    this_peer = "0ed989c6-6dba-4e3c-8034-08adf4262d9e"
    other_peer1 = "6e9a8440-f884-4b54-bfe7-b982f15e40fd"
    other_peer1_this_peer_connection_id = f"{other_peer1}*{this_peer}"

    # Handle different parameter formats
    if isinstance(field_name, dict):
        changed_fields = field_name
    else:
        changed_fields = {field_name: field_value}

    change_sum = {
        "changed_fields": {
            "connections": {
                other_peer1_this_peer_connection_id: changed_fields
            }
        }
    }

    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == expected_status

    # yaml validation
    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.load(stream)

    if response.status_code == 200:
        if isinstance(field_name, str):
            assert new_conf['network']['connections'][other_peer1_this_peer_connection_id][field_name] == field_value
        else:
            for field_name_key, field_name_value in field_name.items():
                assert new_conf['network']['connections'][other_peer1_this_peer_connection_id][field_name_key] == field_name_value


@pytest.mark.parametrize(
    "peer_data_variant,expected_status,test_description",
    [
        # Different endpoint configurations
        ({"endpoint": {"enabled": True, "address": { "ipv4_and_port": {"ipv4": "192.168.1.100", "port": 51111} }}}, 200, "add peer with endpoint"),
        ({"endpoint": {"enabled": True, "address": { "hostname_and_port": {"hostname": "example.com", "port": 51820} }}}, 200, "add peer with hostname endpoint"),
        ({"endpoint": {"enabled": True, "address": { "hostname_and_port": {"hostname": "my-server.example.org", "port": 443} }}}, 200, "add peer with subdomain endpoint"),
        ({"endpoint": {"enabled": False, "address": "none"}}, 200, "add peer without endpoint"),
        ({"endpoint": {"enabled": True, "address": "none"}}, 400, "endpoint validation error - enabled with none"),
        ({"endpoint": {"enabled": True, "address": { "hostname_and_port": {"hostname": "", "port": 51820} }}}, 400, "endpoint validation error - empty hostname"),
        ({"endpoint": {"enabled": True, "address": { "ipv4_and_port": {"ipv4": "invalid-ip", "port": 51820} }}}, 400, "endpoint validation error - invalid IP"),
        ({"endpoint": {"enabled": True, "address": { "ipv4_and_port": {"ipv4": "999.999.999.999", "port": 51820} }}}, 400, "endpoint validation error - out of range IP"),

        # Different peer kind
        ({"kind": "desktop"}, 200, "add desktop peer"),
        ({"kind": ""}, 200, "keep it empty"),

        # Different icon configurations
        ({"icon": {"enabled": True, "src": "custom-icon"}}, 200, "add peer with custom icon"),
        ({"icon": {"enabled": False, "src": ""}}, 200, "add peer without icon"),
        ({"icon": {"enabled": True, "src": ""}}, 400, "icon validation error"),

        # Different DNS configurations
        ({"dns": {"enabled": True, "addresses": ["8.8.8.8"]}}, 200, "add peer with Google DNS"),
        ({"dns": {"enabled": True, "addresses": ["1.1.1.1"]}}, 200, "add peer with Cloudflare DNS"),
        ({"dns": {"enabled": True, "addresses": ["8.8.8.8", "1.1.1.1"]}}, 200, "add peer with multiple DNS"),
        ({"dns": {"enabled": False, "addresses": []}}, 200, "add peer with DNS disabled"),
        ({"dns": {"enabled": True, "addresses": []}}, 400, "DNS validation error - empty addresses"),
        ({"dns": {"enabled": True, "addresses": ["invalid"]}}, 400, "DNS validation error - invalid address"),
        ({"dns": {"enabled": True, "addresses": ["8.8.8.8", "invalid"]}}, 400, "DNS validation error - one invalid in list"),
        ({"dns": {"enabled": True, "addresses": ["999.999.999.999"]}}, 400, "DNS validation error - out of range IP"),

        # Different MTU configurations
        ({"mtu": {"enabled": True, "value": 1420}}, 200, "add peer with MTU 1420"),
        ({"mtu": {"enabled": True, "value": 0}}, 400, "MTU validation error - 0"),
        ({"mtu": {"enabled": True, "value": 10000}}, 200, "add peer with MTU maximum"),
        ({"mtu": {"enabled": False, "value": 30000}}, 200, "add peer with MTU disabled"),
        ({"mtu": {"enabled": True, "value": ""}}, 400, "MTU validation error - empty"),
        ({"mtu": {"enabled": True, "value": 10001}}, 400, "MTU validation error - above max"),
        ({"mtu": {"enabled": True, "value": -1}}, 400, "MTU validation error - negative"),

        # With scripts
        ({"scripts": {
            "pre_up": [{"enabled": True, "script": "echo 'starting';"}],
            "post_up": [{"enabled": True, "script": "echo 'started';"}],
            "pre_down": [{"enabled": True, "script": "echo 'stopping';"}],
            "post_down": [{"enabled": True, "script": "echo 'stopped';"}]
        }}, 200, "add peer with all scripts"),
        ({"scripts": {
            "pre_up": [{"enabled": True, "script": "echo 'starting';"}],
            "post_up": [{"enabled": True, "script": "echo 'started';"}],
            "pre_down": [{"enabled": True, "script": "echo 'stopping';"}],
            "post_down": [{"enabled": True, "script": "echo 'stopped'"}]
        }}, 400, "scripts validation error"),
    ],
)
def test_add_peer_variants(setup_wg_quickrs_agent, peer_data_variant, expected_status, test_description):
    """Parameterized test for adding peers with different configurations."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.load(stream)

    peer_id = f"71c565c3-e5c7-45b6-9f21-3d26c9b07d06"
    peer_data = get_test_peer_data()

    # Update peer data with variant
    peer_data.update(peer_data_variant)
    # Ensure unique address
    peer_data["address"] = f"10.0.34.101"

    change_sum = {
        "added_peers": {
            peer_id: peer_data
        }
    }

    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == expected_status

    # yaml validation
    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.load(stream)

    if response.status_code == 200:
        for field_name_key, field_name_value in peer_data.items():
            if field_name_key == 'endpoint':
                assert new_conf['network']['peers'][peer_id][field_name_key]['enabled'] == field_name_value['enabled']
                if 'ipv4_and_port' in field_name_value['address']:
                    assert new_conf['network']['peers'][peer_id][field_name_key]['address'] == field_name_value['address']['ipv4_and_port']
                elif 'hostname_and_port' in field_name_value['address']:
                    assert new_conf['network']['peers'][peer_id][field_name_key]['address'] == field_name_value['address']['hostname_and_port']
                else:
                    assert new_conf['network']['peers'][peer_id][field_name_key]['address'] == field_name_value['address']
            else:
                assert new_conf['network']['peers'][peer_id][field_name_key] == field_name_value


@pytest.mark.parametrize(
    "connection_data_variant,expected_status,test_description",
    [
        # Different enabled states
        ({"enabled": True}, 200, "add enabled connection"),
        ({"enabled": False}, 200, "add disabled connection"),

        # Different allowed IPs configurations
        ({"allowed_ips_a_to_b": ["0.0.0.0/0"], "allowed_ips_b_to_a": ["0.0.0.0/0"]}, 200, "connection with full routing"),
        ({"allowed_ips_a_to_b": ["192.168.1.0/24"], "allowed_ips_b_to_a": ["10.0.34.0/24"]}, 200, "connection with limited routing"),
        ({"allowed_ips_a_to_b": ["172.16.0.0/16"], "allowed_ips_b_to_a": ["172.16.0.0/16"]}, 200, "connection with private network routing"),
        ({"allowed_ips_a_to_b": ["not-a-subnet"], "allowed_ips_b_to_a": ["172.16.0.0/16"]}, 400, "allowed_ips_a_to_b validation error"),
        ({"allowed_ips_a_to_b": ["172.16.0.0/16"], "allowed_ips_b_to_a": ["not-a-subnet"]}, 400, "allowed_ips_b_to_a validation error"),

        # Different persistent keepalive configurations
        ({"persistent_keepalive": {"enabled": True, "period": 25}}, 200, "connection with 25s keepalive"),
        ({"persistent_keepalive": {"enabled": False, "period": 0}}, 200, "connection without keepalive"),
        ({"persistent_keepalive": {"enabled": True, "period": 0}}, 400, "persistent_keepalive validation error"),

        # Different pre-shared key configurations
        ({"pre_shared_key": ""}, 400, "connection without pre-shared key"),
        ({"pre_shared_key": "QjF2m3eEcOuBjVqE1K5yB6z9Tf1Hk8qW2aXvNc5uE0o="}, 200, "connection with pre-shared key"),
    ],
)
def test_add_connection_variants(setup_wg_quickrs_agent, connection_data_variant, expected_status, test_description):
    """Parameterized test for adding connections with different configurations."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.load(stream)

    # Setup: Create test peers
    peer1_id = "71c565c3-e5c7-45b6-9f21-3d26c9b07d06"
    peer2_id = "349950ac-671f-4ba4-825e-778ebdf79d01"
    connection_id = f"{peer1_id}*{peer2_id}"

    peer_data = get_test_peer_data()
    connection_data = get_test_connection_data()

    # Update connection data with variant
    connection_data.update(connection_data_variant)

    setup_change_sum = {
        "added_peers": {
            peer1_id: {**peer_data, "address": f"10.0.34.160"},
            peer2_id: {**peer_data, "address": f"10.0.34.190"}
        },
        "added_connections": {
            connection_id: connection_data
        }
    }

    response = requests.patch(f"{base_url}/api/network/config", json=setup_change_sum)
    assert response.status_code == expected_status

    # yaml validation
    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.load(stream)

    if response.status_code == 200:
        for field_name_key, field_name_value in connection_data.items():
            assert new_conf['network']['connections'][connection_id][field_name_key] == field_name_value


@pytest.mark.parametrize(
    "field_name,field_value,expected_status,test_description",
    [
        # Amnezia enabled/disabled
        ({"amnezia_parameters": {"enabled": True}}, None, 200, "network amnezia enabled"),
        ({"amnezia_parameters": {"enabled": False}}, None, 200, "network amnezia disabled"),

        # S1 parameter variations
        ({"amnezia_parameters": {"s1": 100}}, None, 200, "network amnezia s1 valid value"),
        ({"amnezia_parameters": {"s1": 0}}, None, 200, "network amnezia s1 minimum value"),
        ({"amnezia_parameters": {"s1": 1132}}, None, 200, "network amnezia s1 maximum value"),
        ({"amnezia_parameters": {"s1": 1133}}, None, 400, "network amnezia s1 above maximum"),
        ({"amnezia_parameters": {"s1": 2000}}, None, 400, "network amnezia s1 too high"),

        # S2 parameter variations
        ({"amnezia_parameters": {"s2": 200}}, None, 200, "network amnezia s2 valid value"),
        ({"amnezia_parameters": {"s2": 0}}, None, 200, "network amnezia s2 minimum value"),
        ({"amnezia_parameters": {"s2": 1188}}, None, 200, "network amnezia s2 maximum value"),
        ({"amnezia_parameters": {"s2": 1189}}, None, 400, "network amnezia s2 above maximum"),
        ({"amnezia_parameters": {"s2": 2000}}, None, 400, "network amnezia s2 too high"),

        # S1 and S2 relationship validation
        ({"amnezia_parameters": {"s1": 100, "s2": 200}}, None, 200, "network amnezia s1 and s2 valid"),
        ({"amnezia_parameters": {"s1": 100, "s2": 156}}, None, 400, "network amnezia s1+56==s2 forbidden"),

        # H1 parameter variations
        ({"amnezia_parameters": {"h1": 1}}, None, 200, "network amnezia h1 valid value"),
        ({"amnezia_parameters": {"h1": 0}}, None, 200, "network amnezia h1 zero"),
        ({"amnezia_parameters": {"h1": 4294967295}}, None, 200, "network amnezia h1 max u32"),

        # H2 parameter variations
        ({"amnezia_parameters": {"h2": 2}}, None, 200, "network amnezia h2 valid value"),
        ({"amnezia_parameters": {"h2": 0}}, None, 200, "network amnezia h2 zero"),
        ({"amnezia_parameters": {"h2": 4294967295}}, None, 200, "network amnezia h2 max u32"),

        # H3 parameter variations
        ({"amnezia_parameters": {"h3": 3}}, None, 200, "network amnezia h3 valid value"),
        ({"amnezia_parameters": {"h3": 0}}, None, 200, "network amnezia h3 zero"),
        ({"amnezia_parameters": {"h3": 4294967295}}, None, 200, "network amnezia h3 max u32"),

        # H4 parameter variations
        ({"amnezia_parameters": {"h4": 4}}, None, 200, "network amnezia h4 valid value"),
        ({"amnezia_parameters": {"h4": 0}}, None, 200, "network amnezia h4 zero"),
        ({"amnezia_parameters": {"h4": 4294967295}}, None, 200, "network amnezia h4 max u32"),

        # Multiple fields combination
        ({"amnezia_parameters": {"enabled": True, "s1": 100, "s2": 200, "h1": 1, "h2": 2, "h3": 3, "h4": 4}}, None, 200, "network amnezia all fields"),
    ],
)
def test_patch_network_field_changes(setup_wg_quickrs_agent, field_name, field_value, expected_status, test_description):
    """Parameterized test for all network field changes."""
    base_url = setup_wg_quickrs_agent("no_auth_multi_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.load(stream)

    # Handle different parameter formats
    if isinstance(field_name, dict):
        changed_fields = field_name
    else:
        changed_fields = {field_name: field_value}

    change_sum = {
        "changed_fields": {
            "network": changed_fields
        }
    }

    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == expected_status

    # yaml validation
    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.load(stream)

    if response.status_code == 200:
        for field_name_key, field_name_value in changed_fields.items():
            if field_name_key == 'amnezia_parameters':
                for amnezia_field_key, amnezia_field_value in field_name_value.items():
                    assert new_conf['network']['amnezia_parameters'][amnezia_field_key] == amnezia_field_value
            elif field_name_key == 'defaults':
                if 'peer' in field_name_value:
                    for peer_field_key, peer_field_value in field_name_value['peer'].items():
                        if peer_field_key == 'amnezia_parameters':
                            assert new_conf['network']['defaults']['peer']['amnezia_parameters'] == peer_field_value
                        else:
                            assert new_conf['network']['defaults']['peer'][peer_field_key] == peer_field_value
                if 'connection' in field_name_value:
                    for conn_field_key, conn_field_value in field_name_value['connection'].items():
                        assert new_conf['network']['defaults']['connection'][conn_field_key] == conn_field_value
            else:
                assert new_conf['network'][field_name_key] == field_name_value


@pytest.mark.parametrize(
    "field_name,field_value,expected_status,test_description",
    [
        # Default peer - kind
        ({"peer": {"kind": "laptop"}}, None, 200, "defaults peer kind change"),
        ({"peer": {"kind": "desktop"}}, None, 200, "defaults peer kind desktop"),
        ({"peer": {"kind": ""}}, None, 200, "defaults peer kind empty"),

        # Default peer - icon
        ({"peer": {"icon": {"enabled": True, "src": "data:image/png;base64,icon"}}}, None, 200, "defaults peer icon enabled"),
        ({"peer": {"icon": {"enabled": False, "src": ""}}}, None, 200, "defaults peer icon disabled"),
        ({"peer": {"icon": {"enabled": True, "src": ""}}}, None, 400, "defaults peer icon validation error"),

        # Default peer - DNS
        ({"peer": {"dns": {"enabled": True, "addresses": ["8.8.8.8"]}}}, None, 200, "defaults peer DNS enabled"),
        ({"peer": {"dns": {"enabled": False, "addresses": []}}}, None, 200, "defaults peer DNS disabled"),
        ({"peer": {"dns": {"enabled": True, "addresses": []}}}, None, 400, "defaults peer DNS validation error"),
        ({"peer": {"dns": {"enabled": True, "addresses": ["invalid"]}}}, None, 400, "defaults peer DNS invalid address"),

        # Default peer - MTU
        ({"peer": {"mtu": {"enabled": True, "value": 1420}}}, None, 200, "defaults peer MTU enabled"),
        ({"peer": {"mtu": {"enabled": False, "value": 0}}}, None, 200, "defaults peer MTU disabled"),
        ({"peer": {"mtu": {"enabled": True, "value": 0}}}, None, 400, "defaults peer MTU zero"),
        ({"peer": {"mtu": {"enabled": True, "value": 10001}}}, None, 400, "defaults peer MTU above maximum"),

        # Default peer - scripts
        ({"peer": {"scripts": {
            "pre_up": [{"enabled": True, "script": "echo 'pre up';"}],
            "post_up": [{"enabled": True, "script": "echo 'post up';"}],
            "pre_down": [{"enabled": True, "script": "echo 'pre down';"}],
            "post_down": [{"enabled": True, "script": "echo 'post down';"}]
        }}}, None, 200, "defaults peer all scripts"),
        ({"peer": {"scripts": {
            "pre_up": [{"enabled": True, "script": "echo 'missing semicolon'"}],
            "post_up": [],
            "pre_down": [],
            "post_down": []
        }}}, None, 400, "defaults peer script validation error"),
        ({"peer": {"scripts": {"pre_up": [{"enabled": True, "script": "echo 'only pre_up';"}]}}}, None, 200, "defaults peer partial script update pre_up"),
        ({"peer": {"scripts": {"post_down": [{"enabled": True, "script": "echo 'only post_down';"}]}}}, None, 200, "defaults peer partial script update post_down"),

        # Default peer - amnezia parameters
        ({"peer": {"amnezia_parameters": {"jc": 10, "jmin": 50, "jmax": 100}}}, None, 200, "defaults peer amnezia valid"),
        ({"peer": {"amnezia_parameters": {"jc": -1, "jmin": 1, "jmax": 1280}}}, None, 200, "defaults peer amnezia boundaries"),
        ({"peer": {"amnezia_parameters": {"jc": 200, "jmin": 50, "jmax": 100}}}, None, 400, "defaults peer amnezia jc invalid"),
        ({"peer": {"amnezia_parameters": {"jc": 10, "jmin": 100, "jmax": 50}}}, None, 400, "defaults peer amnezia jmin > jmax"),
        ({"peer": {"amnezia_parameters": {"jc": 5}}}, None, 200, "defaults peer partial amnezia jc only"),
        ({"peer": {"amnezia_parameters": {"jmin": 30, "jmax": 200}}}, None, 200, "defaults peer partial amnezia jmin and jmax"),

        # Default connection - persistent keepalive
        ({"connection": {"persistent_keepalive": {"enabled": True, "period": 25}}}, None, 200, "defaults connection keepalive enabled"),
        ({"connection": {"persistent_keepalive": {"enabled": False, "period": 0}}}, None, 200, "defaults connection keepalive disabled"),
        ({"connection": {"persistent_keepalive": {"enabled": True, "period": 0}}}, None, 400, "defaults connection keepalive zero"),
        ({"connection": {"persistent_keepalive": {"enabled": True, "period": 70000}}}, None, 400, "defaults connection keepalive above max"),

        # Multiple default fields
        ({"peer": {"kind": "server", "mtu": {"enabled": True, "value": 1420}}, "connection": {"persistent_keepalive": {"enabled": True, "period": 25}}}, None, 200, "defaults multiple fields"),
    ],
)
def test_patch_defaults_field_changes(setup_wg_quickrs_agent, field_name, field_value, expected_status, test_description):
    """Parameterized test for all defaults field changes."""
    base_url = setup_wg_quickrs_agent("no_auth_multi_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.load(stream)

    # Handle different parameter formats
    if isinstance(field_name, dict):
        changed_fields = field_name
    else:
        changed_fields = {field_name: field_value}

    change_sum = {
        "changed_fields": {
            "defaults": changed_fields
        }
    }

    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == expected_status

    # yaml validation
    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.load(stream)

    if response.status_code == 200:
        if 'peer' in changed_fields:
            for peer_field_key, peer_field_value in changed_fields['peer'].items():
                if peer_field_key == 'amnezia_parameters':
                    for amnezia_key, amnezia_value in peer_field_value.items():
                        assert new_conf['network']['defaults']['peer']['amnezia_parameters'][amnezia_key] == amnezia_value
                elif peer_field_key == 'scripts':
                    for script_type, script_value in peer_field_value.items():
                        assert new_conf['network']['defaults']['peer']['scripts'][script_type] == script_value
                else:
                    assert new_conf['network']['defaults']['peer'][peer_field_key] == peer_field_value
        if 'connection' in changed_fields:
            for conn_field_key, conn_field_value in changed_fields['connection'].items():
                assert new_conf['network']['defaults']['connection'][conn_field_key] == conn_field_value

