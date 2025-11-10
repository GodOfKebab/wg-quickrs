from tests.pytest.conftest import setup_wg_quickrs_folder
from tests.pytest.helpers import get_wg_quickrs_command, get_paths
import subprocess
import pytest


def add_peer_no_prompt(opts):
    """Run config add peer with --no-prompt."""
    command = " ".join(get_wg_quickrs_command()) + " config add peer --no-prompt true \\"
    command += opts

    result = subprocess.run(
        command,
        shell=True,
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)
    return result.returncode


def add_connection_no_prompt(opts):
    """Run config add connection with --no-prompt."""
    command = " ".join(get_wg_quickrs_command()) + " config add connection --no-prompt true \\"
    command += opts

    result = subprocess.run(
        command,
        shell=True,
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)
    return result.returncode


def generate_add_peer_opts(
        name="test-peer",
        kind="laptop",
        address="10.0.34.100",  # Use an address that's not taken
        endpoint="--endpoint-enabled false",
        icon="--icon-enabled false",
        dns="--dns-enabled false",
        mtu="--mtu-enabled false",
        script_pre_up="--script-pre-up-enabled false",
        script_post_up="--script-post-up-enabled false",
        script_pre_down="--script-pre-down-enabled false",
        script_post_down="--script-post-down-enabled false",
):
    """Generate add peer options with defaults."""
    return f"""
    --name {name} \\
    --address {address} \\
    --kind {kind} \\
    {endpoint} \\
    {icon} \\
    {dns} \\
    {mtu} \\
    {script_pre_up} \\
    {script_post_up} \\
    {script_pre_down} \\
    {script_post_down}
    """


def generate_add_connection_opts(
        first_peer="9541bbb0-a3c0-4b83-8637-96820cae7983",
        second_peer="6e9a8440-f884-4b54-bfe7-b982f15e40fd",
        allowed_ips_a_to_b="10.0.34.1/32",
        allowed_ips_b_to_a="10.0.34.2/32",
        persistent_keepalive="--persistent-keepalive-enabled false",
):
    """Generate add connection options with defaults."""
    return f"""
    --first-peer {first_peer} \\
    --second-peer {second_peer} \\
    --allowed-ips-a-to-b {allowed_ips_a_to_b} \\
    --allowed-ips-b-to-a {allowed_ips_b_to_a} \\
    {persistent_keepalive}
    """


def test_add_peer_no_prompt_simple(setup_wg_quickrs_folder):
    """Test adding a peer with minimal configuration."""
    setup_wg_quickrs_folder("no_auth_single_peer")
    assert add_peer_no_prompt(generate_add_peer_opts()) == 0


@pytest.mark.parametrize(
    "opt_key, opt_val, success",
    [
        # Name tests
        ("name", "valid-peer-name", True),
        ("name", "peer with spaces", False),
        ("name", "", False),

        # Kind tests
        ("kind", "laptop", True),
        ("kind", "server", True),
        ("kind", "phone", True),

        # Address tests
        ("address", "10.0.34.10", True),
        ("address", "10.0.34.1", False),  # Already taken by existing peer
        ("address", "192.168.1.1", False),  # Outside subnet
        ("address", "not-an-address", False),

        # Endpoint tests
        ("endpoint", "--endpoint-enabled true", False),  # Missing endpoint address
        ("endpoint", "--endpoint-enabled true --endpoint-address example.com:51820", True),
        ("endpoint", "--endpoint-enabled true --endpoint-address 192.168.1.1:51820", True),
        ("endpoint", "--endpoint-enabled true --endpoint-address not-an-endpoint", False),
        ("endpoint", "--endpoint-enabled true --endpoint-address example.com:not-a-port", False),

        # Icon tests
        ("icon", "--icon-enabled true", False),  # Missing icon src
        ("icon", "--icon-enabled true --icon-src example-src", True),
        ("icon", "--icon-enabled true --icon-src https://example.com/icon.png", True),

        # DNS tests
        ("dns", "--dns-enabled true", False),  # Missing DNS addresses
        ("dns", "--dns-enabled true --dns-addresses 1.1.1.1", True),
        ("dns", "--dns-enabled true --dns-addresses 1.1.1.1,8.8.8.8", True),
        ("dns", "--dns-enabled true --dns-addresses not-an-address", False),

        # MTU tests
        ("mtu", "--mtu-enabled true", False),  # Missing MTU value
        ("mtu", "--mtu-enabled true --mtu-value 1420", True),
        ("mtu", "--mtu-enabled true --mtu-value 1280", True),
        ("mtu", "--mtu-enabled true --mtu-value not-a-number", False),
        ("mtu", "--mtu-enabled true --mtu-value 0", False),

        # Script tests
        ("script_pre_up", "--script-pre-up-enabled true", False),
        ("script_pre_up", "--script-pre-up-enabled true --script-pre-up-line 'echo hi;'", True),
        ("script_pre_up", "--script-pre-up-enabled true --script-pre-up-line not-a-script", False),

        ("script_post_up", "--script-post-up-enabled true", False),
        ("script_post_up", "--script-post-up-enabled true --script-post-up-line 'echo hi;'", True),
        ("script_post_up", "--script-post-up-enabled true --script-post-up-line not-a-script", False),

        ("script_pre_down", "--script-pre-down-enabled true", False),
        ("script_pre_down", "--script-pre-down-enabled true --script-pre-down-line 'echo hi;'", True),
        ("script_pre_down", "--script-pre-down-enabled true --script-pre-down-line not-a-script", False),

        ("script_post_down", "--script-post-down-enabled true", False),
        ("script_post_down", "--script-post-down-enabled true --script-post-down-line 'echo hi;'", True),
        ("script_post_down", "--script-post-down-enabled true --script-post-down-line not-a-script", False),
    ],
)
def test_add_peer_no_prompt(setup_wg_quickrs_folder, opt_key, opt_val, success):
    """Test add peer with various options."""
    setup_wg_quickrs_folder("no_auth_single_peer")
    assert (add_peer_no_prompt(generate_add_peer_opts(**{opt_key: opt_val})) == 0) == success


