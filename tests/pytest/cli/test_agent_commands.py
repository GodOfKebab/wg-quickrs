import os
import shutil

from tests.pytest.conftest import setup_wg_quickrs_folder
from tests.pytest.helpers import (
    get_wg_quickrs_command,
    get_paths,
    get_available_firewall_utilities,
    get_available_network_interfaces
)
import yaml
import subprocess
from deepdiff import DeepDiff
import pytest


def deep_get(d, keys):
    for k in keys:
        if isinstance(d, dict) and k in d:
            d = d[k]
        else:
            return None
    return d


def run_and_check_success(cmd, field, field_val, success):
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.safe_load(stream)

    result = subprocess.run(
        get_wg_quickrs_command() + ['agent'] + cmd,
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert (result.returncode == 0) == success
    if not success:
        return

    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.safe_load(stream)

    # check diff
    diff = DeepDiff(old_conf, new_conf, ignore_order=True)
    expected_path = "root" + "".join(f"['{k}']" for k in field)
    assert diff["values_changed"][expected_path]["old_value"] == deep_get(old_conf, field)
    if cmd[0] != "reset-web-password":
        assert diff["values_changed"][expected_path]["new_value"] == field_val


@pytest.mark.parametrize(
    "command, path, value, success",
    [
        ("set-web-address", ('agent', 'web', 'address'), "192.168.10.10", True),
        ("set-web-address", ('agent', 'web', 'address'), 'not-an-address', False),
        ("set-web-http-port", ('agent', 'web', 'http', 'port'), 80, True),
        ("set-web-http-port", ('agent', 'web', 'http', 'port'), "not-a-port", False),
        ("set-web-https-port", ('agent', 'web', 'https', 'port'), 443, True),
        ("set-web-https-port", ('agent', 'web', 'https', 'port'), "not-a-port", False),
        # since the tls cert generation is not tested here, we skip the successful https test case
        ("set-vpn-port", ('agent', 'vpn', 'port'), 51821, True),
        ("set-vpn-port", ('agent', 'vpn', 'port'), "not-a-port", False),
        # skip the successful firewall setting test case because the gateway and utility names will differ in different machines
    ],
)
def test_agent_set_simple(setup_wg_quickrs_folder, command, path, value, success):
    setup_wg_quickrs_folder("no_auth_single_peer")
    run_and_check_success([command, str(value)], path, value, success)


def test_agent_set_https_cert_key(setup_wg_quickrs_agent):
    setup_wg_quickrs_agent("test_pwd_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    os.makedirs(wg_quickrs_config_folder / 'certs/servers/localhost')
    shutil.copyfile(wg_quickrs_config_folder / 'certs/servers/127.0.0.1/cert.pem', wg_quickrs_config_folder / 'certs/servers/localhost/cert.pem')
    shutil.copyfile(wg_quickrs_config_folder / 'certs/servers/127.0.0.1/key.pem', wg_quickrs_config_folder / 'certs/servers/localhost/key.pem')

    for command, path, value, success in [
        ("set-web-https-tls-cert", ('agent', 'web', 'https', 'tls_cert'), 'certs/servers/localhost/cert.pem', True),
        ("set-web-https-tls-cert", ('agent', 'web', 'https', 'tls_cert'), "not-a-cert-path", False),
        ("set-web-https-tls-key", ('agent', 'web', 'https', 'tls_key'), 'certs/servers/localhost/key.pem', True),
        ("set-web-https-tls-key", ('agent', 'web', 'https', 'tls_key'), "not-a-key-path", False),
    ]:
        run_and_check_success([command, str(value)], path, value, success)


def test_agent_reset_web_password(setup_wg_quickrs_folder):
    setup_wg_quickrs_folder("test_pwd_single_peer")

    for command, path, value, success in [
        ("reset-web-password", ('agent', 'web', 'password', 'hash'), "new-test-pwd", True),
    ]:
        run_and_check_success([command, "--password", str(value)], path, value, success)


def test_agent_toggle_simple(setup_wg_quickrs_folder):
    setup_wg_quickrs_folder("no_auth_single_peer")
    for command, path, value, success in [
        ("disable-web-http", ('agent', 'web', 'http', 'enabled'), False, True),
        ("enable-web-http", ('agent', 'web', 'http', 'enabled'), True, True),
        ("enable-vpn", ('agent', 'vpn', 'enabled'), True, True),
        ("disable-vpn", ('agent', 'vpn', 'enabled'), False, True),
        ("enable-firewall", ('agent', 'firewall', 'enabled'), False, False),
        ("enable-web-password", ('agent', 'web', 'password', 'enabled'), True, False),
    ]:
        run_and_check_success([command], path, value, success)


def test_agent_firewall_commands(setup_wg_quickrs_folder):
    """Test firewall utility and gateway commands with various configurations"""
    setup_wg_quickrs_folder("no_auth_single_peer")
    
    utilities = get_available_firewall_utilities()
    interfaces = get_available_network_interfaces()
    
    if not utilities or not interfaces:
        pytest.skip("No firewall utilities or network interfaces available on this system")
    
    utility = utilities[0]
    gateway = interfaces[0]

    # can't enable firewall without setting utility and gateway
    for command, path, value, success in [
        ("enable-firewall", ('agent', 'firewall', 'enabled'), True, False),
    ]:
        run_and_check_success([command], path, value, success)

    # set firewall utility and gateway tests
    for command, path, value, success in [
        ("set-firewall-utility", ('agent', 'firewall', 'utility'), utility, True),
        ("set-firewall-utility", ('agent', 'firewall', 'utility'), "not-a-utility", False),
        ("set-firewall-gateway", ('agent', 'firewall', 'gateway'), gateway, True),
        ("set-firewall-gateway", ('agent', 'firewall', 'gateway'), "not-a-gateway", False),
    ]:
        run_and_check_success([command, str(value)], path, value, success)

    # with firewall utility and gateway set, we can enable/disable firewall
    for command, path, value, success in [
        ("enable-firewall", ('agent', 'firewall', 'enabled'), True, True),
        ("disable-firewall", ('agent', 'firewall', 'enabled'), False, True),
    ]:
        run_and_check_success([command], path, value, success)


def test_agent_toggle_w_pwd(setup_wg_quickrs_agent):
    setup_wg_quickrs_agent("test_pwd_single_peer")
    for command, path, value, success in [
        ("disable-web-https", ('agent', 'web', 'https', 'enabled'), False, True),
        ("enable-web-https", ('agent', 'web', 'https', 'enabled'), True, True),
        ("disable-web-https", ('agent', 'web', 'https', 'enabled'), False, True),
        ("disable-web-password", ('agent', 'web', 'password', 'enabled'), False, True),
        ("enable-web-password", ('agent', 'web', 'password', 'enabled'), True, True),
    ]:
        run_and_check_success([command], path, value, success)

    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    shutil.rmtree(wg_quickrs_config_folder / 'certs', ignore_errors=True)

    # when the tls cert/key doesn't exist, the toggle should fail
    for command, path, value, success in [
        ("enable-web-https", ('agent', 'web', 'https', 'enabled'), True, False),
    ]:
        run_and_check_success([command], path, value, success)
