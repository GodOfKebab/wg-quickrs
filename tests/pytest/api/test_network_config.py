import pytest
import requests
from typing import Dict, Any
from tests.pytest.conftest import setup_wg_quickrs_agent


def get_test_peer_data() -> Dict[str, Any]:
    """Get test peer data for adding peers."""
    return {
        "name": "test-peer",
        "address": "10.0.34.100",
        "endpoint": {
            "enabled": True,
            "value": "192.168.1.100:51820"
        },
        "kind": "laptop",
        "icon": {
            "enabled": False,
            "value": ""
        },
        "dns": {
            "enabled": True,
            "value": "1.1.1.1"
        },
        "mtu": {
            "enabled": False,
            "value": ""
        },
        "scripts": {
            "pre_up": [],
            "post_up": [],
            "pre_down": [],
            "post_down": []
        },
        "private_key": "cL+YuwGKNS8bNnPUVdnGDp7jF5BZs1vp1UxK2Xv+JX0="
    }


def get_test_connection_data() -> Dict[str, Any]:
    """Get test connection data for adding connections."""
    return {
        "enabled": True,
        "pre_shared_key": "QjF2m3eEcOuBjVqE1K5yB6z9Tf1Hk8qW2aXvNc5uE0o=",
        "allowed_ips_a_to_b": "0.0.0.0/0",
        "allowed_ips_b_to_a": "10.0.34.0/24",
        "persistent_keepalive": {
            "enabled": True,
            "value": "25"
        }
    }


def get_this_peer_id(base_url: str) -> str:
    """Helper to get this peer ID from summary."""
    response = requests.get(f"{base_url}/api/network/summary?only_digest=false")
    assert response.status_code == 200
    return response.json()["network"]["this_peer"]


def test_patch_forbidden_endpoint_change(setup_wg_quickrs_agent):
    """Test that changing host peer's endpoint is forbidden."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    this_peer_id = get_this_peer_id(base_url)

    change_sum = {
        "changed_fields": {
            "peers": {
                this_peer_id: {
                    "endpoint": {
                        "enabled": True,
                        "value": "192.168.1.100:51820"
                    }
                }
            }
        }
    }

    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == 403
    data = response.json()
    assert data["status"] == "forbidden"
    assert "can't change the host's endpoint" in data["message"]


def test_patch_peer_not_found(setup_wg_quickrs_agent):
    """Test changing a peer that doesn't exist."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    change_sum = {
        "changed_fields": {
            "peers": {
                "non-existent-peer-id": {
                    "name": "should-fail"
                }
            }
        }
    }

    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == 404
    data = response.json()
    assert data["status"] == "not_found"
    assert "does not exist" in data["message"]


def test_add_peer_with_leased_address(setup_wg_quickrs_agent):
    """Test adding a new peer with a leased address."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    response = requests.get(f"{base_url}/api/network/lease/address")
    assert response.status_code == 200

    peer_data = get_test_peer_data()
    reserved_peer_id = response.json()["peer_id"]
    peer_data["address"] = response.json()["address"]

    fake_peer_id = "not-a-uuid"
    change_sum_w_fake_peer_id = {
        "added_peers": {
            fake_peer_id: peer_data
        }
    }
    response = requests.patch(f"{base_url}/api/network/config", json=change_sum_w_fake_peer_id)
    assert response.status_code == 400
    assert "uuid" in response.json()["message"]

    another_peer_id = "a1c11ade-dd1a-4f5a-a6f9-3b6c6d10f416"
    change_sum_w_another_peer_id = {
        "added_peers": {
            another_peer_id: peer_data
        }
    }
    response = requests.patch(f"{base_url}/api/network/config", json=change_sum_w_another_peer_id)
    assert response.status_code == 403
    assert "reserved for another" in response.json()["message"]

    correct_change_sum = {
        "added_peers": {
            reserved_peer_id: peer_data
        }
    }
    response = requests.patch(f"{base_url}/api/network/config", json=correct_change_sum)
    assert response.status_code == 200


def test_add_bad_connection(setup_wg_quickrs_agent):
    """Test adding a bad connection."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    bad_connection_id = "non-a-connection-id"
    bad_setup_change_sum = {
        "added_connections": {
            bad_connection_id: get_test_connection_data()
        }
    }
    setup_response = requests.patch(f"{base_url}/api/network/config", json=bad_setup_change_sum)
    assert setup_response.status_code == 400
    assert "not a valid connection_id" in setup_response.json()["message"]

    peer1_id = "71c565c3-e5c7-45b6-9f21-3d26c9b07d06"
    peer2_id = "349950ac-671f-4ba4-825e-778ebdf79d01"
    fake_connection_id = f"{peer1_id}*{peer2_id}"
    fake_setup_change_sum = {
        "added_connections": {
            fake_connection_id: get_test_connection_data()
        }
    }
    setup_response = requests.patch(f"{base_url}/api/network/config", json=fake_setup_change_sum)
    assert setup_response.status_code == 400
    assert "'peer_id' doesn't exist" in setup_response.json()["message"]


