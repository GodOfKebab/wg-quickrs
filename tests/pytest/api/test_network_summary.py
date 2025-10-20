import requests


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
    assert "identifier" in network
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
