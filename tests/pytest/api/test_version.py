from tests.pytest.conftest import setup_wg_quickrs_agent
from tests.pytest.helpers import make_get_request_with_retries


def test_version(setup_wg_quickrs_agent):
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    response = make_get_request_with_retries(f"{base_url}/version")

    assert response.status_code == 200
    data = response.json()
    assert "version" in data and "build" in data

