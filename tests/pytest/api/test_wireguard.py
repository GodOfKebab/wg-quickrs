from tests.pytest.conftest import setup_wg_quickrs_agent
import requests
import time


def test_wireguard_status_bad(setup_wg_quickrs_agent):
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    response = requests.post(f"{base_url}/api/wireguard/status", json={"status": 0})  # unknown status
    assert response.status_code == 400


def test_wireguard_status_up_down(setup_wg_quickrs_agent):
    base_url = setup_wg_quickrs_agent("no_auth_single_peer")
    response = requests.post(f"{base_url}/api/wireguard/status", json={"status": 2})  # up status
    assert response.status_code == 200

    is_up = False
    start = time.time()
    while not is_up and time.time() - start < 10:
        response = requests.get(f"{base_url}/api/network/summary?only_digest=true")
        assert response.status_code == 200

        status = response.json()["status"]
        is_up = status == 2  # up status
        time.sleep(1)
    assert is_up == True

    response = requests.post(f"{base_url}/api/wireguard/status", json={"status": 1})  # down status
    assert response.status_code == 200

    is_down = False
    start = time.time()
    while is_up and time.time() - start < 10:
        response = requests.get(f"{base_url}/api/network/summary?only_digest=true")
        assert response.status_code == 200

        status = response.json()["status"]
        is_down = status == 1  # down status
        time.sleep(1)
    assert is_down == True