def test_invalid_json(setup_wg_quickrs_agent):
    """Test sending invalid JSON."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    response = requests.patch(f"{base_url}/api/network/config", data="invalid json")
    assert response.status_code == 400
    data = response.json()
    assert "Invalid JSON" in data["error"]


def test_empty_change_sum(setup_wg_quickrs_agent):
    """Test sending empty change sum (nothing to update)."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    empty_change_sum = {}

    response = requests.patch(f"{base_url}/api/network/config", json=empty_change_sum)
    assert response.status_code == 400
    data = response.json()
    assert data["status"] == "bad_request"
    assert "nothing to update" in data["message"]


def test_connection_not_found(setup_wg_quickrs_agent):
    """Test modifying a connection that doesn't exist."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    change_sum = {
        "changed_fields": {
            "connections": {
                "non-existent-connection": {
                    "enabled": False
                }
            }
        }
    }

    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == 404
    data = response.json()
    assert data["status"] == "not_found"
    assert "does not exist" in data["message"]


# Comprehensive parameterized tests for all fields
@pytest.mark.parametrize(
    "field_name,field_value,expected_status,test_description",
    [
        # Basic string fields
        ("name", "updated-host-name", 200, "peer name change"),
        ("name", "", 400, "empty peer name validation"),
        ("kind", "server", 200, "peer kind change to server"),
        ("address", "10.0.34.50", 200, "peer address change"),
        ("address", "", 400, "empty peer address validation"),
        ("address", "invalid-ip", 400, "invalid peer address format"),
        ("private_key", "kL+YuwGKNS8bNnPUVdnGDp7jF5BZs1vp1UxK2Xv+JX0=", 200, "peer private key change"),
        ("private_key", "", 400, "empty peer private key validation"),
        ("private_key", "invalid-key-format", 400, "invalid peer private key format"),

        # EnabledValue fields - DNS
        ({"dns": {"enabled": True, "value": "8.8.8.8"}}, None, 200, "peer DNS enabled with Google DNS"),
        ({"dns": {"enabled": False, "value": ""}}, None, 200, "peer DNS disabled"),
        ({"dns": {"enabled": True, "value": ""}}, None, 400, "empty DNS value when enabled"),
        ({"dns": {"enabled": True, "value": "invalid-dns"}}, None, 400, "invalid DNS format"),

        # EnabledValue fields - MTU
        ({"mtu": {"enabled": True, "value": "1420"}}, None, 200, "peer MTU enabled with 1420"),
        ({"mtu": {"enabled": False, "value": ""}}, None, 200, "peer MTU disabled"),
        ({"mtu": {"enabled": True, "value": ""}}, None, 400, "empty MTU value when enabled"),
        ({"mtu": {"enabled": True, "value": "invalid"}}, None, 400, "invalid MTU format"),

        # EnabledValue fields - Icon
        ({"icon": {"enabled": True, "value": "laptop-icon"}}, None, 200, "peer icon enabled with laptop"),
        ({"icon": {"enabled": True, "value": "server-icon"}}, None, 200, "peer icon enabled with server"),
        ({"icon": {"enabled": True, "value": "mobile-icon"}}, None, 200, "peer icon enabled with mobile"),
        ({"icon": {"enabled": False, "value": ""}}, None, 200, "peer icon disabled"),

        # Scripts - individual script types
        ({"scripts": {"pre_up": [{"enabled": True, "value": "echo 'pre up script';"}]}}, None, 200, "peer pre_up script"),
        ({"scripts": {"post_up": [{"enabled": True, "value": "echo 'post up script';"}]}}, None, 200, "peer post_up script"),
        ({"scripts": {"pre_down": [{"enabled": True, "value": "echo 'pre down script';"}]}}, None, 200, "peer pre_down script"),
        ({"scripts": {"post_down": [{"enabled": True, "value": "echo 'post down script';"}]}}, None, 200, "peer post_down script"),

        # Scripts - multiple commands
        ({"scripts": {"pre_up": [
            {"enabled": True, "value": "echo 'command 1';"},
            {"enabled": True, "value": "echo 'command 2';"}
        ]}}, None, 200, "peer multiple pre_up scripts"),

        # Scripts - disabled
        ({"scripts": {"pre_up": [{"enabled": False, "value": ""}]}}, None, 200, "peer disabled pre_up script"),

        # Scripts - all types together
        ({"scripts": {
            "pre_up": [{"enabled": True, "value": "echo 'pre up';"}],
            "post_up": [{"enabled": True, "value": "echo 'post up';"}],
            "pre_down": [{"enabled": True, "value": "echo 'pre down';"}],
            "post_down": [{"enabled": True, "value": "echo 'post down';"}]
        }}, None, 200, "peer all script types"),

        # Multiple fields combination
        ({"name": "multi-field-test", "dns": {"enabled": True, "value": "8.8.8.8"}, "mtu": {"enabled": True, "value": "1420"}}, None, 200, "multiple peer fields"),
    ],
)
def test_patch_peer_field_changes(setup_wg_quickrs_agent, field_name, field_value, expected_status, test_description):
    """Comprehensive parameterized test for all peer field changes."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    this_peer_id = get_this_peer_id(base_url)

    # Handle different parameter formats
    if isinstance(field_name, dict):
        changed_fields = field_name
    else:
        changed_fields = {field_name: field_value}

    change_sum = {
        "changed_fields": {
            "peers": {
                this_peer_id: changed_fields
            }
        }
    }

    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == expected_status


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
        ("allowed_ips_a_to_b", "0.0.0.0/0", 200, "allowed_ips_a_to_b all traffic"),
        ("allowed_ips_a_to_b", "192.168.1.0/24", 200, "allowed_ips_a_to_b local network"),
        ("allowed_ips_a_to_b", "172.16.0.0/16", 200, "allowed_ips_a_to_b private network"),
        ("allowed_ips_a_to_b", "10.0.0.0/8", 200, "allowed_ips_a_to_b large private network"),
        ("allowed_ips_a_to_b", "not-a-subnet", 400, "allowed_ips_a_to_b validation error"),

        ("allowed_ips_b_to_a", "0.0.0.0/0", 200, "allowed_ips_b_to_a all traffic"),
        ("allowed_ips_b_to_a", "10.0.34.0/24", 200, "allowed_ips_b_to_a peer network"),
        ("allowed_ips_b_to_a", "192.168.1.0/24", 200, "allowed_ips_b_to_a local network"),
        ("allowed_ips_b_to_a", "not-a-subnet", 400, "allowed_ips_b_to_a validation error"),

        # Persistent keepalive variations
        ("persistent_keepalive", {"enabled": True, "value": "25"}, 200, "persistent keepalive 25 seconds"),
        ("persistent_keepalive", {"enabled": True, "value": "30"}, 200, "persistent keepalive 30 seconds"),
        ("persistent_keepalive", {"enabled": True, "value": "60"}, 200, "persistent keepalive 60 seconds"),
        ("persistent_keepalive", {"enabled": False, "value": ""}, 200, "persistent keepalive disabled"),
        ("persistent_keepalive", {"enabled": True, "value": ""}, 400, "persistent keepalive validation error"),
    ],
)
def test_patch_connection_field_changes(setup_wg_quickrs_agent, field_name, field_value, expected_status, test_description):
    """Parameterized test for all connection field changes."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    # Setup: Create a test connection
    peer1_id = "71c565c3-e5c7-45b6-9f21-3d26c9b07d06"
    peer2_id = "349950ac-671f-4ba4-825e-778ebdf79d01"
    connection_id = f"{peer1_id}*{peer2_id}"

    peer_data = get_test_peer_data()
    connection_data = get_test_connection_data()

    setup_change_sum = {
        "added_peers": {
            peer1_id: {**peer_data, "address": "10.0.34.150"},
            peer2_id: {**peer_data, "address": "10.0.34.151"}
        },
        "added_connections": {
            connection_id: connection_data
        }
    }

    setup_response = requests.patch(f"{base_url}/api/network/config", json=setup_change_sum)
    assert setup_response.status_code == 200

    # Test the field change
    changed_fields = {field_name: field_value}

    change_sum = {
        "changed_fields": {
            "connections": {
                connection_id: changed_fields
            }
        }
    }

    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == expected_status

    # Cleanup
    cleanup_change_sum = {
        "removed_peers": [peer1_id, peer2_id],
        "removed_connections": [connection_id]
    }
    requests.patch(f"{base_url}/api/network/config", json=cleanup_change_sum)


@pytest.mark.parametrize(
    "peer_data_variant,expected_status,test_description",
    [
        # Different peer kind
        ({"kind": "desktop"}, 200, "add desktop peer"),
        ({"kind": ""}, 200, "keep it empty"),

        # Different DNS configurations
        ({"dns": {"enabled": True, "value": "8.8.8.8"}}, 200, "add peer with Google DNS"),
        ({"dns": {"enabled": True, "value": "1.1.1.1"}}, 200, "add peer with Cloudflare DNS"),
        ({"dns": {"enabled": False, "value": ""}}, 200, "add peer with DNS disabled"),
        ({"dns": {"enabled": True, "value": ""}}, 400, "DNS validation error"),

        # Different MTU configurations
        ({"mtu": {"enabled": True, "value": "1420"}}, 200, "add peer with MTU 1420"),
        ({"mtu": {"enabled": True, "value": "1280"}}, 200, "add peer with MTU 1280"),
        ({"mtu": {"enabled": False, "value": ""}}, 200, "add peer with MTU disabled"),
        ({"mtu": {"enabled": True, "value": ""}}, 400, "MTU validation error"),

        # Different endpoint configurations
        ({"endpoint": {"enabled": True, "value": "192.168.1.100:51820"}}, 200, "add peer with endpoint"),
        ({"endpoint": {"enabled": False, "value": ""}}, 200, "add peer without endpoint"),
        ({"endpoint": {"enabled": True, "value": ""}}, 400, "endpoint validation error"),

        # Different icon configurations
        ({"icon": {"enabled": True, "value": "custom-icon"}}, 200, "add peer with custom icon"),
        ({"icon": {"enabled": False, "value": ""}}, 200, "add peer without icon"),
        ({"icon": {"enabled": True, "value": ""}}, 400, "icon validation error"),

        # With scripts
        ({"scripts": {
            "pre_up": [{"enabled": True, "value": "echo 'starting';"}],
            "post_up": [{"enabled": True, "value": "echo 'started';"}],
            "pre_down": [{"enabled": True, "value": "echo 'stopping';"}],
            "post_down": [{"enabled": True, "value": "echo 'stopped';"}]
        }}, 200, "add peer with all scripts"),
        ({"scripts": {
            "pre_up": [{"enabled": True, "value": "echo 'starting';"}],
            "post_up": [{"enabled": True, "value": "echo 'started';"}],
            "pre_down": [{"enabled": True, "value": "echo 'stopping';"}],
            "post_down": [{"enabled": True, "value": "echo 'stopped'"}]
        }}, 400, "scripts validation error"),
    ],
)
def test_add_peer_variants(setup_wg_quickrs_agent, peer_data_variant, expected_status, test_description):
    """Parameterized test for adding peers with different configurations."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

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

    # Cleanup
    cleanup_change_sum = {
        "removed_peers": [peer_id]
    }
    requests.patch(f"{base_url}/api/network/config", json=cleanup_change_sum)


