from tests.pytest.conftest import setup_wg_quickrs_agent
import requests


def test_get_network_lease_id_address_incremental_no_repeats(setup_wg_quickrs_agent):
    """Test that leases are given out incrementally with no repeating addresses."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")

    # Request leases for 10.0.34.2-254
    for i in range(253):
        response = requests.get(f"{base_url}/api/network/lease/address")
        assert response.status_code == 200
        
        data = response.json()
        assert data["address"] == f"10.0.34.{i+2}"

    # The next one should fail
    response = requests.get(f"{base_url}/api/network/lease/address")
    assert response.status_code == 500

