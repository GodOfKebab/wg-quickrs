from tests.pytest.conftest import setup_wg_quickrs_agent
import requests
import base64


def is_valid_wg_key(s: str) -> bool:
    try:
        decoded = base64.b64decode(s, validate=True)
        return len(decoded) == 32
    except Exception:
        return False


def test_public_private_keys(setup_wg_quickrs_agent):
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    response = requests.get(f"{base_url}/api/wireguard/public-private-keys")

    assert response.status_code == 200
    data = response.json()
    assert "public_key" in data and "private_key" in data
    assert is_valid_wg_key(data["public_key"]) and is_valid_wg_key(data["private_key"])


def test_preshared_keys(setup_wg_quickrs_agent):
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    response = requests.get(f"{base_url}/api/wireguard/pre-shared-key")

    assert response.status_code == 200
    data = response.json()
    assert "pre_shared_key" in data
    assert is_valid_wg_key(data["pre_shared_key"])

