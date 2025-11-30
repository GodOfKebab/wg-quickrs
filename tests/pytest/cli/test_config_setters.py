import subprocess
import pytest
from tests.pytest.helpers import get_wg_quickrs_command
from tests.pytest.conftest import setup_wg_quickrs_folder


@pytest.mark.parametrize(
    "command,expected_success",
    [
        (["config", "set", "network", "name", "test-network"], True),
        (["config", "set", "network", "subnet", "10.0.50.0/24"], True),
        (["config", "set", "network", "subnet", "invalid"], False),
    ],
)
def test_config_set_network(setup_wg_quickrs_folder, command, expected_success):
    """Test setting network configuration."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + command,
        capture_output=True,
        text=True
    )

    if expected_success:
        assert result.returncode == 0
    else:
        assert result.returncode != 0


@pytest.mark.parametrize(
    "command,expected_success",
    [
        (["config", "set", "network", "peer", "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "name", "new-peer-name"], True),
        (["config", "set", "network", "peer", "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "address", "10.0.34.50"], True),
        (["config", "set", "network", "peer", "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "endpoint", "192.168.1.1:51820"], True),
        (["config", "set", "network", "peer", "00000000-0000-0000-0000-000000000000", "name", "invalid-peer"], False),
    ],
)
def test_config_set_peer(setup_wg_quickrs_folder, command, expected_success):
    """Test setting peer configuration."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + command,
        capture_output=True,
        text=True
    )

    if expected_success:
        assert result.returncode == 0
    else:
        assert result.returncode != 0


@pytest.mark.parametrize(
    "command",
    [
        ["config", "enable", "network", "peer", "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "endpoint"],
        ["config", "disable", "network", "peer", "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "endpoint"],
    ],
)
def test_config_peer_endpoint_toggle(setup_wg_quickrs_folder, command):
    """Test enabling/disabling peer endpoint."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + command,
        capture_output=True,
        text=True
    )

    assert result.returncode == 0


def test_config_reset_peer_private_key(setup_wg_quickrs_folder):
    """Test resetting peer private key."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    # Get current key
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "network", "peers", "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "private-key"],
        capture_output=True,
        text=True
    )
    old_key = result.stdout.strip()

    # Reset key
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "reset", "network", "peer", "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "private-key"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0

    # Verify key changed
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "network", "peers", "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "private-key"],
        capture_output=True,
        text=True
    )
    new_key = result.stdout.strip()
    assert new_key != old_key


@pytest.mark.parametrize(
    "action",
    ["enable", "disable"],
)
def test_config_connection_toggle(setup_wg_quickrs_folder, action):
    """Test enabling/disabling connection."""
    setup_wg_quickrs_folder("no_auth_multi_peer")

    conn_id = "9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e"
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", action, "network", "connection", conn_id],
        capture_output=True,
        text=True
    )

    assert result.returncode == 0


def test_config_set_connection_allowed_ips(setup_wg_quickrs_folder):
    """Test setting connection allowed IPs."""
    setup_wg_quickrs_folder("no_auth_multi_peer")

    conn_id = "9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e"
    result = subprocess.run(
        get_wg_quickrs_command() + [
            "config", "set", "network", "connection", conn_id,
            "allowed-ips-a-to-b", "0.0.0.0/0"
        ],
        capture_output=True,
        text=True
    )

    assert result.returncode == 0


def test_config_set_connection_persistent_keepalive(setup_wg_quickrs_folder):
    """Test setting connection persistent keepalive."""
    setup_wg_quickrs_folder("no_auth_multi_peer")

    conn_id = "9541bbb0-a3c0-4b83-8637-96820cae7983*0ed989c6-6dba-4e3c-8034-08adf4262d9e"
    result = subprocess.run(
        get_wg_quickrs_command() + [
            "config", "set", "network", "connection", conn_id,
            "persistent-keepalive", "25"
        ],
        capture_output=True,
        text=True
    )

    assert result.returncode == 0


