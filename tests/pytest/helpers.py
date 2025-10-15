import requests
import time
import pathlib
import socket


def get_paths():
    pytest_folder = pathlib.Path(__file__).parent.resolve()
    wg_quickrs_config_folder = pytest_folder / ".wg-quickrs-pytest"
    wg_quickrs_config_file = wg_quickrs_config_folder / "conf.yml"
    return pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file


def get_wg_quickrs_command():
    pytest_folder, wg_quickrs_config_folder, _ = get_paths()
    return [
        str(pytest_folder.parent.parent.resolve() / "src/target/release/wg-quickrs"),
        '--wg-quickrs-config-folder',
        str(wg_quickrs_config_folder)
    ]


def wait_for_port(host_port, timeout=10.0):
    """Wait until TCP port is open or timeout"""
    start = time.time()
    while time.time() - start < timeout:
        try:
            with socket.create_connection(host_port, timeout=1):
                return True
        except OSError:
            time.sleep(0.1)
    return False

