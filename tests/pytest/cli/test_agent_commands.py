import os
import shutil

from tests.pytest.conftest import setup_wg_quickrs_folder
from tests.pytest.helpers import (
    get_wg_quickrs_command,
    get_paths,
    get_available_firewall_utilities,
    get_available_network_interfaces,
    deep_get
)
import subprocess
from deepdiff import DeepDiff
import pytest
from ruamel.yaml import YAML
yaml = YAML()
yaml.preserve_quotes = True



def run_and_check_success(cmd, field, field_val, success):
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.load(stream)

    result = subprocess.run(
        get_wg_quickrs_command() + ['config'] + cmd,
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert (result.returncode == 0) == success
    if not success:
        return

    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.load(stream)

    # check diff
    diff = DeepDiff(old_conf, new_conf, ignore_order=True)
    expected_path = "root" + "".join(f"['{k}']" for k in field)

    # If the field type changes from str to PathBuf (like when setting fw utility), only check 'type_changes'
    if 'type_changes' in diff:
        assert diff["type_changes"][expected_path]["new_value"] == field_val
        return
    # Otherwise, check 'values_changed'
    assert diff["values_changed"][expected_path]["old_value"] == deep_get(old_conf, field)
    # Python testing doesn't calculate hashes, so we can't compare the hash values
    if cmd[0] != "reset":
        assert diff["values_changed"][expected_path]["new_value"] == field_val


@pytest.mark.parametrize(
    "command, path, value, success",
    [
        (["agent", "web", "address"], ('agent', 'web', 'address'), "192.168.10.10", True),
        (["agent", "web", "address"], ('agent', 'web', 'address'), 'not-an-address', False),
        (["agent", "web", "http", "port"], ('agent', 'web', 'http', 'port'), 80, True),
        (["agent", "web", "http", "port"], ('agent', 'web', 'http', 'port'), "not-a-port", False),
        (["agent", "web", "https", "port"], ('agent', 'web', 'https', 'port'), 443, True),
        (["agent", "web", "https", "port"], ('agent', 'web', 'https', 'port'), "not-a-port", False),
        # since the tls cert generation is not tested here, we skip the successful https test case
        (["agent", "vpn", "port"], ('agent', 'vpn', 'port'), 51821, True),
        (["agent", "vpn", "port"], ('agent', 'vpn', 'port'), "not-a-port", False),
        # skip the successful firewall setting test case because the gateway and utility names will differ in different machines
    ],
)
def test_agent_set_simple(setup_wg_quickrs_folder, command, path, value, success):
    setup_wg_quickrs_folder("no_auth_single_peer")
    run_and_check_success(["set"] + command + [str(value)], path, value, success)


def test_agent_set_https_cert_key(setup_wg_quickrs_folder):
    setup_wg_quickrs_folder("test_pwd_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    os.makedirs(wg_quickrs_config_folder / 'certs/servers/localhost')
    shutil.copyfile(wg_quickrs_config_folder / 'certs/servers/127.0.0.1/cert.pem', wg_quickrs_config_folder / 'certs/servers/localhost/cert.pem')
    shutil.copyfile(wg_quickrs_config_folder / 'certs/servers/127.0.0.1/key.pem', wg_quickrs_config_folder / 'certs/servers/localhost/key.pem')

    for command, path, value, success in [
        (["agent", "web", "https", "tls-cert"], ('agent', 'web', 'https', 'tls_cert'), 'certs/servers/localhost/cert.pem', True),
        (["agent", "web", "https", "tls-cert"], ('agent', 'web', 'https', 'tls_cert'), "not-a-cert-path", False),
        (["agent", "web", "https", "tls-key"], ('agent', 'web', 'https', 'tls_key'), 'certs/servers/localhost/key.pem', True),
        (["agent", "web", "https", "tls-key"], ('agent', 'web', 'https', 'tls_key'), "not-a-key-path", False),
    ]:
        run_and_check_success(["set"] + command + [str(value)], path, value, success)


def test_agent_reset_web_password(setup_wg_quickrs_folder):
    setup_wg_quickrs_folder("test_pwd_single_peer")

    for command, path, value, success in [
        (["agent", "web", "password"], ('agent', 'web', 'password', 'hash'), "new-test-pwd", True),
    ]:
        run_and_check_success(["reset"] + command + ["--password", str(value)], path, value, success)


def test_agent_toggle_simple(setup_wg_quickrs_folder):
    setup_wg_quickrs_folder("no_auth_single_peer")
    for action, target, path, value, success in [
        ("disable", ["agent", "web", "http"], ('agent', 'web', 'http', 'enabled'), False, True),
        ("enable", ["agent", "web", "http"], ('agent', 'web', 'http', 'enabled'), True, True),
        ("enable", ["agent", "vpn"], ('agent', 'vpn', 'enabled'), True, True),
        ("disable", ["agent", "vpn"], ('agent', 'vpn', 'enabled'), False, True),
        ("enable", ["agent", "firewall"], ('agent', 'firewall', 'enabled'), False, False),
        ("enable", ["agent", "web", "password"], ('agent', 'web', 'password', 'enabled'), True, False),
    ]:
        run_and_check_success([action] + target, path, value, success)


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
    for action, target, path, value, success in [
        ("enable", ["agent", "firewall"], ('agent', 'firewall', 'enabled'), True, False),
    ]:
        run_and_check_success([action] + target, path, value, success)

    # set firewall utility and gateway tests
    for command, path, value, success in [
        (["agent", "firewall", "utility"], ('agent', 'firewall', 'utility'), utility, True),
        (["agent", "firewall", "utility"], ('agent', 'firewall', 'utility'), "not-a-utility", False),
        (["agent", "firewall", "gateway"], ('agent', 'firewall', 'gateway'), gateway, True),
        (["agent", "firewall", "gateway"], ('agent', 'firewall', 'gateway'), "not-a-gateway", False),
    ]:
        run_and_check_success(["set"] + command + [str(value)], path, value, success)

    # with firewall utility and gateway set, we can enable/disable firewall
    for action, target, path, value, success in [
        ("enable", ["agent", "firewall"], ('agent', 'firewall', 'enabled'), True, True),
        ("disable", ["agent", "firewall"], ('agent', 'firewall', 'enabled'), False, True),
    ]:
        run_and_check_success([action] + target, path, value, success)


def test_agent_toggle_w_pwd(setup_wg_quickrs_folder):
    setup_wg_quickrs_folder("test_pwd_single_peer")
    for action, target, path, value, success in [
        ("disable", ["agent", "web", "https"], ('agent', 'web', 'https', 'enabled'), False, True),
        ("enable", ["agent", "web", "https"], ('agent', 'web', 'https', 'enabled'), True, True),
        ("disable", ["agent", "web", "https"], ('agent', 'web', 'https', 'enabled'), False, True),
        ("disable", ["agent", "web", "password"], ('agent', 'web', 'password', 'enabled'), False, True),
        ("enable", ["agent", "web", "password"], ('agent', 'web', 'password', 'enabled'), True, True),
    ]:
        run_and_check_success([action] + target, path, value, success)

    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    shutil.rmtree(wg_quickrs_config_folder / 'certs', ignore_errors=True)

    # when the tls cert/key doesn't exist, the toggle should fail
    for action, target, path, value, success in [
        ("enable", ["agent", "web", "https"], ('agent', 'web', 'https', 'enabled'), True, False),
    ]:
        run_and_check_success([action] + target, path, value, success)


def test_agent_commands_without_config(setup_wg_quickrs_folder):
    """Test that agent commands fail when config file doesn't exist."""
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    setup_wg_quickrs_folder(None)

    # Remove config file if it exists
    if os.path.exists(wg_quickrs_config_file):
        os.remove(wg_quickrs_config_file)

    commands_to_test = [
        ["set", "agent", "web", "address", "192.168.1.1"],
        ["set", "agent", "web", "http", "port", "8080"],
        ["set", "agent", "vpn", "port", "51821"],
        ["enable", "agent", "web", "http"],
        ["disable", "agent", "web", "http"],
        ["enable", "agent", "vpn"],
        ["disable", "agent", "vpn"],
    ]

    for command in commands_to_test:
        result = subprocess.run(
            get_wg_quickrs_command() + command,
            capture_output=True,
            text=True
        )
        assert result.returncode != 0


@pytest.mark.parametrize(
    "command,args",
    [
        ("set", ["agent", "web", "address"]),  # missing value
        ("set", ["agent", "web", "http", "port"]),  # missing value
        ("set", ["agent", "vpn", "port"]),  # missing value
        ("set", ["agent", "firewall", "utility"]),  # missing value
        ("set", ["agent", "firewall", "gateway"]),  # missing value
    ],
)
def test_agent_commands_missing_arguments(setup_wg_quickrs_folder, command, args):
    """Test that agent commands fail when required arguments are missing."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + [command] + args,
        capture_output=True,
        text=True
    )
    assert result.returncode != 0


def test_agent_run_command(setup_wg_quickrs_folder):
    """Test that agent run command works (just starting, not full execution)."""
    setup_wg_quickrs_folder("no_auth_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    # Test that run command with invalid config fails appropriately
    # Corrupt the config file
    with open(wg_quickrs_config_file, 'w') as f:
        f.write("invalid: yaml: content: [")

    result = subprocess.run(
        get_wg_quickrs_command() + ['run'],
        capture_output=True,
        text=True,
        timeout=2
    )
    assert result.returncode != 0
