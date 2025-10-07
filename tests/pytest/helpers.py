import requests
import time
import pathlib


def get_paths():
    pytest_folder = pathlib.Path(__file__).parent.resolve()
    wg_quickrs_config_folder = pytest_folder / ".wg-quickrs-pytest"
    wg_quickrs_config_file = wg_quickrs_config_folder / "conf.yml"
    return pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file


def get_wg_quickrs_command():
    pytest_folder, wg_quickrs_config_folder, _ = get_paths()
    return [
        pytest_folder.parent.parent.resolve() / "src/target/release/wg-quickrs",
        '--wg-quickrs-config-folder',
        wg_quickrs_config_folder
    ]


def make_get_request_with_retries(url, max_retries=3, delay_seconds=1):
    response = None
    for i in range(max_retries):
        try:
            response = requests.get(url)
        except (requests.exceptions.ConnectionError, ConnectionRefusedError):
            time.sleep(delay_seconds)
    if response is None:
        raise Exception(f"Unable to connect to wg-quickrs agent at {url}")
    return response


def make_post_request_with_retries(url, json=None, verify=None, max_retries=3, delay_seconds=1):
    response = None
    for i in range(max_retries):
        try:
            response = requests.post(url, json=json, verify=verify)
        except (requests.exceptions.ConnectionError, ConnectionRefusedError) as e:
            print(e)
            time.sleep(delay_seconds)
    if response is None:
        raise Exception(f"Unable to connect to wg-quickrs agent at {url}")
    return response

