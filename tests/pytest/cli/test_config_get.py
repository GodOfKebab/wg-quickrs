import os
import pytest
import subprocess
from tests.pytest.conftest import setup_wg_quickrs_folder
from tests.pytest.helpers import get_wg_quickrs_command, get_paths


@pytest.mark.parametrize(
    "command,expected_value",
    [
        (["get", "agent", "web", "address"], "127.0.0.1"),
        (["get", "agent", "web", "http", "enabled"], "true"),
        (["get", "agent", "web", "http", "port"], "9080"),
        (["get", "agent", "web", "https", "enabled"], "false"),
        (["get", "agent", "web", "https", "port"], "9443"),
        (["get", "agent", "web", "password", "enabled"], "false"),
        (["get", "agent", "vpn", "enabled"], "false"),
        (["get", "agent", "vpn", "port"], "51829"),
        (["get", "agent", "firewall", "enabled"], "false"),
    ],
)
def test_config_get_no_auth(setup_wg_quickrs_folder, command, expected_value):
    """Test that config get commands return correct values from no_auth_single_peer config."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + ["config"] + command,
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode == 0
    # Get the last non-empty line which contains the actual value
    output_lines = [line for line in result.stdout.strip().split('\n') if line]
    assert output_lines[-1] == expected_value


@pytest.mark.parametrize(
    "command,expected_value",
    [
        (["get", "agent", "web", "address"], "127.0.0.1"),
        (["get", "agent", "web", "http", "enabled"], "true"),
        (["get", "agent", "web", "http", "port"], "9080"),
        (["get", "agent", "web", "https", "enabled"], "true"),
        (["get", "agent", "web", "https", "port"], "9443"),
        (["get", "agent", "web", "password", "enabled"], "true"),
        (["get", "agent", "vpn", "enabled"], "false"),
        (["get", "agent", "vpn", "port"], "51829"),
        (["get", "agent", "firewall", "enabled"], "false"),
    ],
)
def test_config_get_with_pwd(setup_wg_quickrs_folder, command, expected_value):
    """Test that config get commands return correct values from test_pwd_single_peer config."""
    setup_wg_quickrs_folder("test_pwd_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + ["config"] + command,
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode == 0
    # Get the last non-empty line which contains the actual value
    output_lines = [line for line in result.stdout.strip().split('\n') if line]
    assert output_lines[-1] == expected_value


def test_config_get_https_tls_cert(setup_wg_quickrs_folder):
    """Test that config get commands return correct TLS certificate path."""
    setup_wg_quickrs_folder("test_pwd_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "agent", "web", "https", "tls-cert"],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode == 0
    # Get the last non-empty line which contains the actual value
    output_lines = [line for line in result.stdout.strip().split('\n') if line]
    assert "certs/servers/127.0.0.1/cert.pem" in output_lines[-1]


def test_config_get_https_tls_key(setup_wg_quickrs_folder):
    """Test that config get commands return correct TLS key path."""
    setup_wg_quickrs_folder("test_pwd_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "agent", "web", "https", "tls-key"],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode == 0
    # Get the last non-empty line which contains the actual value
    output_lines = [line for line in result.stdout.strip().split('\n') if line]
    assert "certs/servers/127.0.0.1/key.pem" in output_lines[-1]


def test_config_get_password_hash(setup_wg_quickrs_folder):
    """Test that config get commands return password hash."""
    setup_wg_quickrs_folder("test_pwd_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "agent", "web", "password", "hash"],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode == 0
    # Get the last non-empty line which contains the actual value
    output_lines = [line for line in result.stdout.strip().split('\n') if line]
    # Should return a non-empty hash (argon2 format starts with $argon2)
    assert output_lines[-1].startswith("$argon2")


def test_config_get_firewall_utility_and_gateway(setup_wg_quickrs_folder):
    """Test that config get commands return firewall utility and gateway (empty when not set)."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    # Test utility
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "agent", "firewall", "utility"],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode == 0
    # For empty values, println!("") outputs a newline after the log messages
    # Remove only the final newline (not all trailing newlines) to preserve the empty line from println!
    if result.stdout.endswith('\n'):
        output_without_final_newline = result.stdout[:-1]
    else:
        output_without_final_newline = result.stdout
    all_lines = output_without_final_newline.split('\n')
    # The last line should be empty (the println!("") output)
    assert all_lines[-1] == ""

    # Test gateway
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "agent", "firewall", "gateway"],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode == 0
    # Empty when not set
    if result.stdout.endswith('\n'):
        output_without_final_newline = result.stdout[:-1]
    else:
        output_without_final_newline = result.stdout
    all_lines = output_without_final_newline.split('\n')
    assert all_lines[-1] == ""


