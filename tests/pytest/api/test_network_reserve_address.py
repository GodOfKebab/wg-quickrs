from tests.pytest.conftest import setup_wg_quickrs_agent
from tests.pytest.helpers import get_test_peer_data, get_paths
import requests
from ruamel.yaml import YAML
yaml = YAML()
yaml.preserve_quotes = True


def test_network_reserve_address_incremental_no_repeats(setup_wg_quickrs_agent):
    """Test that reservations are given out incrementally with no repeating addresses."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    # Request reservations for 10.0.34.2-254
    for i in range(253):
        response = requests.post(f"{base_url}/api/network/reserve/address")
        assert response.status_code == 200
        
        data = response.json()
        assert data["address"] == f"10.0.34.{i+2}"

    # The next one should fail
    response = requests.post(f"{base_url}/api/network/reserve/address")
    assert response.status_code == 409


def test_add_peer_with_reserved_address(setup_wg_quickrs_agent):
    """Test adding a new peer with a reserved address."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    response = requests.post(f"{base_url}/api/network/reserve/address")
    assert response.status_code == 200

    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.load(stream)

    assert response.json()["address"] in old_conf["network"]["reservations"]

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

    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.load(stream)
    assert reserved_peer_id in new_conf["network"]["peers"]

def test_change_peer_address_with_conflicting_reserved_address(setup_wg_quickrs_agent):
    base_url = setup_wg_quickrs_agent("no_auth_multi_peer")

    response = requests.post(f"{base_url}/api/network/reserve/address")
    assert response.status_code == 200

    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.load(stream)

    assert response.json()["address"] in old_conf["network"]["reservations"]

    other_peer_id = "6e9a8440-f884-4b54-bfe7-b982f15e40fd"

    change_sum = {
        "changed_fields": {
            "peers": {
                other_peer_id: {
                    "address": response.json()["address"]
                }
            }
        }
    }

    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == 400
    assert "address is already reserved for another peer" in response.json()["message"]


def test_add_peer_with_unreserved_address(setup_wg_quickrs_agent):
    """Test adding a peer with an unreserved address (should succeed)."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    peer_id = "b2c11ade-dd1a-4f5a-a6f9-3b6c6d10f417"
    peer_data = get_test_peer_data()
    peer_data["address"] = "10.0.34.200"  # unreserved address

    change_sum = {
        "added_peers": {
            peer_id: peer_data
        }
    }

    response = requests.patch(f"{base_url}/api/network/config", json=change_sum)
    assert response.status_code == 200

