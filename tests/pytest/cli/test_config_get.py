import os
import pytest
import subprocess
from tests.pytest.conftest import setup_wg_quickrs_folder
from tests.pytest.helpers import get_wg_quickrs_command, get_paths
from ruamel.yaml import YAML
yaml = YAML()
yaml.preserve_quotes = True


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


# ========== Tests for YAML struct output ==========

def get_yaml_output(command):
    """Helper to get and parse YAML output from a config get command."""
    result = subprocess.run(
        get_wg_quickrs_command() + ["config"] + command,
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode == 0
    return yaml.load(result.stdout)


def get_config_field_from_file(field_path):
    """Helper to read a field directly from the config file using ruamel.yaml."""
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    with open(wg_quickrs_config_file) as f:
        config = yaml.load(f)

    # Navigate to the field
    current = config
    for key in field_path:
        current = current[key]

    return current


@pytest.mark.parametrize(
    "command,expected_keys",
    [
        (["get", "agent"], ["web", "vpn", "firewall"]),
        (["get", "agent", "web"], ["address", "http", "https", "password"]),
        (["get", "agent", "web", "http"], ["enabled", "port"]),
        (["get", "agent", "web", "https"], ["enabled", "port", "tls_cert", "tls_key"]),
        (["get", "agent", "web", "password"], ["enabled", "hash"]),
        (["get", "agent", "vpn"], ["enabled", "port"]),
        (["get", "agent", "firewall"], ["enabled", "utility", "gateway"]),
    ],
)
def test_config_get_yaml_struct_no_auth(setup_wg_quickrs_folder, command, expected_keys):
    """Test that config get commands return valid YAML for struct objects."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    data = get_yaml_output(command)

    # Verify all expected keys are present
    for key in expected_keys:
        assert key in data, f"Expected key '{key}' not found in YAML output"


@pytest.mark.parametrize(
    "command,field_path,expected_value",
    [
        (["get", "agent"], ["web", "address"], "127.0.0.1"),
        (["get", "agent"], ["web", "http", "enabled"], True),
        (["get", "agent"], ["web", "http", "port"], 9080),
        (["get", "agent"], ["vpn", "enabled"], False),
        (["get", "agent"], ["vpn", "port"], 51829),
        (["get", "agent"], ["firewall", "enabled"], False),
        (["get", "agent", "web"], ["address"], "127.0.0.1"),
        (["get", "agent", "web"], ["http", "enabled"], True),
        (["get", "agent", "web"], ["http", "port"], 9080),
        (["get", "agent", "web", "http"], ["enabled"], True),
        (["get", "agent", "web", "http"], ["port"], 9080),
        (["get", "agent", "vpn"], ["enabled"], False),
        (["get", "agent", "vpn"], ["port"], 51829),
    ],
)
def test_config_get_yaml_struct_values(setup_wg_quickrs_folder, command, field_path, expected_value):
    """Test that config get YAML output contains correct values."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    data = get_yaml_output(command)

    # Navigate to the field
    current = data
    for key in field_path:
        assert key in current, f"Key '{key}' not found while navigating to {'.'.join(field_path)}"
        current = current[key]

    assert current == expected_value, f"Expected {expected_value}, got {current}"


def test_config_get_yaml_full_agent_structure(setup_wg_quickrs_folder):
    """Test that getting full agent returns complete structure."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    data = get_yaml_output(["get", "agent"])

    # Verify top-level structure
    assert "web" in data
    assert "vpn" in data
    assert "firewall" in data

    # Verify web structure
    assert data["web"]["address"] == "127.0.0.1"
    assert "http" in data["web"]
    assert "https" in data["web"]
    assert "password" in data["web"]

    # Verify nested structures have expected fields
    assert data["web"]["http"]["enabled"] == True
    assert data["web"]["http"]["port"] == 9080
    assert data["web"]["https"]["enabled"] == False
    assert data["web"]["https"]["port"] == 9443


def test_config_get_yaml_with_modified_values(setup_wg_quickrs_folder):
    """Test that YAML output reflects modified values."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    # Modify some values
    subprocess.run(
        get_wg_quickrs_command() + ["config", "set", "agent", "web", "http", "port", "8080"],
        capture_output=True,
        text=True
    )
    subprocess.run(
        get_wg_quickrs_command() + ["config", "disable", "agent", "web", "http"],
        capture_output=True,
        text=True
    )

    # Get the YAML output
    data = get_yaml_output(["get", "agent", "web", "http"])

    # Verify the changes are reflected
    assert data["enabled"] == False
    assert data["port"] == 8080


def test_config_get_yaml_struct_with_password(setup_wg_quickrs_folder):
    """Test YAML output with password-enabled configuration."""
    setup_wg_quickrs_folder("test_pwd_single_peer")

    data = get_yaml_output(["get", "agent", "web"])

    # Verify password is enabled
    assert data["password"]["enabled"] == True
    # Hash should be non-empty
    assert data["password"]["hash"] != ""
    assert data["password"]["hash"].startswith("$argon2")


def test_config_get_yaml_agent_vpn_structure(setup_wg_quickrs_folder):
    """Test getting VPN structure as YAML."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    data = get_yaml_output(["get", "agent", "vpn"])

    assert data["enabled"] == False
    assert data["port"] == 51829
    assert len(data) == 2  # Should only have enabled and port


def test_config_get_yaml_agent_firewall_structure(setup_wg_quickrs_folder):
    """Test getting firewall structure as YAML."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    data = get_yaml_output(["get", "agent", "firewall"])

    assert data["enabled"] == False
    assert data["utility"] == ""
    assert data["gateway"] == ""
    assert len(data) == 3  # Should only have enabled, utility, and gateway


def test_config_get_yaml_consistency_with_individual_gets(setup_wg_quickrs_folder):
    """Test that YAML struct output matches individual field gets."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    # Get struct as YAML
    http_struct = get_yaml_output(["get", "agent", "web", "http"])

    # Get individual fields
    enabled_result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "agent", "web", "http", "enabled"],
        capture_output=True,
        text=True
    )
    port_result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "agent", "web", "http", "port"],
        capture_output=True,
        text=True
    )

    enabled_lines = [line for line in enabled_result.stdout.strip().split('\n') if line]
    port_lines = [line for line in port_result.stdout.strip().split('\n') if line]

    # Compare values
    assert str(http_struct["enabled"]).lower() == enabled_lines[-1].lower()
    assert str(http_struct["port"]) == port_lines[-1]


# ========== Tests for Network Configuration ==========

@pytest.mark.parametrize(
    "command,expected_value",
    [
        (["get", "network", "name"], "wg-quickrs-home"),
        (["get", "network", "subnet"], "10.0.34.0/24"),
        (["get", "network", "this-peer"], "0ed989c6-6dba-4e3c-8034-08adf4262d9e"),
    ],
)
def test_config_get_network_no_auth(setup_wg_quickrs_folder, command, expected_value):
    """Test that network config get commands return correct values from no_auth_single_peer config."""
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
    "command",
    [
        ["get", "network", "name"],
        ["get", "network", "subnet"],
        ["get", "network", "this-peer"],
    ],
)
def test_config_get_network_without_config(setup_wg_quickrs_folder, command):
    """Test that network config get commands fail when config file doesn't exist."""
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


@pytest.mark.parametrize(
    "command,expected_keys",
    [
        (["get", "network"], ["name", "subnet", "this_peer", "peers", "connections", "defaults", "reservations", "updated_at"]),
    ],
)
def test_config_get_network_yaml_struct(setup_wg_quickrs_folder, command, expected_keys):
    """Test that network config get command returns valid YAML with all expected keys."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    data = get_yaml_output(command)

    # Verify all expected keys are present
    for key in expected_keys:
        assert key in data, f"Expected key '{key}' not found in YAML output"


@pytest.mark.parametrize(
    "command,field_path,expected_value",
    [
        (["get", "network"], ["name"], "wg-quickrs-home"),
        (["get", "network"], ["subnet"], "10.0.34.0/24"),
        (["get", "network"], ["this_peer"], "0ed989c6-6dba-4e3c-8034-08adf4262d9e"),
    ],
)
def test_config_get_network_yaml_values(setup_wg_quickrs_folder, command, field_path, expected_value):
    """Test that network YAML output contains correct values."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    data = get_yaml_output(command)

    # Navigate to the field
    current = data
    for key in field_path:
        assert key in current, f"Key '{key}' not found while navigating to {'.'.join(field_path)}"
        current = current[key]

    assert str(current) == expected_value, f"Expected {expected_value}, got {current}"


def test_config_get_network_full_structure(setup_wg_quickrs_folder):
    """Test that getting full network returns complete structure with nested objects."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    data = get_yaml_output(["get", "network"])

    # Verify top-level structure
    assert "name" in data
    assert "subnet" in data
    assert "this_peer" in data
    assert "peers" in data
    assert "connections" in data
    assert "defaults" in data
    assert "reservations" in data
    assert "updated_at" in data

    # Verify simple values
    assert data["name"] == "wg-quickrs-home"
    assert data["subnet"] == "10.0.34.0/24"
    assert data["this_peer"] == "0ed989c6-6dba-4e3c-8034-08adf4262d9e"

    # Verify nested structures exist
    assert isinstance(data["peers"], dict)
    assert isinstance(data["connections"], dict)
    assert isinstance(data["defaults"], dict)
    assert isinstance(data["reservations"], dict)

    # Verify defaults structure
    assert "peer" in data["defaults"]
    assert "connection" in data["defaults"]


def test_config_get_network_yaml_consistency_with_individual_gets(setup_wg_quickrs_folder):
    """Test that YAML network struct output matches individual field gets."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    # Get struct as YAML
    network_struct = get_yaml_output(["get", "network"])

    # Get individual fields
    name_result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "network", "name"],
        capture_output=True,
        text=True
    )
    subnet_result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "network", "subnet"],
        capture_output=True,
        text=True
    )
    this_peer_result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "network", "this-peer"],
        capture_output=True,
        text=True
    )

    name_lines = [line for line in name_result.stdout.strip().split('\n') if line]
    subnet_lines = [line for line in subnet_result.stdout.strip().split('\n') if line]
    this_peer_lines = [line for line in this_peer_result.stdout.strip().split('\n') if line]

    # Compare values
    assert network_struct["name"] == name_lines[-1]
    assert network_struct["subnet"] == subnet_lines[-1]
    assert network_struct["this_peer"] == this_peer_lines[-1]


