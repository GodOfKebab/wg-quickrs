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


import pytest
import requests


@pytest.mark.parametrize(
    "changed_fields",
    [
        # single field
        {"name": "updated-host-name"},
        # nested field
        {
            "scripts": {
                "pre_up": [
                    {
                        "enabled": True,
                        "value": "echo 'pre up script';"
                    }
                ],
            }
        },
        # multiple fields
        {
            "name": "multi-field-test",
            "dns": {
                "enabled": True,
                "value": "8.8.8.8"
            },
            "mtu": {
                "enabled": True,
                "value": "1420"
            }
        }
    ],
)
def test_patch_peer_config(setup_wg_quickrs_agent, changed_fields):
    """Test patching peer configuration with various field changes."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    this_peer_id = get_this_peer_id(base_url)

    change_sum = {
        "changed_fields": {
            "peers": {
                this_peer_id: changed_fields
            }
        }
    }

    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "ok"


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


def test_add_peer(setup_wg_quickrs_agent):
    """Test adding a new peer."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    peer_id = "test-peer-12345"
    peer_data = get_test_peer_data()
    
    change_sum = {
        "added_peers": {
            peer_id: peer_data
        }
    }
    
    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "ok"


def test_remove_peer(setup_wg_quickrs_agent):
    """Test removing a peer."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    # First, add a peer
    peer_id = "test-peer-to-remove"
    peer_data = get_test_peer_data()
    
    add_change_sum = {
        "added_peers": {
            peer_id: peer_data
        }
    }
    
    response = requests.patch(f"{base_url}/api/network/config", json=add_change_sum)
    assert response.status_code == 200
    
    # Now remove the peer
    remove_change_sum = {
        "removed_peers": [peer_id]
    }
    
    response = requests.patch(f"{base_url}/api/network/config", json=remove_change_sum)
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "ok"


def test_add_connection(setup_wg_quickrs_agent):
    """Test adding a new connection."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    # First, add two peers
    peer1_id = "peer-1"
    peer2_id = "peer-2"
    peer_data = get_test_peer_data()
    
    # Add first peer
    peer1_data = peer_data.copy()
    peer1_data["address"] = "10.0.34.101"
    
    add_peers_change_sum = {
        "added_peers": {
            peer1_id: peer1_data,
            peer2_id: {**peer_data, "address": "10.0.34.102"}
        }
    }
    
    response = requests.patch(f"{base_url}/api/network/config", json=add_peers_change_sum)
    assert response.status_code == 200
    
    # Now add a connection between them
    connection_id = f"{peer1_id}-{peer2_id}"
    connection_data = get_test_connection_data()
    
    add_connection_change_sum = {
        "added_connections": {
            connection_id: connection_data
        }
    }
    
    response = requests.patch(f"{base_url}/api/network/config", json=add_connection_change_sum)
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "ok"


def test_remove_connection(setup_wg_quickrs_agent):
    """Test removing a connection."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    # First, add peers and connection
    peer1_id = "peer-1"
    peer2_id = "peer-2"
    connection_id = f"{peer1_id}-{peer2_id}"
    
    peer_data = get_test_peer_data()
    connection_data = get_test_connection_data()
    
    # Add peers and connection
    setup_change_sum = {
        "added_peers": {
            peer1_id: {**peer_data, "address": "10.0.34.201"},
            peer2_id: {**peer_data, "address": "10.0.34.202"}
        },
        "added_connections": {
            connection_id: connection_data
        }
    }
    
    response = requests.patch(f"{base_url}/api/network/config", json=setup_change_sum)
    assert response.status_code == 200
    
    # Remove the connection
    remove_change_sum = {
        "removed_connections": [connection_id]
    }
    
    response = requests.patch(f"{base_url}/api/network/config", json=remove_change_sum)
    assert response.status_code == 200
    data = response.json()
    assert data["status"] == "ok"


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


def test_validation_errors(setup_wg_quickrs_agent):
    """Test various validation errors."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    this_peer_id = get_this_peer_id(base_url)

    # Test invalid peer name (empty string)
    change_sum = {
        "changed_fields": {
            "peers": {
                this_peer_id: {
                    "name": ""
                }
            }
        }
    }
    
    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == 400
    data = response.json()
    assert data["status"] == "bad_request"


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