@pytest.mark.parametrize(
    "connection_data_variant,expected_status,test_description",
    [
        # Different enabled states
        ({"enabled": True}, 200, "add enabled connection"),
        ({"enabled": False}, 200, "add disabled connection"),

        # Different allowed IPs configurations
        ({"allowed_ips_a_to_b": "0.0.0.0/0", "allowed_ips_b_to_a": "0.0.0.0/0"}, 200, "connection with full routing"),
        ({"allowed_ips_a_to_b": "192.168.1.0/24", "allowed_ips_b_to_a": "10.0.34.0/24"}, 200, "connection with limited routing"),
        ({"allowed_ips_a_to_b": "172.16.0.0/16", "allowed_ips_b_to_a": "172.16.0.0/16"}, 200, "connection with private network routing"),
        ({"allowed_ips_a_to_b": "not-a-subnet", "allowed_ips_b_to_a": "172.16.0.0/16"}, 400, "allowed_ips_a_to_b validation error"),
        ({"allowed_ips_a_to_b": "172.16.0.0/16", "allowed_ips_b_to_a": "not-a-subnet"}, 400, "allowed_ips_b_to_a validation error"),

        # Different persistent keepalive configurations
        ({"persistent_keepalive": {"enabled": True, "value": "25"}}, 200, "connection with 25s keepalive"),
        ({"persistent_keepalive": {"enabled": True, "value": "30"}}, 200, "connection with 30s keepalive"),
        ({"persistent_keepalive": {"enabled": True, "value": "60"}}, 200, "connection with 60s keepalive"),
        ({"persistent_keepalive": {"enabled": False, "value": ""}}, 200, "connection without keepalive"),
        ({"persistent_keepalive": {"enabled": True, "value": ""}}, 400, "persistent_keepalive validation error"),

        # Different pre-shared key configurations
        ({"pre_shared_key": ""}, 400, "connection without pre-shared key"),
        ({"pre_shared_key": "QjF2m3eEcOuBjVqE1K5yB6z9Tf1Hk8qW2aXvNc5uE0o="}, 200, "connection with pre-shared key"),
    ],
)
def test_add_connection_variants(setup_wg_quickrs_agent, connection_data_variant, expected_status, test_description):
    """Parameterized test for adding connections with different configurations."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

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

    # Cleanup
    cleanup_change_sum = {
        "removed_peers": [peer1_id, peer2_id],
        "removed_connections": [connection_id]
    }
    requests.patch(f"{base_url}/api/network/config", json=cleanup_change_sum)