def test_config_get_network_with_pwd_config(setup_wg_quickrs_folder):
    """Test network get commands work with different config files."""
    setup_wg_quickrs_folder("test_pwd_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "network", "name"],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode == 0
    output_lines = [line for line in result.stdout.strip().split('\n') if line]
    # Should return network name (may be different from no_auth_single_peer)
    assert len(output_lines[-1]) > 0


# ========== Tests for Network Peers ==========

def test_config_get_network_peers_all(setup_wg_quickrs_folder):
    """Test getting all peers returns YAML."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    data = get_yaml_output(["get", "network", "peers"])

    # Should have at least one peer
    assert isinstance(data, dict)
    assert len(data) > 0
    # The test config has one peer
    assert "0ed989c6-6dba-4e3c-8034-08adf4262d9e" in data


def test_config_get_network_peer_by_id(setup_wg_quickrs_folder):
    """Test getting specific peer by UUID."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    peer_id = "0ed989c6-6dba-4e3c-8034-08adf4262d9e"
    data = get_yaml_output(["get", "network", "peers", peer_id])

    # Verify peer structure
    assert "name" in data
    assert "address" in data
    assert "endpoint" in data
    assert "kind" in data
    assert data["name"] == "wg-quickrs-host"
    assert data["address"] == "10.0.34.1"


@pytest.mark.parametrize(
    "peer_id,field,expected_value",
    [
        ("0ed989c6-6dba-4e3c-8034-08adf4262d9e", "name", "wg-quickrs-host"),
        ("0ed989c6-6dba-4e3c-8034-08adf4262d9e", "address", "10.0.34.1"),
        ("0ed989c6-6dba-4e3c-8034-08adf4262d9e", "kind", "server"),
    ],
)
def test_config_get_network_peer_field(setup_wg_quickrs_folder, peer_id, field, expected_value):
    """Test getting individual peer fields."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "network", "peers", peer_id, field],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode == 0
    output_lines = [line for line in result.stdout.strip().split('\n') if line]
    assert output_lines[-1] == expected_value


def test_config_get_network_peer_not_found(setup_wg_quickrs_folder):
    """Test that getting non-existent peer returns error."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    fake_peer_id = "00000000-0000-0000-0000-000000000000"
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "network", "peers", fake_peer_id],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode != 0
    # Error is logged to stdout
    assert "peer not found" in result.stdout


