from tests.pytest.conftest import setup_wg_quickrs_agent
from tests.pytest.helpers import get_this_peer_id, get_test_peer_data, get_test_connection_data, get_paths
import requests
import yaml


def test_patch_forbidden_endpoint_change(setup_wg_quickrs_agent):
    """Test that changing host peer's endpoint is forbidden."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.safe_load(stream)

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

    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.safe_load(stream)
    assert old_conf == new_conf


def test_patch_peer_not_found(setup_wg_quickrs_agent):
    """Test changing a peer that doesn't exist."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.safe_load(stream)

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

    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.safe_load(stream)
    assert old_conf == new_conf


def test_add_peer_with_leased_address(setup_wg_quickrs_agent):
    """Test adding a new peer with a leased address."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    response = requests.get(f"{base_url}/api/network/lease/address")
    assert response.status_code == 200

    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.safe_load(stream)

    assert response.json()["address"] in old_conf["network"]["leases"]

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

    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.safe_load(stream)
    assert old_conf == new_conf

    another_peer_id = "a1c11ade-dd1a-4f5a-a6f9-3b6c6d10f416"
    change_sum_w_another_peer_id = {
        "added_peers": {
            another_peer_id: peer_data
        }
    }
    response = requests.patch(f"{base_url}/api/network/config", json=change_sum_w_another_peer_id)
    assert response.status_code == 403
    assert "reserved for another" in response.json()["message"]

    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.safe_load(stream)
    assert old_conf == new_conf

    correct_change_sum = {
        "added_peers": {
            reserved_peer_id: peer_data
        }
    }
    response = requests.patch(f"{base_url}/api/network/config", json=correct_change_sum)
    assert response.status_code == 200

    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.safe_load(stream)
    assert reserved_peer_id in new_conf["network"]["peers"]


def test_add_bad_connection(setup_wg_quickrs_agent):
    """Test adding a bad connection."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.safe_load(stream)

    bad_connection_id = "non-a-connection-id"
    bad_setup_change_sum = {
        "added_connections": {
            bad_connection_id: get_test_connection_data()
        }
    }
    setup_response = requests.patch(f"{base_url}/api/network/config", json=bad_setup_change_sum)
    assert setup_response.status_code == 400
    assert "not a valid connection_id" in setup_response.json()["message"]

    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.safe_load(stream)
    assert old_conf == new_conf

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

    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.safe_load(stream)
    assert old_conf == new_conf


def test_invalid_json(setup_wg_quickrs_agent):
    """Test sending invalid JSON."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.safe_load(stream)

    response = requests.patch(f"{base_url}/api/network/config", data="invalid json")
    assert response.status_code == 400
    data = response.json()
    assert "Invalid JSON" in data["error"]

    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.safe_load(stream)
    assert old_conf == new_conf


def test_empty_change_sum(setup_wg_quickrs_agent):
    """Test sending empty change sum (nothing to update)."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.safe_load(stream)

    empty_change_sum = {}

    response = requests.patch(f"{base_url}/api/network/config", json=empty_change_sum)
    assert response.status_code == 400
    data = response.json()
    assert data["status"] == "bad_request"
    assert "nothing to update" in data["message"]

    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.safe_load(stream)
    assert old_conf == new_conf


def test_connection_not_found(setup_wg_quickrs_agent):
    """Test modifying a connection that doesn't exist."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.safe_load(stream)

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

    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.safe_load(stream)
    assert old_conf == new_conf


def test_remove_peer(setup_wg_quickrs_agent):
    """Test deleting an auto-created connection."""
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
        new_conf = yaml.safe_load(stream)

    assert other_peer1 not in new_conf["network"]["peers"]
    assert this_peer in new_conf["network"]["peers"] and other_peer2 in new_conf["network"]["peers"]
    assert other_peer1_this_peer_connection_id not in new_conf["network"]["connections"]


def test_remove_connection(setup_wg_quickrs_agent):
    """Test deleting an auto-created connection."""
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
        new_conf = yaml.safe_load(stream)

    assert this_peer in new_conf["network"]["peers"] and other_peer1 in new_conf["network"]["peers"] and other_peer2 in new_conf["network"]["peers"]
    assert other_peer1_this_peer_connection_id not in new_conf["network"]["connections"]