@pytest.mark.parametrize(
    "command",
    [
        ["get", "agent", "web", "address"],
        ["get", "agent", "web", "http", "enabled"],
        ["get", "agent", "web", "http", "port"],
        ["get", "agent", "web", "https", "enabled"],
        ["get", "agent", "web", "https", "port"],
        ["get", "agent", "web", "https", "tls-cert"],
        ["get", "agent", "web", "https", "tls-key"],
        ["get", "agent", "web", "password", "enabled"],
        ["get", "agent", "web", "password", "hash"],
        ["get", "agent", "vpn", "enabled"],
        ["get", "agent", "vpn", "port"],
        ["get", "agent", "firewall", "enabled"],
        ["get", "agent", "firewall", "utility"],
        ["get", "agent", "firewall", "gateway"],
    ],
)
def test_config_get_without_config(setup_wg_quickrs_folder, command):
    """Test that config get commands fail when config file doesn't exist."""
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    setup_wg_quickrs_folder(None)

    # Remove config file if it exists
    if os.path.exists(wg_quickrs_config_file):
        os.remove(wg_quickrs_config_file)

    result = subprocess.run(
        get_wg_quickrs_command() + ["config"] + command,
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode != 0


def test_config_get_after_set(setup_wg_quickrs_folder):
    """Test that config get commands return updated values after set commands."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    # Set a new value
    set_result = subprocess.run(
        get_wg_quickrs_command() + ["config", "set", "agent", "web", "address", "192.168.1.100"],
        capture_output=True,
        text=True
    )
    print(set_result.stdout)
    print(set_result.stderr)
    assert set_result.returncode == 0

    # Get the value and verify it was updated
    get_result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "agent", "web", "address"],
        capture_output=True,
        text=True
    )
    print(get_result.stdout)
    print(get_result.stderr)

    assert get_result.returncode == 0
    # Get the last non-empty line which contains the actual value
    output_lines = [line for line in get_result.stdout.strip().split('\n') if line]
    assert output_lines[-1] == "192.168.1.100"


def test_config_get_after_enable_disable(setup_wg_quickrs_folder):
    """Test that config get commands return updated values after enable/disable commands."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    # Initially HTTP is enabled
    get_result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "agent", "web", "http", "enabled"],
        capture_output=True,
        text=True
    )
    assert get_result.returncode == 0
    output_lines = [line for line in get_result.stdout.strip().split('\n') if line]
    assert output_lines[-1] == "true"

    # Disable HTTP
    disable_result = subprocess.run(
        get_wg_quickrs_command() + ["config", "disable", "agent", "web", "http"],
        capture_output=True,
        text=True
    )
    print(disable_result.stdout)
    print(disable_result.stderr)
    assert disable_result.returncode == 0

    # Get the value and verify it was disabled
    get_result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "agent", "web", "http", "enabled"],
        capture_output=True,
        text=True
    )
    print(get_result.stdout)
    print(get_result.stderr)

    assert get_result.returncode == 0
    output_lines = [line for line in get_result.stdout.strip().split('\n') if line]
    assert output_lines[-1] == "false"

    # Enable HTTP again
    enable_result = subprocess.run(
        get_wg_quickrs_command() + ["config", "enable", "agent", "web", "http"],
        capture_output=True,
        text=True
    )
    print(enable_result.stdout)
    print(enable_result.stderr)
    assert enable_result.returncode == 0

    # Get the value and verify it was enabled
    get_result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "agent", "web", "http", "enabled"],
        capture_output=True,
        text=True
    )
    print(get_result.stdout)
    print(get_result.stderr)

    assert get_result.returncode == 0
    output_lines = [line for line in get_result.stdout.strip().split('\n') if line]
    assert output_lines[-1] == "true"
