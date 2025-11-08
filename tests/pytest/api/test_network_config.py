from tests.pytest.conftest import setup_wg_quickrs_agent
from tests.pytest.helpers import get_this_peer_id, get_test_peer_data, get_test_connection_data, get_paths
import requests
import uuid
from ruamel.yaml import YAML
yaml = YAML()
yaml.preserve_quotes = True


def test_patch_peer_not_found(setup_wg_quickrs_agent):
    """Test changing a peer that doesn't exist."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.load(stream)

    change_sum = {
        "changed_fields": {
            "peers": {
                uuid.uuid4().hex: {  # non-existent-peer-id
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


def test_add_bad_connection(setup_wg_quickrs_agent):
    """Test adding a bad connection."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.load(stream)

    bad_connection_id = "non-a-connection-id"
    bad_setup_change_sum = {
        "added_connections": {
            bad_connection_id: get_test_connection_data()
        }
    }
    setup_response = requests.patch(f"{base_url}/api/network/config", json=bad_setup_change_sum)
    assert setup_response.status_code == 400
    assert "uuid*uuid" in setup_response.json()["message"]

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
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.load(stream)

    response = requests.patch(f"{base_url}/api/network/config", data="invalid json")
    assert response.status_code == 400
    data = response.json()
    assert "invalid JSON" in data["message"]


