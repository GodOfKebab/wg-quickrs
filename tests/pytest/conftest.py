from tests.pytest.helpers import get_paths, get_wg_quickrs_command, wait_for_tcp_port, wait_for_wireguard
from subprocess import Popen
import os
import sys
import shutil
import pytest
from ruamel.yaml import YAML
yaml = YAML()
yaml.preserve_quotes = True


def setup_certs_folder(address):
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    os.makedirs(pytest_folder / ".certs", exist_ok=True)

    address_cert_path = pytest_folder / f".certs/{address}"
    if not os.path.exists(address_cert_path):
        os.mkdir(address_cert_path)
        tls_cert_generator = Popen(f"wget -qO- https://github.com/GodOfKebab/tls-cert-generator/releases/download/v1.3.0/tls-cert-generator.sh | sh -s -- -f -o {address_cert_path} --country 'XX' --state 'XX' --locality 'XX' --org 'XX' --ou 'XX' --cn 'tls-cert-generator@XX' {address}", shell=True)
        tls_cert_generator.wait()

    shutil.copytree(
        address_cert_path,
        wg_quickrs_config_folder / "certs",
        dirs_exist_ok=True
    )


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
    shutil.rmtree(wg_quickrs_config_folder, ignore_errors=True)

    def _setup(which_conf: str):
        if which_conf is None:
            os.makedirs(wg_quickrs_config_folder, exist_ok=True)
        else:
            shutil.copytree(
                pytest_folder / f"data/{which_conf}",
                wg_quickrs_config_folder,
                dirs_exist_ok=True
            )
            # Load config to extract agent address
            with open(wg_quickrs_config_file) as stream:
                conf = yaml.load(stream)
            # TLS cert generation
            if conf['agent']['web']['https']['enabled']:
                setup_certs_folder(conf['agent']['web']['address'])

            conf['agent']['vpn']['wg'] = shutil.which("wg")
            if sys.platform == "linux":
                conf['agent']['vpn']['wg_userspace']['enabled'] = False
            else:
                conf['agent']['vpn']['wg_userspace']['enabled'] = True
                conf['agent']['vpn']['wg_userspace']['binary'] = shutil.which("wireguard-go")
            with open(wg_quickrs_config_file, 'w') as stream:
                yaml.dump(conf, stream)

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
    def _setup(which_conf: str, use_sudo=False):
        pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = setup_wg_quickrs_folder(which_conf)

        # Load config to extract agent address
        with open(wg_quickrs_config_file) as stream:
            conf = yaml.load(stream)

        # prefer https over http
        if conf['agent']['web']['https']['enabled']:
            base_url = f"https://{conf['agent']['web']['address']}:{conf['agent']['web']['https']['port']}"
            host_port = (conf['agent']['web']['address'], conf['agent']['web']['https']['port'])
        elif conf['agent']['web']['http']['enabled']:
            base_url = f"http://{conf['agent']['web']['address']}:{conf['agent']['web']['http']['port']}"
            host_port = (conf['agent']['web']['address'], conf['agent']['web']['http']['port'])
        else:
            base_url = None
            host_port = None

        # Start agent
        agent = Popen(get_wg_quickrs_command(use_sudo) + ['agent', 'run'])

        # Wait for http(s) to start listening
        if host_port:
            if not wait_for_tcp_port(host_port, timeout=10):
                agent.terminate()
                raise RuntimeError("Agent failed to start http(s) within timeout")

        # Wait for the wireguard tunnel to start listening
        if conf['agent']['vpn']['enabled']:
            if not wait_for_wireguard(base_url, conf['agent']['web']['https']['enabled'], timeout=10):
                agent.terminate()
                raise RuntimeError("Agent failed to start wireguard tunnel within timeout")


        # terminate agent when the test is over
        def fin():
            agent.terminate()
            agent.wait()

        request.addfinalizer(fin)

        return base_url

    return _setup

