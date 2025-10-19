from tests.pytest.conftest import setup_wg_quickrs_agent
import requests


def test_version(setup_wg_quickrs_agent):
    """Test getting the version."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    response = requests.get(f"{base_url}/api/version")

    assert response.status_code == 200
    data = response.json()
    assert "version" in data and "build" in data