def test_empty_change_sum(setup_wg_quickrs_agent):
    """Test sending an empty change sum (nothing to update)."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.load(stream)

    empty_change_sum = {}

    response = requests.patch(f"{base_url}/api/network/config", json=empty_change_sum)
    assert response.status_code == 400
    data = response.json()
    assert data["status"] == "bad_request"
    assert "nothing to update" in data["message"]


def test_connection_not_found(setup_wg_quickrs_agent):
    """Test modifying a connection that doesn't exist."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.load(stream)

    change_sum = {
        "changed_fields": {
            "connections": {
                f"{uuid.uuid4().hex}*{uuid.uuid4().hex}": {  # non-existent-connection
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


def test_remove_peer(setup_wg_quickrs_agent):
    """Test deleting a peer."""
    base_url = setup_wg_quickrs_agent("no_auth_multi_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    this_peer = "0ed989c6-6dba-4e3c-8034-08adf4262d9e"
    other_peer1 = "6e9a8440-f884-4b54-bfe7-b982f15e40fd"
    other_peer2 = "9541bbb0-a3c0-4b83-8637-96820cae7983"
    other_peer1_this_peer_connection_id = f"{other_peer1}*{this_peer}"

    # Test: Remove a peer
    removed_peers_change_sum = {
        "removed_peers": [other_peer1]
    }
    response = requests.patch(f"{base_url}/api/network/config", json=removed_peers_change_sum)
    assert response.status_code == 200

    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.load(stream)

    assert other_peer1 not in new_conf["network"]["peers"]
    assert this_peer in new_conf["network"]["peers"] and other_peer2 in new_conf["network"]["peers"]
    assert other_peer1_this_peer_connection_id not in new_conf["network"]["connections"]

def test_remove_this_peer(setup_wg_quickrs_agent):
    """Test deleting a peer."""
    base_url = setup_wg_quickrs_agent("no_auth_multi_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    this_peer = "0ed989c6-6dba-4e3c-8034-08adf4262d9e"

    # Test: Remove a peer
    removed_peers_change_sum = {
        "removed_peers": [this_peer]
    }
    response = requests.patch(f"{base_url}/api/network/config", json=removed_peers_change_sum)
    assert response.status_code == 403


def test_remove_connection(setup_wg_quickrs_agent):
    """Test deleting a connection."""
    base_url = setup_wg_quickrs_agent("no_auth_multi_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    this_peer = "0ed989c6-6dba-4e3c-8034-08adf4262d9e"
    other_peer1 = "6e9a8440-f884-4b54-bfe7-b982f15e40fd"
    other_peer2 = "9541bbb0-a3c0-4b83-8637-96820cae7983"
    other_peer1_this_peer_connection_id = f"{other_peer1}*{this_peer}"

    # Test: Remove a peer
    removed_connections_change_sum = {
        "removed_connections": [other_peer1_this_peer_connection_id]
    }
    response = requests.patch(f"{base_url}/api/network/config", json=removed_connections_change_sum)
    assert response.status_code == 200

    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.load(stream)

    assert this_peer in new_conf["network"]["peers"] and other_peer1 in new_conf["network"]["peers"] and other_peer2 in new_conf["network"]["peers"]
    assert other_peer1_this_peer_connection_id not in new_conf["network"]["connections"]


def test_change_peer_address_with_conflicting_address(setup_wg_quickrs_agent):
    base_url = setup_wg_quickrs_agent("no_auth_multi_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.load(stream)

    other_peer_id = "6e9a8440-f884-4b54-bfe7-b982f15e40fd"

    change_sum = {
        "changed_fields": {
            "peers": {
                other_peer_id: {
                    "address": "10.0.34.1"
                }
            }
        }
    }

    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == 400
    assert "taken by" in response.json()["message"]
    assert "wg-quickrs-host" in response.json()["message"]


def test_patch_with_malformed_json(setup_wg_quickrs_agent):
    """Test PATCH with various malformed JSON payloads."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    # Test with completely invalid JSON
    response = requests.patch(
        f"{base_url}/api/network/config",
        data="this is not json",
        headers={"Content-Type": "application/json"}
    )
    assert response.status_code == 400


def test_add_peer_with_duplicate_address(setup_wg_quickrs_agent):
    """Test adding a peer with an address that's already in use."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    peer_id = "b2c11ade-dd1a-4f5a-a6f9-3b6c6d10f417"
    peer_data = get_test_peer_data()
    peer_data["address"] = "10.0.34.1"  # This address is already used by the existing peer

    change_sum = {
        "added_peers": {
            peer_id: peer_data
        }
    }

    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == 400
    assert "address" in response.json()["message"].lower()


def test_add_peer_with_duplicate_id(setup_wg_quickrs_agent):
    """Test adding a peer with an ID that already exists."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    this_peer_id = get_this_peer_id(base_url)
    peer_data = get_test_peer_data()
    peer_data["address"] = "10.0.34.100"

    change_sum = {
        "added_peers": {
            this_peer_id: peer_data  # Using existing peer ID
        }
    }

    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == 403
    assert "already exists" in response.json()["message"]


def test_change_nonexistent_field(setup_wg_quickrs_agent):
    """Test changing a field structure that doesn't match expected format."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    this_peer_id = get_this_peer_id(base_url)

    change_sum = {
        "changed_fields": {
            "invalid_section": {
                this_peer_id: {
                    "name": "test"
                }
            }
        }
    }

    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == 400


def test_add_connection_between_same_peer(setup_wg_quickrs_agent):
    """Test adding a connection where both peers are the same."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    this_peer_id = get_this_peer_id(base_url)
    connection_id = f"{this_peer_id}*{this_peer_id}"

    change_sum = {
        "added_connections": {
            connection_id: get_test_connection_data()
        }
    }

    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == 403
    assert "loopback" in response.json()["message"]


def test_add_duplicate_connection(setup_wg_quickrs_agent):
    """Test adding a connection that already exists."""
    base_url = setup_wg_quickrs_agent("no_auth_multi_peer")

    this_peer = "0ed989c6-6dba-4e3c-8034-08adf4262d9e"
    other_peer1 = "6e9a8440-f884-4b54-bfe7-b982f15e40fd"
    existing_connection_id = f"{other_peer1}*{this_peer}"

    change_sum = {
        "added_connections": {
            existing_connection_id: get_test_connection_data()
        }
    }

    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == 403
    assert "already exists" in response.json()["message"].lower()

