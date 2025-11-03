from typing import Dict, Any
import time
import pathlib
import socket
import os
import subprocess
import platform
import requests


def get_paths():
    pytest_folder = pathlib.Path(__file__).parent.resolve()
    wg_quickrs_config_folder = pytest_folder / ".wg-quickrs-pytest"
    wg_quickrs_config_file = wg_quickrs_config_folder / "conf.yml"
    return pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file


def get_wg_quickrs_command(use_sudo=False):
    pytest_folder, wg_quickrs_config_folder, _ = get_paths()
    command = [
        str(pytest_folder.parent.parent.resolve() / "src/target/release/wg-quickrs"),
        '--wg-quickrs-config-folder',
        str(wg_quickrs_config_folder)
    ]
    if use_sudo:
        command = ["sudo"] + command
    return command


def wait_for_port(host_port, timeout=10.0):
    """Wait until TCP port is open or timeout"""
    start = time.time()
    while time.time() - start < timeout:
        try:
            with socket.create_connection(host_port, timeout=1):
                return True
        except OSError:
            time.sleep(0.1)
    return False


def get_available_firewall_utilities():
    """Get a list of available firewall utilities on the system"""

    candidates = ["iptables", "pfctl"]
    available = []
    
    for prog in candidates:
        if not os.environ.get("PATH"): continue

        for path_dir in os.environ["PATH"].split(os.pathsep):
            full_path = os.path.join(path_dir, prog)
            if os.path.isfile(full_path):
                available.append(full_path)
                break
    
    return available


def get_available_network_interfaces():
    """Get a list of available network interfaces on the system (with IPv4 addresses, non-loopback)"""
    interfaces = []
    
    try:
        if platform.system() == "Darwin":  # macOS
            # Use ifconfig to get interfaces with IPv4 addresses
            result = subprocess.run(
                ["ifconfig"],
                capture_output=True,
                text=True,
                timeout=5
            )
            if result.returncode != 0: return []

            current_iface = None
            for line in result.stdout.split("\n"):
                # Look for the interface name (starts at the beginning of line)
                if line and not line[0].isspace():
                    parts = line.split(":")
                    if len(parts) >= 1:
                        current_iface = parts[0].strip()
                # Look for inet (IPv4) address
                elif current_iface and "inet " in line and "127.0.0.1" not in line:
                    # Found an interface with a non-loopback IPv4 address
                    if current_iface not in interfaces:
                        interfaces.append(current_iface)
        
        elif platform.system() == "Linux":
            # Use ip addr to get interfaces with IPv4 addresses
            result = subprocess.run(
                ["ip", "-4", "-o", "addr", "show"],
                capture_output=True,
                text=True,
                timeout=5
            )
            if result.returncode != 0: return []

            for line in result.stdout.split("\n"):
                if line.strip():
                    parts = line.split()
                    if len(parts) >= 2:
                        iface = parts[1].strip()
                        # Skip loopback
                        if iface and iface != "lo" and "127.0.0.1" not in line:
                            if iface not in interfaces:
                                interfaces.append(iface)
    except Exception as e:
        print(f"Error getting interfaces: {e}")
    
    return interfaces


def get_test_peer_data() -> Dict[str, Any]:
    """Get test peer data for adding peers."""
    return {
        "name": "test-peer",
        "address": "127.0.0.1",
        "endpoint": {
            "enabled": True,
            "address": { "ipv4_and_port": {"ipv4": "192.168.1.100", "port": 51820} },
        },
        "kind": "laptop",
        "icon": {
            "enabled": False,
            "src": ""
        },
        "dns": {
            "enabled": True,
            "addresses": ["1.1.1.1"]
        },
        "mtu": {
            "enabled": False,
            "value": 1420
        },
        "scripts": {
            "pre_up": [],
            "post_up": [],
            "pre_down": [],
            "post_down": []
        },
        "private_key": "cL+YuwGKNS8bNnPUVdnGDp7jF5BZs1vp1UxK2Xv+JX0="
    }


def get_test_connection_data() -> Dict[str, Any]:
    """Get test connection data for adding connections."""
    return {
        "enabled": True,
        "pre_shared_key": "QjF2m3eEcOuBjVqE1K5yB6z9Tf1Hk8qW2aXvNc5uE0o=",
        "allowed_ips_a_to_b": ["0.0.0.0/0"],
        "allowed_ips_b_to_a": ["10.0.34.0/24"],
        "persistent_keepalive": {
            "enabled": True,
            "period": 25
        }
    }


def get_this_peer_id(base_url: str) -> str:
    """Helper to get this peer ID from summary."""
    response = requests.get(f"{base_url}/api/network/summary?only_digest=false")
    assert response.status_code == 200
    return response.json()["network"]["this_peer"]

def deep_get(d, keys):
    for k in keys:
        if isinstance(d, dict) and k in d:
            d = d[k]
        else:
            return None
    return d