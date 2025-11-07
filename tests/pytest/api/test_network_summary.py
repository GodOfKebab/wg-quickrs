from tests.pytest.conftest import setup_wg_quickrs_agent
import requests
import pytest


def test_get_summary_full(setup_wg_quickrs_agent):
    """Test GET /api/network/summary with only_digest=false."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    response = requests.get( f"{base_url}/api/network/summary?only_digest=false")
    assert response.status_code == 200

    # Verify the full summary structure
    data = response.json()
    assert "network" in data
    assert "telemetry" in data
    assert "digest" in data
    assert "status" in data
    assert "timestamp" in data

    # Verify network structure
    network = data["network"]
    assert "name" in network
    assert "subnet" in network
    assert "this_peer" in network
    assert "peers" in network
    assert "connections" in network
    assert "defaults" in network
    assert "reservations" in network
    assert "updated_at" in network


def test_get_summary_digest_only(setup_wg_quickrs_agent):
    """Test GET /api/network/summary with only_digest=true."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    response = requests.get(f"{base_url}/api/network/summary?only_digest=true")
    assert response.status_code == 200

    # Verify the digest-only structure (no network field)
    data = response.json()
    assert "network" not in data
    assert "telemetry" in data
    assert "digest" in data
    assert "status" in data
    assert "timestamp" in data


def test_get_summary_default_param(setup_wg_quickrs_agent):
    """Test GET /api/network/summary without query params."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    response = requests.get(f"{base_url}/api/network/summary")
    assert response.status_code == 400


@pytest.mark.parametrize(
    "query_param,expected_status",
    [
        ("only_digest=invalid", 400),  # invalid boolean value
        ("only_digest=1", 400),  # invalid boolean format
        ("only_digest=yes", 400),  # invalid boolean format
        ("only_digest=true&invalid_param=true", 200),  # unknown param should be ignored
    ],
)
def test_get_summary_invalid_query_params(setup_wg_quickrs_agent, query_param, expected_status):
    """Test GET /api/network/summary with invalid query parameters."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    response = requests.get(f"{base_url}/api/network/summary?{query_param}")
    assert response.status_code == expected_status


def test_get_summary_with_multi_peer(setup_wg_quickrs_agent):
    """Test GET /api/network/summary with multiple peers."""
    base_url = setup_wg_quickrs_agent("no_auth_multi_peer")
    response = requests.get(f"{base_url}/api/network/summary?only_digest=false")
    assert response.status_code == 200

    # Verify multiple peers exist
    data = response.json()
    assert len(data["network"]["peers"]) > 1
    assert len(data["network"]["connections"]) >= 1
