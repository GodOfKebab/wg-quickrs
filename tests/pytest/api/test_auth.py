import pytest

from tests.pytest.conftest import setup_wg_quickrs_agent
from tests.pytest.helpers import get_paths, get_token
import requests


def test_api_token(setup_wg_quickrs_agent):
    """Test POST /api/token."""
    base_url = setup_wg_quickrs_agent("test_pwd_single_peer")

    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    # Try getting a token with the wrong password
    response = requests.post(f"{base_url}/api/token",
                             json={ "client_id": "pytest", "password": "..." },
                             verify=wg_quickrs_config_folder / "certs/root/rootCA.crt")
    assert response.status_code == 401
    assert not response.text.startswith("ey")

    # Try getting a token with the correct password
    _ = get_token(base_url)


@pytest.mark.parametrize(
    "path",
    [
        "version",
        "network/summary?only_digest=false",
    ])
def test_api_get_protected(setup_wg_quickrs_agent, path):
    """Test GET /api/<path>. (auth required)"""
    base_url = setup_wg_quickrs_agent("test_pwd_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    # expect failure without a token
    response = requests.get(f"{base_url}/api/{path}",
                            verify=wg_quickrs_config_folder / "certs/root/rootCA.crt")
    assert response.status_code == 401

    # expect success with token
    response = requests.get(f"{base_url}/api/{path}",
                            headers={ "Authorization": f"Bearer {get_token(base_url)}" },
                            verify=wg_quickrs_config_folder / "certs/root/rootCA.crt")
    assert response.status_code == 200


def test_api_patch_protected(setup_wg_quickrs_agent):
    """Test PATCH /api/network/config. (auth required)"""
    base_url = setup_wg_quickrs_agent("test_pwd_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    # expect failure without a token
    response = requests.patch(f"{base_url}/api/network/config",
                            verify=wg_quickrs_config_folder / "certs/root/rootCA.crt")
    assert response.status_code == 401

    # expect success with token
    response = requests.patch(f"{base_url}/api/network/config",
                            headers={ "Authorization": f"Bearer {get_token(base_url)}" },
                            verify=wg_quickrs_config_folder / "certs/root/rootCA.crt")
    assert response.status_code == 400  # bad request expected since the body is empty


@pytest.mark.parametrize(
    "path,expected_status",
    [
        ("wireguard/status", 403),  # bad request expected since vpn is not enabled
        ("network/reserve/address", 200),
    ])
def test_api_post_protected(setup_wg_quickrs_agent, path, expected_status):
    """Test POST /api/wireguard/status. (auth required)"""
    base_url = setup_wg_quickrs_agent("test_pwd_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    # expect failure without a token
    response = requests.post(f"{base_url}/api/{path}",
                              verify=wg_quickrs_config_folder / "certs/root/rootCA.crt")
    assert response.status_code == 401

    # expect success with token
    response = requests.post(f"{base_url}/api/{path}",
                              headers={ "Authorization": f"Bearer {get_token(base_url)}" },
                              verify=wg_quickrs_config_folder / "certs/root/rootCA.crt")
    assert response.status_code == expected_status

def test_get_index_html(setup_wg_quickrs_agent):
    """Test GET /index.html. (no auth required)"""
    base_url = setup_wg_quickrs_agent("test_pwd_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    response = requests.get(f"{base_url}/index.html",
                            verify=wg_quickrs_config_folder / "certs/root/rootCA.crt")
    assert response.status_code == 200


@pytest.mark.parametrize(
    "json_data,expected_status",
    [
        ({"client_id": "test"}, 400),  # missing password field
        ({"password": "test"}, 400),  # missing client_id field
        ({}, 400),  # empty object
    ],
)
def test_api_token_missing_fields(setup_wg_quickrs_agent, json_data, expected_status):
    """Test POST /api/token with missing or invalid fields."""
    base_url = setup_wg_quickrs_agent("test_pwd_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    response = requests.post(f"{base_url}/api/token",
                             json=json_data,
                             verify=wg_quickrs_config_folder / "certs/root/rootCA.crt")
    assert response.status_code == expected_status


def test_api_token_invalid_json(setup_wg_quickrs_agent):
    """Test POST /api/token with invalid JSON."""
    base_url = setup_wg_quickrs_agent("test_pwd_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    response = requests.post(f"{base_url}/api/token",
                             data="invalid json",
                             verify=wg_quickrs_config_folder / "certs/root/rootCA.crt")
    assert response.status_code == 400


@pytest.mark.parametrize(
    "auth_header,expected_status",
    [
        ("", 401),  # empty auth header
        ("Bearer", 401),  # malformed bearer token (no token)
        ("Bearer invalid-token", 401),  # invalid token format
        ("Basic dGVzdDp0ZXN0", 401),  # wrong auth type (Basic instead of Bearer)
        ("bearer valid-looking-token-xyz", 401),  # lowercase bearer
    ],
)
def test_api_protected_malformed_auth(setup_wg_quickrs_agent, auth_header, expected_status):
    """Test protected API endpoints with malformed authorization headers."""
    base_url = setup_wg_quickrs_agent("test_pwd_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    headers = {}
    if auth_header:
        headers["Authorization"] = auth_header

    response = requests.get(f"{base_url}/api/network/summary?only_digest=false",
                            headers=headers,
                            verify=wg_quickrs_config_folder / "certs/root/rootCA.crt")
    assert response.status_code == expected_status


def test_api_auth_endpoint_without_password(setup_wg_quickrs_agent):
    """Test that non-auth endpoints work without authentication when no password is set."""
    base_url = setup_wg_quickrs_agent("test_pwd_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    # Get index.html without auth
    response = requests.get(f"{base_url}/",
                            verify=wg_quickrs_config_folder / "certs/root/rootCA.crt")
    assert response.status_code == 200

