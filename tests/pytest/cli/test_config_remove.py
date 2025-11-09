import subprocess
import pytest
from tests.pytest.helpers import get_wg_quickrs_command
from tests.pytest.conftest import setup_wg_quickrs_folder


def test_config_remove_peer_success(setup_wg_quickrs_folder):
    """Test successfully removing a peer."""
    setup_wg_quickrs_folder("no_auth_multi_peer")

    # Verify peer exists first (using a peer that is NOT this_peer)
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "get", "network", "peers", "9541bbb0-a3c0-4b83-8637-96820cae7983", "name"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0

    # Verify initial peer count
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "list", "peers"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0
    initial_peers = [line for line in result.stdout.strip().split('\n') if line]
    assert len(initial_peers) == 3

    # Remove the peer
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "remove", "peer", "9541bbb0-a3c0-4b83-8637-96820cae7983"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0

    # Verify peer is removed
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "list", "peers"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0
    assert "9541bbb0-a3c0-4b83-8637-96820cae7983" not in result.stdout
    remaining_peers = [line for line in result.stdout.strip().split('\n') if line]
    assert len(remaining_peers) == 2


def test_config_remove_peer_nonexistent(setup_wg_quickrs_folder):
    """Test removing a peer that doesn't exist."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "remove", "peer", "00000000-0000-0000-0000-000000000000"],
        capture_output=True,
        text=True
    )
    assert result.returncode != 0
    assert "not found" in result.stdout.lower() or "error" in result.stdout.lower()


def test_config_remove_this_peer_forbidden(setup_wg_quickrs_folder):
    """Test that removing this_peer is not allowed."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    # this_peer in the test config is 0ed989c6-6dba-4e3c-8034-08adf4262d9e
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "remove", "peer", "0ed989c6-6dba-4e3c-8034-08adf4262d9e"],
        capture_output=True,
        text=True
    )
    assert result.returncode != 0
    assert "cannot remove this_peer" in result.stdout.lower() or "this_peer" in result.stdout.lower()


def test_config_remove_peer_cascades_to_connections(setup_wg_quickrs_folder):
    """Test that removing a peer also removes its connections."""
    setup_wg_quickrs_folder("no_auth_multi_peer")

    # Verify connections exist first
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "list", "connections"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0
    initial_connections = [line for line in result.stdout.strip().split('\n') if line]
    assert len(initial_connections) == 2

    # Remove a peer that has connections (not this_peer)
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "remove", "peer", "9541bbb0-a3c0-4b83-8637-96820cae7983"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0

    # Verify connections involving this peer are removed
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "list", "connections"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0
    # Connections should be reduced since peer was removed
    remaining_connections = [line for line in result.stdout.strip().split('\n') if line and "No connections" not in line]
    assert len(remaining_connections) < len(initial_connections)


def test_config_remove_connection_success(setup_wg_quickrs_folder):
    """Test successfully removing a connection."""
    setup_wg_quickrs_folder("no_auth_multi_peer")

    # Get a connection ID first
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "list", "connections"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0
    connections = [line for line in result.stdout.strip().split('\n') if line]
    assert len(connections) == 2

    # Extract connection ID from format "name1<->name2 (connectionid)"
    # Parse the connection ID from the first line
    first_connection = connections[0]
    # Format is "name1<->name2 (uuid*uuid)"
    connection_id = first_connection[first_connection.rfind('(') + 1:first_connection.rfind(')')]

    # Remove the connection
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "remove", "connection", connection_id],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0

    # Verify connection is removed
    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "list", "connections"],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0
    remaining_connections = [line for line in result.stdout.strip().split('\n') if line]
    assert len(remaining_connections) == 1  # One connection removed


def test_config_remove_connection_nonexistent(setup_wg_quickrs_folder):
    """Test removing a connection that doesn't exist."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "remove", "connection", "00000000-0000-0000-0000-000000000000*00000000-0000-0000-0000-000000000001"],
        capture_output=True,
        text=True
    )
    assert result.returncode != 0
    assert "not found" in result.stdout.lower() or "error" in result.stdout.lower()


@pytest.mark.parametrize(
    "invalid_uuid",
    [
        "invalid-uuid",
    ],
)
def test_config_remove_peer_invalid_uuid(setup_wg_quickrs_folder, invalid_uuid):
    """Test removing a peer with invalid UUID format."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "remove", "peer", invalid_uuid],
        capture_output=True,
        text=True
    )
    assert result.returncode != 0


@pytest.mark.parametrize(
    "invalid_connection_id",
    [
        "invalid-connection-id",
        "00000000-0000-0000-0000-000000000000*invalid-uuid",
        "invalid-uuid*00000000-0000-0000-0000-000000000000",
    ],
)
def test_config_remove_connection_invalid_format(setup_wg_quickrs_folder, invalid_connection_id):
    """Test removing a connection with invalid connection ID format."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "remove", "connection", invalid_connection_id],
        capture_output=True,
        text=True
    )
    assert result.returncode != 0


def test_config_remove_reservation_nonexistent(setup_wg_quickrs_folder):
    """Test removing a reservation that doesn't exist."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "remove", "reservation", "10.0.34.100"],
        capture_output=True,
        text=True
    )
    assert result.returncode != 0
    assert "not found" in result.stdout.lower() or "error" in result.stdout.lower()


@pytest.mark.parametrize(
    "invalid_ip",
    [
        "invalid-ip",
        "999.999.999.999",
        "10.0.34",
        "",
    ],
)
def test_config_remove_reservation_invalid_ip(setup_wg_quickrs_folder, invalid_ip):
    """Test removing a reservation with invalid IPv4 address format."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "remove", "reservation", invalid_ip],
        capture_output=True,
        text=True
    )
    assert result.returncode != 0
