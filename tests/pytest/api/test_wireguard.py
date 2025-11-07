from tests.pytest.conftest import setup_wg_quickrs_agent
import requests
import time
import pytest


def test_wireguard_status_bad(setup_wg_quickrs_agent):
    """Test setting wireguard status to an unknown value."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    response = requests.post(f"{base_url}/api/wireguard/status", json={"status": "unknown"})  # unknown status
    assert response.status_code == 400


@pytest.mark.parametrize(
    "json_data,expected_status",
    [
        ({"status": "unknown"}, 400),  # unknown status
        ({"status": ""}, 400),  # empty status
        ({"status": "UP"}, 400),  # uppercase (case-sensitive)
        ({"status": "Down"}, 400),  # mixed case
        ({}, 400),  # missing status field
        ({"invalid_field": "up"}, 400),  # wrong field name
    ],
)
def test_wireguard_status_invalid_requests(setup_wg_quickrs_agent, json_data, expected_status):
    """Test POST /api/wireguard/status with various invalid payloads."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    response = requests.post(f"{base_url}/api/wireguard/status", json=json_data)
    assert response.status_code == expected_status


def test_wireguard_status_up_down(setup_wg_quickrs_agent):
    """Test setting wireguard status up then down."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer", use_sudo=True)
    response = requests.post(f"{base_url}/api/wireguard/status", json={"status": "up"})
    assert response.status_code == 200

    response = requests.get(f"{base_url}/api/network/summary?only_digest=true")
    assert response.json()["status"] == "up"

    response = requests.post(f"{base_url}/api/wireguard/status", json={"status": "down"})
    assert response.status_code == 200

    response = requests.get(f"{base_url}/api/network/summary?only_digest=true")
    assert response.json()["status"] == "down"


def test_wireguard_status_idempotent_up(setup_wg_quickrs_agent):
    """Test that sending 'up' multiple times is idempotent."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer", use_sudo=True)

    # Send up the up command twice
    response1 = requests.post(f"{base_url}/api/wireguard/status", json={"status": "up"})
    assert response1.status_code == 200

    time.sleep(1)

    response2 = requests.post(f"{base_url}/api/wireguard/status", json={"status": "up"})
    assert response2.status_code == 200


def test_wireguard_status_idempotent_down(setup_wg_quickrs_agent):
    """Test that sending 'down' multiple times is idempotent."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer", use_sudo=True)

    # Ensure it's down first
    response1 = requests.post(f"{base_url}/api/wireguard/status", json={"status": "down"})
    assert response1.status_code == 200

    time.sleep(1)

    # Send the down command again
    response2 = requests.post(f"{base_url}/api/wireguard/status", json={"status": "down"})
    assert response2.status_code == 200

