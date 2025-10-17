import time
import pathlib
import socket
import os
import subprocess
import platform


def get_paths():
    pytest_folder = pathlib.Path(__file__).parent.resolve()
    wg_quickrs_config_folder = pytest_folder / ".wg-quickrs-pytest"
    wg_quickrs_config_file = wg_quickrs_config_folder / "conf.yml"
    return pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file


def get_wg_quickrs_command():
    pytest_folder, wg_quickrs_config_folder, _ = get_paths()
    return [
        str(pytest_folder.parent.parent.resolve() / "src/target/release/wg-quickrs"),
        '--wg-quickrs-config-folder',
        str(wg_quickrs_config_folder)
    ]


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