# ========== Tests for Network Defaults ==========

def test_config_get_network_defaults(setup_wg_quickrs_folder):
    """Test getting all defaults."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    data = get_yaml_output(["get", "network", "defaults"])

    assert "peer" in data
    assert "connection" in data


def test_config_get_network_defaults_peer(setup_wg_quickrs_folder):
    """Test getting default peer configuration."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    data = get_yaml_output(["get", "network", "defaults", "peer"])

    assert "kind" in data
    assert "icon" in data
    assert "dns" in data
    assert "mtu" in data
    assert "scripts" in data
    assert data["kind"] == "laptop"


def test_config_get_network_defaults_connection(setup_wg_quickrs_folder):
    """Test getting default connection configuration."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    data = get_yaml_output(["get", "network", "defaults", "connection"])

    assert "persistent_keepalive" in data
    assert data["persistent_keepalive"]["enabled"] == True
    assert data["persistent_keepalive"]["period"] == 25


# ========== Tests for Network Updated At ==========

def test_config_get_network_updated_at(setup_wg_quickrs_folder):
    """Test getting network updated timestamp."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "network", "updated-at"],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode == 0
    output_lines = [line for line in result.stdout.strip().split('\n') if line]
    # Should return a timestamp (format: 2025-10-04 00:36:44 UTC or 2025-10-04T00:36:44Z)
    assert len(output_lines[-1]) > 0
    # Check for common timestamp indicators
    assert ("UTC" in output_lines[-1] or "Z" in output_lines[-1] or
            "+" in output_lines[-1] or "-" in output_lines[-1])


# ========== Tests for Network Connections and Reservations ==========

def test_config_get_network_connections_empty(setup_wg_quickrs_folder):
    """Test getting connections when there are none."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    data = get_yaml_output(["get", "network", "connections"])

    # Should be empty dict
    assert isinstance(data, dict)
    assert len(data) == 0


def test_config_get_network_reservations_empty(setup_wg_quickrs_folder):
    """Test getting reservations when there are none."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    data = get_yaml_output(["get", "network", "reservations"])

    # Should be empty dict
    assert isinstance(data, dict)
    assert len(data) == 0


# ========== Tests for Combined Getters ==========

def test_config_get_network_and_agent_together(setup_wg_quickrs_folder):
    """Test that both agent and network getters work in the same session."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    # Get agent web address
    agent_result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "agent", "web", "address"],
        capture_output=True,
        text=True
    )
    assert agent_result.returncode == 0

    # Get network name
    network_result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "network", "name"],
        capture_output=True,
        text=True
    )
    assert network_result.returncode == 0

    # Get full config
    full_result = get_yaml_output(["get", "network"])
    assert "name" in full_result
    assert "peers" in full_result
