import subprocess
from tests.pytest.conftest import setup_wg_quickrs_folder
from tests.pytest.helpers import get_wg_quickrs_command, get_paths


def test_config_conf_stdout(setup_wg_quickrs_folder):
    """Test that config conf generates WireGuard config to stdout."""
    setup_wg_quickrs_folder("no_auth_multi_peer")

    # Use the this_peer from the config
    peer_id = "0ed989c6-6dba-4e3c-8034-08adf4262d9e"

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "conf", peer_id],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode == 0
    # Check that the output contains WireGuard config sections
    assert "[Interface]" in result.stdout
    assert "PrivateKey" in result.stdout
    assert "Address" in result.stdout
    assert "ListenPort" in result.stdout
    assert "DNS" in result.stdout
    # Check for peer sections
    assert "[Peer]" in result.stdout
    assert "PublicKey" in result.stdout
    assert "PresharedKey" in result.stdout
    assert "AllowedIPs" in result.stdout
    assert "PersistentKeepalive" in result.stdout


def test_config_conf_stripped(setup_wg_quickrs_folder):
    """Test that config conf --stripped generates minimal WireGuard config."""
    setup_wg_quickrs_folder("no_auth_multi_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    peer_id = "0ed989c6-6dba-4e3c-8034-08adf4262d9e"

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "conf", peer_id, "--stripped"],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode == 0
    # Address, DNS, MTU should not be present in stripped mode
    assert "Address" not in result.stdout
    assert "DNS" not in result.stdout
    assert "MTU" not in result.stdout
    # Scripts should not be present in stripped mode
    assert "PreUp" not in result.stdout
    assert "PostUp" not in result.stdout
    assert "PreDown" not in result.stdout
    assert "PostDown" not in result.stdout


def test_config_conf_to_file(setup_wg_quickrs_folder):
    """Test that config conf -o writes to a file."""
    setup_wg_quickrs_folder("no_auth_multi_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    peer_id = "0ed989c6-6dba-4e3c-8034-08adf4262d9e"
    output_file = wg_quickrs_config_folder / "test.conf"

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "conf", peer_id, "-o", str(output_file)],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode == 0
    # Stdout should be empty when writing to file
    assert result.stdout == ""
    # Check that the file was created
    assert output_file.exists()
    # Read the file and verify contents
    with open(output_file, 'r') as f:
        contents = f.read()
        assert "[Interface]" in contents
        assert "[Peer]" in contents


def test_config_conf_invalid_peer_id(setup_wg_quickrs_folder):
    """Test that config conf fails gracefully with invalid peer ID."""
    setup_wg_quickrs_folder("no_auth_multi_peer")

    # Use a peer ID that doesn't exist
    peer_id = "00000000-0000-0000-0000-000000000000"

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "conf", peer_id],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    # Should fail with non-zero exit code
    assert result.returncode != 0
    # Should contain error message (error goes to stdout)
    assert "peer" in result.stdout.lower() or "not found" in result.stdout.lower()


def test_config_conf_with_amnezia_enabled(setup_wg_quickrs_folder):
    """Test that config conf includes Amnezia parameters when enabled."""
    setup_wg_quickrs_folder("no_auth_multi_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    peer_id = "0ed989c6-6dba-4e3c-8034-08adf4262d9e"

    # First enable amnezia parameters via subprocess
    # We need to use the awg binary path for this to work
    awg_path = wg_quickrs_config_folder / "bin" / "awg"
    subprocess.run(
        get_wg_quickrs_command() + ["config", "set", "agent", "vpn", "wg", str(awg_path)],
        capture_output=True,
        text=True
    )
    subprocess.run(
        get_wg_quickrs_command() + ["config", "enable", "network", "amnezia-parameters"],
        capture_output=True,
        text=True
    )

    result = subprocess.run(
        get_wg_quickrs_command() + ["config", "conf", peer_id],
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)

    assert result.returncode == 0
    # Check for Amnezia parameters
    assert "Jc" in result.stdout
    assert "Jmin" in result.stdout
    assert "Jmax" in result.stdout
    assert "S1" in result.stdout
    assert "S2" in result.stdout
    assert "H1" in result.stdout
    assert "H2" in result.stdout
    assert "H3" in result.stdout
    assert "H4" in result.stdout
