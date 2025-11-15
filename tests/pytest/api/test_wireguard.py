from tests.pytest.conftest import setup_wg_quickrs_agent
import requests
import time
import pytest

from tests.pytest.helpers import get_paths, get_token


def test_wireguard_status_up_uninitialized(setup_wg_quickrs_agent):
    """Test setting wireguard status up then down."""
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    response = requests.post(f"{base_url}/api/wireguard/status", json={"status": "up"})
    assert response.status_code == 403
    assert "VPN is disabled in configuration" in response.content.decode("utf-8")


def test_wireguard_status_bad(setup_wg_quickrs_agent):
    """Test setting wireguard status to an unknown value."""
    base_url = setup_wg_quickrs_agent("test_pwd_single_peer_w_enabled_vpn", use_sudo=True)
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    for status in ["unknown", "UP", "Down", ""]:
        response = requests.post(f"{base_url}/api/wireguard/status",
                                 headers={ "Authorization": f"Bearer {get_token(base_url)}" },
                                 verify=wg_quickrs_config_folder / "certs/root/rootCA.crt",
                                 json={"status": status})
        assert response.status_code == 400

    for json in [{}, {'invalid_field': 'up'}]:
        response = requests.post(f"{base_url}/api/wireguard/status",
                                 headers={ "Authorization": f"Bearer {get_token(base_url)}" },
                                 verify=wg_quickrs_config_folder / "certs/root/rootCA.crt",
                                 json=json)
        assert response.status_code == 400


def test_wireguard_status_up_down_up(setup_wg_quickrs_agent):
    """Test setting wireguard status up, down, then up."""
    base_url = setup_wg_quickrs_agent("test_pwd_single_peer_w_enabled_vpn", use_sudo=True)

    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    response = requests.post(f"{base_url}/api/wireguard/status",
                             headers={ "Authorization": f"Bearer {get_token(base_url)}" },
                             verify=wg_quickrs_config_folder / "certs/root/rootCA.crt",
                             json={"status": "down"})
    assert response.status_code == 200
    response = requests.get(f"{base_url}/api/network/summary?only_digest=true",
                            headers={ "Authorization": f"Bearer {get_token(base_url)}" },
                            verify=wg_quickrs_config_folder / "certs/root/rootCA.crt")
    assert response.json()["status"] == "down"

    response = requests.post(f"{base_url}/api/wireguard/status",
                             headers={ "Authorization": f"Bearer {get_token(base_url)}" },
                             verify=wg_quickrs_config_folder / "certs/root/rootCA.crt",
                             json={"status": "up"})
    assert response.status_code == 200
    response = requests.get(f"{base_url}/api/network/summary?only_digest=true",
                            headers={ "Authorization": f"Bearer {get_token(base_url)}" },
                            verify=wg_quickrs_config_folder / "certs/root/rootCA.crt")
    assert response.json()["status"] == "up"


def test_wireguard_status_idempotent_up(setup_wg_quickrs_agent):
    """Test that sending 'up' multiple times is idempotent."""
    base_url = setup_wg_quickrs_agent("test_pwd_single_peer_w_enabled_vpn", use_sudo=True)

    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    # Send up the up command twice
    response = requests.post(f"{base_url}/api/wireguard/status",
                             headers={ "Authorization": f"Bearer {get_token(base_url)}" },
                             verify=wg_quickrs_config_folder / "certs/root/rootCA.crt",
                             json={"status": "up"})
    assert response.status_code == 200
    response = requests.get(f"{base_url}/api/network/summary?only_digest=true",
                            headers={ "Authorization": f"Bearer {get_token(base_url)}" },
                            verify=wg_quickrs_config_folder / "certs/root/rootCA.crt")
    assert response.json()["status"] == "up"

    response = requests.post(f"{base_url}/api/wireguard/status",
                             headers={ "Authorization": f"Bearer {get_token(base_url)}" },
                             verify=wg_quickrs_config_folder / "certs/root/rootCA.crt",
                             json={"status": "up"})
    assert response.status_code == 200
    response = requests.get(f"{base_url}/api/network/summary?only_digest=true",
                            headers={ "Authorization": f"Bearer {get_token(base_url)}" },
                            verify=wg_quickrs_config_folder / "certs/root/rootCA.crt")
    assert response.json()["status"] == "up"


def test_wireguard_status_idempotent_down(setup_wg_quickrs_agent):
    """Test that sending 'down' multiple times is idempotent."""
    base_url = setup_wg_quickrs_agent("test_pwd_single_peer_w_enabled_vpn", use_sudo=True)

    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    # Ensure it's down first
    response = requests.post(f"{base_url}/api/wireguard/status",
                             headers={ "Authorization": f"Bearer {get_token(base_url)}" },
                             verify=wg_quickrs_config_folder / "certs/root/rootCA.crt",
                             json={"status": "down"})
    assert response.status_code == 200
    response = requests.get(f"{base_url}/api/network/summary?only_digest=true",
                            headers={ "Authorization": f"Bearer {get_token(base_url)}" },
                            verify=wg_quickrs_config_folder / "certs/root/rootCA.crt")
    assert response.json()["status"] == "down"

    # Send the down command again
    response = requests.post(f"{base_url}/api/wireguard/status",
                             headers={ "Authorization": f"Bearer {get_token(base_url)}" },
                             verify=wg_quickrs_config_folder / "certs/root/rootCA.crt",
                             json={"status": "down"})
    assert response.status_code == 200
    response = requests.get(f"{base_url}/api/network/summary?only_digest=true",
                            headers={ "Authorization": f"Bearer {get_token(base_url)}" },
                            verify=wg_quickrs_config_folder / "certs/root/rootCA.crt")
    assert response.json()["status"] == "down"

