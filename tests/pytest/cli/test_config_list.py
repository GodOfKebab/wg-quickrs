import subprocess
import pytest
from tests.pytest.helpers import get_wg_quickrs_command
from tests.pytest.conftest import setup_wg_quickrs_folder


def test_config_list_peers_single_peer(setup_wg_quickrs_folder):
    """Test listing a single peer with endpoint enabled."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "list", "peers"],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode == 0

    # Verify expected output format: "name (peerid) @ address / {endpoint if enabled}"
    output_lines = [line for line in result.stdout.strip().split('\n') if line]
    assert len(output_lines) == 1

    # Parse the output
    line = output_lines[0]
    assert "wg-quickrs-host" in line
    assert "0ed989c6-6dba-4e3c-8034-08adf4262d9e" in line
    assert "10.0.34.1" in line
    # Endpoint should be present since it's enabled in the config
    assert "127.0.0.1:51820" in line
    assert " / " in line  # Separator between address and endpoint


@pytest.mark.parametrize(
    "expected_substring",
    [
        "wg-quickrs-host",  # Peer name
        "0ed989c6-6dba-4e3c-8034-08adf4262d9e",  # Peer ID
        "10.0.34.1",  # Address
        "127.0.0.1:51820",  # Endpoint
    ],
)
def test_config_list_peers_contains_expected_data(setup_wg_quickrs_folder, expected_substring):
    """Test that peer listing contains all expected data elements."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "list", "peers"],
        capture_output=True,
        text=True
    )

    assert result.returncode == 0
    assert expected_substring in result.stdout


def test_config_list_connections_empty(setup_wg_quickrs_folder):
    """Test listing connections when there are none."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "list", "connections"],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode == 0
    assert "No connections found" in result.stdout


def test_config_list_with_different_config(setup_wg_quickrs_folder):
    """Test list commands with a different configuration file."""
    setup_wg_quickrs_folder("no_auth_multi_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "list", "peers"],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode == 0
    output_lines = [line for line in result.stdout.strip().split('\n') if line]
    assert len(output_lines) == 3

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "list", "connections"],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode == 0
    output_lines = [line for line in result.stdout.strip().split('\n') if line]
    assert len(output_lines) == 2


def test_config_list_reservations_empty(setup_wg_quickrs_folder):
    """Test listing reservations when there are none."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "list", "reservations"],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode == 0
    assert "No reservations found" in result.stdout

# TODO: add tests for reservations
