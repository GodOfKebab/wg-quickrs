from tests.pytest.conftest import setup_wg_quickrs_agent
from tests.pytest.helpers import get_paths
import requests


def test_api_token(setup_wg_quickrs_agent):
    base_url = setup_wg_quickrs_agent("test_pwd_single_peer")

    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    # Try getting a token with the correct password
    response = requests.post(f"{base_url}/api/token",
                             json={ "client_id": "pytest", "password": "test" },
                             verify=wg_quickrs_config_folder / "certs/root/rootCA.crt")
    assert response.status_code == 200
    assert response.text.startswith("ey")

    # Try getting a token with the wrong password
    response = requests.post(f"{base_url}/api/token",
                             json={ "client_id": "pytest", "password": "..." },
                             verify=wg_quickrs_config_folder / "certs/root/rootCA.crt")
    assert response.status_code == 401
    assert not response.text.startswith("ey")