def test_add_peer_missing_required_with_no_prompt(setup_wg_quickrs_folder):
    """Test that --no-prompt fails when required options are missing."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    command = " ".join(get_wg_quickrs_command()) + " config add peer --no-prompt true"
    result = subprocess.run(command, shell=True, capture_output=True, text=True)

    assert result.returncode != 0
    assert "Error: CLI option" in result.stdout or "Error: CLI option" in result.stderr or "ERROR" in result.stdout or "ERROR" in result.stderr


def test_add_connection_no_prompt_simple(setup_wg_quickrs_folder):
    """Test adding a connection with minimal configuration."""
    setup_wg_quickrs_folder("no_auth_multi_peer")
    assert add_connection_no_prompt(generate_add_connection_opts()) == 0


@pytest.mark.parametrize(
    "opt_key, opt_val, success",
    [
        # Allowed IPs tests
        ("allowed_ips_a_to_b", "10.0.34.0/24", True),
        ("allowed_ips_a_to_b", "10.0.34.1/32,10.0.34.2/32", True),
        ("allowed_ips_a_to_b", "not-a-cidr", False),

        ("allowed_ips_b_to_a", "10.0.34.0/24", True),
        ("allowed_ips_b_to_a", "10.0.34.1/32,10.0.34.2/32", True),
        ("allowed_ips_b_to_a", "not-a-cidr", False),

        # Persistent keepalive tests
        ("persistent_keepalive", "--persistent-keepalive-enabled true", False),  # Missing period
        ("persistent_keepalive", "--persistent-keepalive-enabled true --persistent-keepalive-period 25", True),
        ("persistent_keepalive", "--persistent-keepalive-enabled true --persistent-keepalive-period 0", False),
        ("persistent_keepalive", "--persistent-keepalive-enabled true --persistent-keepalive-period not-a-number", False),
    ],
)
def test_add_connection_no_prompt(setup_wg_quickrs_folder, opt_key, opt_val, success):
    """Test add connection with various options."""
    setup_wg_quickrs_folder("no_auth_multi_peer")
    assert (add_connection_no_prompt(generate_add_connection_opts(**{opt_key: opt_val})) == 0) == success


def test_add_connection_duplicate(setup_wg_quickrs_folder):
    """Test that adding a duplicate connection fails."""
    setup_wg_quickrs_folder("no_auth_multi_peer")

    # Add connection once - should succeed
    assert add_connection_no_prompt(generate_add_connection_opts()) == 0

    # Try to add the same connection again - should fail
    assert add_connection_no_prompt(generate_add_connection_opts()) != 0


def test_add_connection_nonexistent_peers(setup_wg_quickrs_folder):
    """Test that adding a connection with nonexistent peers fails."""
    setup_wg_quickrs_folder("no_auth_multi_peer")

    # Try with a nonexistent first peer
    ret = add_connection_no_prompt(generate_add_connection_opts(
        first_peer="00000000-0000-0000-0000-000000000000"
    ))
    assert ret != 0

    # Try with a nonexistent second peer
    ret = add_connection_no_prompt(generate_add_connection_opts(
        second_peer="00000000-0000-0000-0000-000000000000"
    ))
    assert ret != 0


def test_add_connection_same_peer(setup_wg_quickrs_folder):
    """Test that connecting a peer to itself fails."""
    setup_wg_quickrs_folder("no_auth_multi_peer")

    peer_id = "9541bbb0-a3c0-4b83-8637-96820cae7983"
    ret = add_connection_no_prompt(generate_add_connection_opts(
        first_peer=peer_id,
        second_peer=peer_id
    ))
    assert ret != 0


def test_add_connection_missing_required_with_no_prompt(setup_wg_quickrs_folder):
    """Test that --no-prompt fails when required options are missing."""
    setup_wg_quickrs_folder("no_auth_multi_peer")

    command = " ".join(get_wg_quickrs_command()) + " config add connection --no-prompt true"
    result = subprocess.run(command, shell=True, capture_output=True, text=True)

    assert result.returncode != 0
    assert "ERROR" in result.stdout or "ERROR" in result.stderr
