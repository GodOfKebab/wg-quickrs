from tests.pytest.helpers import get_paths, get_wg_quickrs_command
from subprocess import Popen
import shutil
import pytest
import yaml


@pytest.fixture(scope="function")
def setup_wg_quickrs_folder(request):
    """
    Prepare a wg-quickrs config folder from the test data.
    Usage in a test:
        def test_something(setup_wg_quickrs_folder):
            pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = setup_wg_quickrs_folder("no_auth_single_peer")
            ...
    """
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    shutil.rmtree(wg_quickrs_config_folder)

    def _setup(which_conf: str):
        shutil.copytree(
            pytest_folder / f"data/{which_conf}",
            wg_quickrs_config_folder,
            dirs_exist_ok=True
        )
        return pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file

    return _setup


@pytest.fixture(scope="function")
def setup_wg_quickrs_agent(request, setup_wg_quickrs_folder):
    """
    Prepare a wg-quickrs agent from the test data.
    Usage in a test:
        def test_something(setup_wg_quickrs_agent):
            base_url = setup_wg_quickrs_agent("no_auth_single_peer")
            ...
    """
    def _setup(which_conf: str):
        pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = setup_wg_quickrs_folder(which_conf)

        # Load config to extract agent address
        with open(wg_quickrs_config_file) as stream:
            conf = yaml.safe_load(stream)

        # prefer https over http
        if conf['agent']['web']['https']['enabled']:
            base_url = f"https://{conf['agent']['web']['address']}:{conf['agent']['web']['https']['port']}"
        elif conf['agent']['web']['http']['enabled']:
            base_url = f"http://{conf['agent']['web']['address']}:{conf['agent']['web']['http']['port']}"
        else:
            base_url = None

        # Start agent
        agent = Popen(get_wg_quickrs_command() + ['agent', 'run'])

        # terminate agent when the test is over
        def fin():
            agent.terminate()
            agent.wait()

        request.addfinalizer(fin)

        return base_url

    return _setup