@pytest.mark.parametrize(
    "command",
    [
        ["config", "set", "network", "peer", "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "kind", "laptop"],
        ["config", "set", "network", "peer", "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "icon", "https://example.com/icon.png"],
        ["config", "set", "network", "peer", "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "dns", "8.8.8.8,8.8.4.4"],
        ["config", "set", "network", "peer", "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "mtu", "1420"],
        ["config", "set", "network", "peer", "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "amnezia-parameters", "jc", "40"],
        ["config", "set", "network", "peer", "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "amnezia-parameters", "jmin", "70"],
        ["config", "set", "network", "peer", "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "amnezia-parameters", "jmax", "130"],
    ],
)
def test_config_set_peer_additional_fields(setup_wg_quickrs_folder, command):
    """Test setting peer kind, icon, dns, mtu, and amnezia parameters fields."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + command,
        capture_output=True,
        text=True
    )

    assert result.returncode == 0


@pytest.mark.parametrize(
    "field",
    ["dns", "mtu"],
)
def test_config_peer_field_toggle(setup_wg_quickrs_folder, field):
    """Test enabling/disabling peer dns, mtu."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    # Enable
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "enable", "network", "peer", "0ed989c6-6dba-4e3c-8034-08adf4262d9e", field],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0

    # Disable
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "disable", "network", "peer", "0ed989c6-6dba-4e3c-8034-08adf4262d9e", field],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0


def test_config_peer_icon_toggle(setup_wg_quickrs_folder):
    """Test enabling/disabling peer icon (requires value to be set first)."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    # Set icon value first
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "set", "network", "peer", "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "icon", "test.png"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0

    # Disable (icon was auto-enabled when set)
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "disable", "network", "peer", "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "icon"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0

    # Enable again
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "enable", "network", "peer", "0ed989c6-6dba-4e3c-8034-08adf4262d9e", "icon"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0


@pytest.mark.parametrize(
    "command",
    [
        ["config", "set", "network", "defaults", "peer", "kind", "server"],
        ["config", "set", "network", "defaults", "peer", "icon", "https://example.com/default.png"],
        ["config", "set", "network", "defaults", "peer", "dns", "1.1.1.1,1.0.0.1"],
        ["config", "set", "network", "defaults", "peer", "mtu", "1380"],
        ["config", "set", "network", "defaults", "peer", "amnezia-parameters", "jc", "35"],
        ["config", "set", "network", "defaults", "peer", "amnezia-parameters", "jmin", "65"],
        ["config", "set", "network", "defaults", "peer", "amnezia-parameters", "jmax", "125"],
        ["config", "set", "network", "defaults", "connection", "persistent-keepalive", "30"],
    ],
)
def test_config_set_defaults(setup_wg_quickrs_folder, command):
    """Test setting network defaults."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + command,
        capture_output=True,
        text=True
    )

    assert result.returncode == 0


@pytest.mark.parametrize(
    "field",
    ["dns", "mtu"],
)
def test_config_defaults_peer_field_toggle(setup_wg_quickrs_folder, field):
    """Test enabling/disabling default peer fields."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    # Enable
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "enable", "network", "defaults", "peer", field],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0

    # Disable
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "disable", "network", "defaults", "peer", field],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0


def test_config_defaults_peer_icon_toggle(setup_wg_quickrs_folder):
    """Test enabling/disabling default peer icon (requires value to be set first)."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    # Set icon value first
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "set", "network", "defaults", "peer", "icon", "default.png"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0

    # Disable (icon was auto-enabled when set)
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "disable", "network", "defaults", "peer", "icon"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0

    # Enable again
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "enable", "network", "defaults", "peer", "icon"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0


def test_config_defaults_connection_persistent_keepalive_toggle(setup_wg_quickrs_folder):
    """Test enabling/disabling default connection persistent keepalive."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    # Enable
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "enable", "network", "defaults", "connection", "persistent-keepalive"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0

    # Disable
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "disable", "network", "defaults", "connection", "persistent-keepalive"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0


@pytest.mark.parametrize(
    "command",
    [
        ["config", "set", "network", "amnezia-parameters", "s1", "60"],
        ["config", "set", "network", "amnezia-parameters", "s2", "160"],
        ["config", "set", "network", "amnezia-parameters", "h1", "800000000"],
        ["config", "set", "network", "amnezia-parameters", "h2", "1700000000"],
        ["config", "set", "network", "amnezia-parameters", "h3", "2900000000"],
        ["config", "set", "network", "amnezia-parameters", "h4", "3300000000"],
    ],
)
def test_config_set_network_amnezia_parameters(setup_wg_quickrs_folder, command):
    """Test setting network amnezia parameters."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + command,
        capture_output=True,
        text=True
    )

    assert result.returncode == 0

@pytest.mark.parametrize(
    "wg_tool, success",
    [
        ("wg", False),
        ("awg", True),
    ],
)
def test_config_network_amnezia_parameters_toggle(setup_wg_quickrs_folder, wg_tool, success):
    """Test enabling/disabling network amnezia parameters."""
    if wg_tool == "awg":
        which_conf = "no_auth_single_peer_awg"
    else:
        which_conf = "no_auth_single_peer"
    setup_wg_quickrs_folder(which_conf)

    # Enable
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "enable", "network", "amnezia-parameters"],
        capture_output=True,
        text=True
    )
    assert (result.returncode == 0) == success

    # Disable
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "disable", "network", "amnezia-parameters"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0


def test_config_agent_vpn_wg_userspace_toggle(setup_wg_quickrs_folder):
    """Test enabling/disabling agent VPN WireGuard userspace mode."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    # Enable
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "enable", "agent", "vpn", "wg-userspace"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0

    # Disable
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "disable", "agent", "vpn", "wg-userspace"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0
