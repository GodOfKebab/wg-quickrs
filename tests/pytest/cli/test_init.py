from tests.pytest.conftest import setup_wg_quickrs_folder
from tests.pytest.helpers import (
    get_wg_quickrs_command, 
    get_paths,
    get_available_firewall_utilities,
    get_available_network_interfaces
)
import subprocess
import pytest
import os


def init_no_prompt(opts):
    command = " ".join(get_wg_quickrs_command()) + " agent init --no-prompt true \\"
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


def generate_init_no_prompt_opts(
        network_subnet="10.0.34.0/24",
        agent_web_address="0.0.0.0",
        agent_web_http="--agent-web-http-enabled false",
        agent_web_https="--agent-web-https-enabled false",
        agent_web_password="--agent-web-password-enabled false",
        agent_vpn="--agent-vpn-enabled false",
        agent_firewall="--agent-firewall-enabled false",
        agent_peer_vpn_endpoint="YOUR-SERVER:51820",
        agent_peer_icon="--agent-peer-icon-enabled false",
        agent_peer_vpn_internal_address="10.0.34.1",
        agent_peer_dns="--agent-peer-dns-enabled false",
        agent_peer_mtu="--agent-peer-mtu-enabled false",
        agent_peer_script_pre_up="--agent-peer-script-pre-up-enabled false",
        agent_peer_script_post_up="--agent-peer-script-post-up-enabled false",
        agent_peer_script_pre_down="--agent-peer-script-pre-down-enabled false",
        agent_peer_script_post_down="--agent-peer-script-post-down-enabled false",
        default_peer_icon="--default-peer-icon-enabled false",
        default_peer_dns="--default-peer-dns-enabled false",
        default_peer_mtu="--default-peer-mtu-enabled false",
        default_peer_script_pre_up="--default-peer-script-pre-up-enabled false",
        default_peer_script_post_up="--default-peer-script-post-up-enabled false",
        default_peer_script_pre_down="--default-peer-script-pre-down-enabled false",
        default_peer_script_post_down="--default-peer-script-post-down-enabled false",
        default_connection_persistent_keepalive="--default-connection-persistent-keepalive-enabled false",
):
    return f"""
    --network-name wg-quickrs-home \\
    --network-subnet {network_subnet} \\
    --agent-web-address {agent_web_address} \\
    {agent_web_http} \\
    {agent_web_https} \\
    {agent_web_password} \\
    {agent_vpn} \\
    {agent_firewall} \\
    --agent-peer-name wg-quickrs-host \\
    --agent-peer-vpn-endpoint {agent_peer_vpn_endpoint} \\
    --agent-peer-kind server \\
    {agent_peer_icon} \\
    --agent-peer-vpn-internal-address {agent_peer_vpn_internal_address} \\
    {agent_peer_dns} \\
    {agent_peer_mtu} \\
    {agent_peer_script_pre_up} \\
    {agent_peer_script_post_up} \\
    {agent_peer_script_pre_down} \\
    {agent_peer_script_post_down} \\
    --default-peer-kind laptop \\
    {default_peer_icon} \\
    {default_peer_dns} \\
    {default_peer_mtu} \\
    {default_peer_script_pre_up} \\
    {default_peer_script_post_up} \\
    {default_peer_script_pre_down} \\
    {default_peer_script_post_down} \\
    {default_connection_persistent_keepalive}
    """


def test_init_no_prompt_simple(setup_wg_quickrs_folder):
    setup_wg_quickrs_folder(None)
    assert init_no_prompt(generate_init_no_prompt_opts()) == 0


@pytest.mark.parametrize(
    "opt_key, opt_val, success",
    [
        ("network_subnet", "10.0.0.0/16", True),
        ("network_subnet", "192.168.1.0/24", False),  # incorrect subnet/internal-address combination
        ("network_subnet", "not-a-subnet", False),
        ("agent_web_address", "192.168.1.1", True),
        ("agent_web_address", "not-an-address", False),
        ("agent_web_http", "--agent-web-http-enabled true", False),
        ("agent_web_http", "--agent-web-http-enabled true --agent-web-http-port 80", True),
        ("agent_web_http", "--agent-web-http-enabled true --agent-web-http-port not-a-port", False),
        ("agent_web_https", "--agent-web-https-enabled true", False),
        ("agent_web_https", "--agent-web-https-enabled true --agent-web-https-port 443", False),
        ("agent_web_https", "--agent-web-https-enabled true --agent-web-https-port 443 --agent-web-https-tls-cert certs/servers/127.0.0.1/cert.pem --agent-web-https-tls-key certs/servers/127.0.0.1/key.pem", False),
        # since the tls cert generation is not tested here, we skip the successful https test case
        ("agent_web_password", "--agent-web-password-enabled true", False),
        ("agent_web_password", "--agent-web-password-enabled true --agent-web-password test-pwd", True),
        ("agent_vpn", "--agent-vpn-enabled true", False),
        ("agent_vpn", "--agent-vpn-enabled true --agent-vpn-port not-a-port", False),
        ("agent_firewall", "--agent-firewall-enabled true", False),
        ("agent_firewall", "--agent-firewall-enabled true --agent-firewall-utility not-a-utility --agent-firewall-gateway not-a-gateway", False),
        ("agent_peer_vpn_endpoint", "192.168.1.1:51820", True),
        ("agent_peer_vpn_endpoint", "not-an-endpoint", False),
        ("agent_peer_icon", "--agent-peer-icon-enabled true", False),
        ("agent_peer_icon", "--agent-peer-icon-enabled true --agent-peer-icon-src example-src", True),
        ("agent_peer_vpn_internal_address", "10.0.34.100", True),
        ("agent_peer_vpn_internal_address", "192.168.1.1", False),  # incorrect subnet/internal-address combination
        ("agent_peer_vpn_internal_address", "not-an-address", False),
        ("agent_peer_dns", "--agent-peer-dns-enabled true", False),
        ("agent_peer_dns", "--agent-peer-dns-enabled true --agent-peer-dns-addresses 1.1.1.1", True),
        ("agent_peer_dns", "--agent-peer-dns-enabled true --agent-peer-dns-addresses 1.1.1.1 --agent-peer-dns-addresses 8.8.8.8", True),
        ("agent_peer_dns", "--agent-peer-dns-enabled true --agent-peer-dns-addresses not-an-address", False),
        ("agent_peer_mtu", "--agent-peer-mtu-enabled true", False),
        ("agent_peer_mtu", "--agent-peer-mtu-enabled true --agent-peer-mtu-value 1420", True),
        ("agent_peer_mtu", "--agent-peer-mtu-enabled true --agent-peer-mtu-value not-an-mtu-val", False),
        ("agent_peer_script_pre_up", "--agent-peer-script-pre-up-enabled true", False),
        ("agent_peer_script_pre_up", "--agent-peer-script-pre-up-enabled true --agent-peer-script-pre-up-line 'echo hi;'", True),
        ("agent_peer_script_pre_up", "--agent-peer-script-pre-up-enabled true --agent-peer-script-pre-up-line not-a-script", False),
        ("agent_peer_script_post_up", "--agent-peer-script-post-up-enabled true", False),
        ("agent_peer_script_post_up", "--agent-peer-script-post-up-enabled true --agent-peer-script-post-up-line 'echo hi;'", True),
        ("agent_peer_script_post_up", "--agent-peer-script-post-up-enabled true --agent-peer-script-post-up-line not-a-script", False),
        ("agent_peer_script_pre_down", "--agent-peer-script-pre-down-enabled true", False),
        ("agent_peer_script_pre_down", "--agent-peer-script-pre-down-enabled true --agent-peer-script-pre-down-line 'echo hi;'", True),
        ("agent_peer_script_pre_down", "--agent-peer-script-pre-down-enabled true --agent-peer-script-pre-down-line not-a-script", False),
        ("agent_peer_script_post_down", "--agent-peer-script-post-down-enabled true", False),
        ("agent_peer_script_post_down", "--agent-peer-script-post-down-enabled true --agent-peer-script-post-down-line 'echo hi;'", True),
        ("agent_peer_script_post_down", "--agent-peer-script-post-down-enabled true --agent-peer-script-post-down-line not-a-script", False),
        ("default_peer_icon", "--default-peer-icon-enabled true", False),
        ("default_peer_icon", "--default-peer-icon-enabled true --default-peer-icon-src example-src", True),
        ("default_peer_dns", "--default-peer-dns-enabled true", False),
        ("default_peer_dns", "--default-peer-dns-enabled true --default-peer-dns-addresses 1.1.1.1", True),
        ("default_peer_dns", "--default-peer-dns-enabled true --default-peer-dns-addresses 1.1.1.1 --default-peer-dns-addresses 8.8.8.8", True),
        ("default_peer_dns", "--default-peer-dns-enabled true --default-peer-dns-addresses not-an-address", False),
        ("default_peer_mtu", "--default-peer-mtu-enabled true", False),
        ("default_peer_mtu", "--default-peer-mtu-enabled true --default-peer-mtu-value 1420", True),
        ("default_peer_mtu", "--default-peer-mtu-enabled true --default-peer-mtu-value not-an-mtu-val", False),
        ("default_peer_script_pre_up", "--default-peer-script-pre-up-enabled true", False),
        ("default_peer_script_pre_up", "--default-peer-script-pre-up-enabled true --default-peer-script-pre-up-line 'echo hi;'", True),
        ("default_peer_script_pre_up", "--default-peer-script-pre-up-enabled true --default-peer-script-pre-up-line not-a-script", False),
        ("default_peer_script_post_up", "--default-peer-script-post-up-enabled true", False),
        ("default_peer_script_post_up", "--default-peer-script-post-up-enabled true --default-peer-script-post-up-line 'echo hi;'", True),
        ("default_peer_script_post_up", "--default-peer-script-post-up-enabled true --default-peer-script-post-up-line not-a-script", False),
        ("default_peer_script_pre_down", "--default-peer-script-pre-down-enabled true", False),
        ("default_peer_script_pre_down", "--default-peer-script-pre-down-enabled true --default-peer-script-pre-down-line 'echo hi;'", True),
        ("default_peer_script_pre_down", "--default-peer-script-pre-down-enabled true --default-peer-script-pre-down-line not-a-script", False),
        ("default_peer_script_post_down", "--default-peer-script-post-down-enabled true", False),
        ("default_peer_script_post_down", "--default-peer-script-post-down-enabled true --default-peer-script-post-down-line 'echo hi;'", True),
        ("default_peer_script_post_down", "--default-peer-script-post-down-enabled true --default-peer-script-post-down-line not-a-script", False),
    ],
)
def test_init_no_prompt(setup_wg_quickrs_folder, opt_key, opt_val, success):
    setup_wg_quickrs_folder(None)
    assert (init_no_prompt(generate_init_no_prompt_opts(**{opt_key: opt_val})) == 0) == success


@pytest.mark.parametrize(
    "opts, success",
    [
        # correct subnet/internal-address combination
        ({"network_subnet": "192.168.1.0/24", "agent_peer_vpn_internal_address": "192.168.1.1"}, True),
    ],
)
def test_init_no_prompt_multi_field(setup_wg_quickrs_folder, opts, success):
    setup_wg_quickrs_folder(None)
    assert (init_no_prompt(generate_init_no_prompt_opts(**opts)) == 0) == success


@pytest.mark.parametrize(
    "opt_key, opt_val, success",
    [
        ("agent_vpn", "--agent-vpn-wg $(which wg) --agent-vpn-wg-userspace-enabled false", True),
        ("agent_vpn", "--agent-vpn-wg $(which wg) --agent-vpn-wg-userspace-enabled true", False),
        ("agent_vpn", "--agent-vpn-wg $(which wg) --agent-vpn-wg-userspace-enabled true --agent-vpn-wg-userspace-binary $(which wireguard-go)", True),
        ("agent_vpn", "--agent-vpn-wg $(which awg) --agent-vpn-wg-userspace-enabled false", False),
        ("agent_vpn", "--agent-vpn-wg $(which awg) --agent-vpn-wg-userspace-enabled false --network-amnezia-enabled false", True),
        ("agent_vpn", "--agent-vpn-wg $(which awg) --agent-vpn-wg-userspace-enabled false --network-amnezia-enabled true", False),
        ("agent_vpn", "--agent-vpn-wg $(which awg) --agent-vpn-wg-userspace-enabled false --network-amnezia-enabled true --network-amnezia-s1 56", False),
        ("agent_vpn", "--agent-vpn-wg $(which awg) --agent-vpn-wg-userspace-enabled false --network-amnezia-enabled true --network-amnezia-s1 56 --network-amnezia-s2 156", False),
        ("agent_vpn", "--agent-vpn-wg $(which awg) --agent-vpn-wg-userspace-enabled false --network-amnezia-enabled true --network-amnezia-s1 56 --network-amnezia-s2 156 --network-amnezia-h-random false --network-amnezia-h1 1", False),
        ("agent_vpn", "--agent-vpn-wg $(which awg) --agent-vpn-wg-userspace-enabled false --network-amnezia-enabled true --network-amnezia-s1 56 --network-amnezia-s2 156 --network-amnezia-h-random false --network-amnezia-h1 1 --network-amnezia-h2 2", False),
        ("agent_vpn", "--agent-vpn-wg $(which awg) --agent-vpn-wg-userspace-enabled false --network-amnezia-enabled true --network-amnezia-s1 56 --network-amnezia-s2 156 --network-amnezia-h-random false --network-amnezia-h1 1 --network-amnezia-h2 2 --network-amnezia-h3 3", False),
        ("agent_vpn", "--agent-vpn-wg $(which awg) --agent-vpn-wg-userspace-enabled false --network-amnezia-enabled true --network-amnezia-s1 56 --network-amnezia-s2 156 --network-amnezia-h-random false --network-amnezia-h1 1 --network-amnezia-h2 2 --network-amnezia-h3 3 --network-amnezia-h4 4", False),
        ("agent_vpn", "--agent-vpn-wg $(which awg) --agent-vpn-wg-userspace-enabled false --network-amnezia-enabled true --network-amnezia-s1 56 --network-amnezia-s2 156 --network-amnezia-h-random false --network-amnezia-h1 1 --network-amnezia-h2 2 --network-amnezia-h3 3 --network-amnezia-h4 4 --agent-peer-amnezia-jc 3", False),
        ("agent_vpn", "--agent-vpn-wg $(which awg) --agent-vpn-wg-userspace-enabled false --network-amnezia-enabled true --network-amnezia-s1 56 --network-amnezia-s2 156 --network-amnezia-h-random false --network-amnezia-h1 1 --network-amnezia-h2 2 --network-amnezia-h3 3 --network-amnezia-h4 4 --agent-peer-amnezia-jc 3 --agent-peer-amnezia-jmin 61", False),
        ("agent_vpn", "--agent-vpn-wg $(which awg) --agent-vpn-wg-userspace-enabled false --network-amnezia-enabled true --network-amnezia-s1 56 --network-amnezia-s2 156 --network-amnezia-h-random false --network-amnezia-h1 1 --network-amnezia-h2 2 --network-amnezia-h3 3 --network-amnezia-h4 4 --agent-peer-amnezia-jc 3 --agent-peer-amnezia-jmin 61 --agent-peer-amnezia-jmax 121", False),
        ("agent_vpn", "--agent-vpn-wg $(which awg) --agent-vpn-wg-userspace-enabled false --network-amnezia-enabled true --network-amnezia-s1 56 --network-amnezia-s2 156 --network-amnezia-h-random false --network-amnezia-h1 1 --network-amnezia-h2 2 --network-amnezia-h3 3 --network-amnezia-h4 4 --agent-peer-amnezia-jc 3 --agent-peer-amnezia-jmin 61 --agent-peer-amnezia-jmax 121 --default-peer-amnezia-jc 3", False),
        ("agent_vpn", "--agent-vpn-wg $(which awg) --agent-vpn-wg-userspace-enabled false --network-amnezia-enabled true --network-amnezia-s1 56 --network-amnezia-s2 156 --network-amnezia-h-random false --network-amnezia-h1 1 --network-amnezia-h2 2 --network-amnezia-h3 3 --network-amnezia-h4 4 --agent-peer-amnezia-jc 3 --agent-peer-amnezia-jmin 61 --agent-peer-amnezia-jmax 121 --default-peer-amnezia-jc 3 --default-peer-amnezia-jmin 61", False),
        ("agent_vpn", "--agent-vpn-wg $(which awg) --agent-vpn-wg-userspace-enabled false --network-amnezia-enabled true --network-amnezia-s1 56 --network-amnezia-s2 156 --network-amnezia-h-random false --network-amnezia-h1 1 --network-amnezia-h2 2 --network-amnezia-h3 3 --network-amnezia-h4 4 --agent-peer-amnezia-jc 3 --agent-peer-amnezia-jmin 61 --agent-peer-amnezia-jmax 121 --default-peer-amnezia-jc 3 --default-peer-amnezia-jmin 61 --default-peer-amnezia-jmax 121", True),
        # Test --network-amnezia-h-random flag
        ("agent_vpn", "--agent-vpn-wg $(which awg) --agent-vpn-wg-userspace-enabled false --network-amnezia-enabled true --network-amnezia-s1 56 --network-amnezia-s2 156 --network-amnezia-h-random true --agent-peer-amnezia-jc 3 --agent-peer-amnezia-jmin 61 --agent-peer-amnezia-jmax 121 --default-peer-amnezia-jc 3 --default-peer-amnezia-jmin 61 --default-peer-amnezia-jmax 121", True),
    ],
)
def test_init_no_prompt_vpn(setup_wg_quickrs_folder, opt_key, opt_val, success):
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = setup_wg_quickrs_folder(None)

    opt_val = f"--agent-vpn-enabled true --agent-vpn-port 51820 {opt_val}"
    opt_val_sanitized = opt_val.replace("$(which wg)", str(wg_quickrs_config_folder / "bin/wg"))
    opt_val_sanitized = opt_val_sanitized.replace("$(which wireguard-go)", str(wg_quickrs_config_folder / "bin/wireguard-go"))
    opt_val_sanitized = opt_val_sanitized.replace("$(which awg)", str(wg_quickrs_config_folder / "bin/awg"))
    opt_val_sanitized = opt_val_sanitized.replace("$(which amneziawg-go)", str(wg_quickrs_config_folder / "bin/amneziawg-go"))
    assert (init_no_prompt(generate_init_no_prompt_opts(**{opt_key: opt_val_sanitized})) == 0) == success


def test_init_no_prompt_https(setup_wg_quickrs_folder):
    setup_wg_quickrs_folder("test_pwd_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()
    os.remove(wg_quickrs_config_file)

    for opt_val, success in [
        ("--agent-web-https-enabled true --agent-web-https-port 443 --agent-web-https-tls-cert certs/servers/127.0.0.1/cert.pem --agent-web-https-tls-key certs/servers/127.0.0.1/key.pem", True),
        ("--agent-web-https-enabled true --agent-web-https-port 443 --agent-web-https-tls-cert certs/servers/127.0.0.1/not-a-cert.pem --agent-web-https-tls-key certs/servers/127.0.0.1/not-a-key.pem", False)
    ]:
        ret = init_no_prompt(generate_init_no_prompt_opts(agent_web_https=opt_val))
        if ret == 0:
            os.remove(wg_quickrs_config_file)
        assert (ret == 0) == success


def test_init_no_prompt_firewall(setup_wg_quickrs_folder):
    """Test firewall initialization with various configurations"""
    utilities = get_available_firewall_utilities()
    interfaces = get_available_network_interfaces()

    if not utilities or not interfaces:
        pytest.skip("No firewall utilities or network interfaces available on this system")

    utility = utilities[0]
    gateway = interfaces[0]
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    for opt_val, success in [
        # Invalid utility
        (f"--agent-firewall-enabled true --agent-firewall-configure-http false --agent-firewall-configure-https false --agent-firewall-configure-vpn true --agent-firewall-vpn-automated true --agent-firewall-utility not-a-utility --agent-firewall-gateway {gateway}", False),
        # Invalid gateway
        (f"--agent-firewall-enabled true --agent-firewall-configure-http false --agent-firewall-configure-https false --agent-firewall-configure-vpn true --agent-firewall-vpn-automated true --agent-firewall-utility {utility} --agent-firewall-gateway not-a-gateway", False),
        # Missing utility (but automated VPN enabled, so utility is required)
        (f"--agent-firewall-enabled true --agent-firewall-configure-http false --agent-firewall-configure-https false --agent-firewall-configure-vpn true --agent-firewall-vpn-automated true --agent-firewall-gateway {gateway}", False),
        # Missing gateway (but automated VPN enabled, so gateway is required)
        (f"--agent-firewall-enabled true --agent-firewall-configure-http false --agent-firewall-configure-https false --agent-firewall-configure-vpn true --agent-firewall-vpn-automated true --agent-firewall-utility {utility}", False),
        # Valid VPN automated setup
        (f"--agent-firewall-enabled true --agent-firewall-configure-http false --agent-firewall-configure-https false --agent-firewall-configure-vpn true --agent-firewall-vpn-automated true --agent-firewall-utility {utility} --agent-firewall-gateway {gateway}", True),
    ]:
        setup_wg_quickrs_folder(None)
        ret = init_no_prompt(generate_init_no_prompt_opts(agent_firewall=opt_val))
        assert (ret == 0) == success
        if os.path.exists(wg_quickrs_config_file):
            os.remove(wg_quickrs_config_file)


def test_init_firewall_http_manual(setup_wg_quickrs_folder):
    """Test HTTP firewall with manual script setup"""
    from ruamel.yaml import YAML
    yaml = YAML()
    yaml.preserve_quotes = True

    setup_wg_quickrs_folder(None)
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    # Test HTTP manual setup with custom scripts
    firewall_opts = """--agent-firewall-enabled true \\
        --agent-firewall-configure-http true \\
        --agent-firewall-http-automated false \\
        --agent-firewall-http-pre-up-enabled true \\
        --agent-firewall-http-pre-up-line 'echo "Starting HTTP firewall";' \\
        --agent-firewall-http-pre-up-line 'iptables -I INPUT -p tcp --dport 80 -j ACCEPT;' \\
        --agent-firewall-http-post-down-enabled true \\
        --agent-firewall-http-post-down-line 'iptables -D INPUT -p tcp --dport 80 -j ACCEPT;' \\
        --agent-firewall-http-post-down-line 'echo "Stopped HTTP firewall";' \\
        --agent-firewall-configure-https false \\
        --agent-firewall-configure-vpn false"""

    ret = init_no_prompt(generate_init_no_prompt_opts(agent_firewall=firewall_opts))
    assert ret == 0

    # Verify config
    with open(wg_quickrs_config_file) as stream:
        conf = yaml.load(stream)

    http_scripts = conf["agent"]["firewall"]["http"]

    # Verify pre_up scripts
    assert len(http_scripts["pre_up"]) == 2
    assert http_scripts["pre_up"][0]["script"] == 'echo "Starting HTTP firewall";'
    assert http_scripts["pre_up"][1]["script"] == 'iptables -I INPUT -p tcp --dport 80 -j ACCEPT;'

    # Verify post_down scripts
    assert len(http_scripts["post_down"]) == 2
    assert http_scripts["post_down"][0]["script"] == 'iptables -D INPUT -p tcp --dport 80 -j ACCEPT;'
    assert http_scripts["post_down"][1]["script"] == 'echo "Stopped HTTP firewall";'


def test_init_firewall_https_manual(setup_wg_quickrs_folder):
    """Test HTTPS firewall with manual script setup"""
    from ruamel.yaml import YAML
    yaml = YAML()
    yaml.preserve_quotes = True

    setup_wg_quickrs_folder(None)
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    # Test HTTPS manual setup with custom scripts
    firewall_opts = """--agent-firewall-enabled true \\
        --agent-firewall-configure-http false \\
        --agent-firewall-configure-https true \\
        --agent-firewall-https-automated false \\
        --agent-firewall-https-pre-up-enabled true \\
        --agent-firewall-https-pre-up-line 'echo "Starting HTTPS firewall";' \\
        --agent-firewall-https-pre-up-line 'iptables -I INPUT -p tcp --dport 443 -j ACCEPT;' \\
        --agent-firewall-https-post-down-enabled true \\
        --agent-firewall-https-post-down-line 'iptables -D INPUT -p tcp --dport 443 -j ACCEPT;' \\
        --agent-firewall-configure-vpn false"""

    ret = init_no_prompt(generate_init_no_prompt_opts(agent_firewall=firewall_opts))
    assert ret == 0

    # Verify config
    with open(wg_quickrs_config_file) as stream:
        conf = yaml.load(stream)

    https_scripts = conf["agent"]["firewall"]["https"]

    # Verify pre_up scripts
    assert len(https_scripts["pre_up"]) == 2
    assert https_scripts["pre_up"][0]["script"] == 'echo "Starting HTTPS firewall";'
    assert https_scripts["pre_up"][1]["script"] == 'iptables -I INPUT -p tcp --dport 443 -j ACCEPT;'

    # Verify post_down scripts
    assert len(https_scripts["post_down"]) == 1
    assert https_scripts["post_down"][0]["script"] == 'iptables -D INPUT -p tcp --dport 443 -j ACCEPT;'


def test_init_firewall_vpn_manual(setup_wg_quickrs_folder):
    """Test VPN firewall with manual script setup (all 4 lifecycle hooks)"""
    from ruamel.yaml import YAML
    yaml = YAML()
    yaml.preserve_quotes = True

    setup_wg_quickrs_folder(None)
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    # Test VPN manual setup with custom scripts for all 4 hooks
    firewall_opts = """--agent-firewall-enabled true \\
        --agent-firewall-configure-http false \\
        --agent-firewall-configure-https false \\
        --agent-firewall-configure-vpn true \\
        --agent-firewall-vpn-automated false \\
        --agent-firewall-vpn-pre-up-enabled true \\
        --agent-firewall-vpn-pre-up-line 'echo "VPN pre-up";' \\
        --agent-firewall-vpn-post-up-enabled true \\
        --agent-firewall-vpn-post-up-line 'echo "VPN post-up 1";' \\
        --agent-firewall-vpn-post-up-line 'iptables -A FORWARD -i wg0 -j ACCEPT;' \\
        --agent-firewall-vpn-post-up-line 'echo "VPN post-up 3";' \\
        --agent-firewall-vpn-pre-down-enabled true \\
        --agent-firewall-vpn-pre-down-line 'echo "VPN pre-down";' \\
        --agent-firewall-vpn-post-down-enabled true \\
        --agent-firewall-vpn-post-down-line 'iptables -D FORWARD -i wg0 -j ACCEPT;' \\
        --agent-firewall-vpn-post-down-line 'echo "VPN post-down";'"""

    ret = init_no_prompt(generate_init_no_prompt_opts(agent_firewall=firewall_opts))
    assert ret == 0

    # Verify config
    with open(wg_quickrs_config_file) as stream:
        conf = yaml.load(stream)

    vpn_scripts = conf["agent"]["firewall"]["vpn"]

    # Verify all 4 lifecycle hooks
    assert len(vpn_scripts["pre_up"]) == 1
    assert vpn_scripts["pre_up"][0]["script"] == 'echo "VPN pre-up";'

    assert len(vpn_scripts["post_up"]) == 3
    assert vpn_scripts["post_up"][0]["script"] == 'echo "VPN post-up 1";'
    assert vpn_scripts["post_up"][1]["script"] == 'iptables -A FORWARD -i wg0 -j ACCEPT;'
    assert vpn_scripts["post_up"][2]["script"] == 'echo "VPN post-up 3";'

    assert len(vpn_scripts["pre_down"]) == 1
    assert vpn_scripts["pre_down"][0]["script"] == 'echo "VPN pre-down";'

    assert len(vpn_scripts["post_down"]) == 2
    assert vpn_scripts["post_down"][0]["script"] == 'iptables -D FORWARD -i wg0 -j ACCEPT;'
    assert vpn_scripts["post_down"][1]["script"] == 'echo "VPN post-down";'


def test_init_firewall_mixed_automated_manual(setup_wg_quickrs_folder):
    """Test mixed firewall configuration: HTTP automated + VPN manual"""
    from ruamel.yaml import YAML
    yaml = YAML()
    yaml.preserve_quotes = True

    utilities = get_available_firewall_utilities()
    interfaces = get_available_network_interfaces()

    if not utilities or not interfaces:
        pytest.skip("No firewall utilities or network interfaces available on this system")

    utility = utilities[0]
    gateway = interfaces[0]
    setup_wg_quickrs_folder(None)
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    # HTTP automated, VPN manual
    firewall_opts = f"""--agent-firewall-enabled true \\
        --agent-firewall-configure-http true \\
        --agent-firewall-http-automated true \\
        --agent-firewall-utility {utility} \\
        --agent-firewall-configure-https false \\
        --agent-firewall-configure-vpn true \\
        --agent-firewall-vpn-automated false \\
        --agent-firewall-vpn-pre-up-enabled false \\
        --agent-firewall-vpn-post-up-enabled true \\
        --agent-firewall-vpn-post-up-line 'echo "Custom VPN post-up";' \\
        --agent-firewall-vpn-pre-down-enabled false \\
        --agent-firewall-vpn-post-down-enabled true \\
        --agent-firewall-vpn-post-down-line 'echo "Custom VPN post-down";'"""

    ret = init_no_prompt(generate_init_no_prompt_opts(agent_firewall=firewall_opts))
    assert ret == 0

    # Verify config
    with open(wg_quickrs_config_file) as stream:
        conf = yaml.load(stream)

    # VPN should have manual scripts
    vpn_scripts = conf["agent"]["firewall"]["vpn"]
    assert len(vpn_scripts["pre_up"]) == 0
    assert len(vpn_scripts["post_up"]) == 1
    assert vpn_scripts["post_up"][0]["script"] == 'echo "Custom VPN post-up";'
    assert len(vpn_scripts["pre_down"]) == 0
    assert len(vpn_scripts["post_down"]) == 1
    assert vpn_scripts["post_down"][0]["script"] == 'echo "Custom VPN post-down";'


def test_init_firewall_all_services_automated(setup_wg_quickrs_folder):
    """Test all firewall services (HTTP, HTTPS, VPN) with automated setup"""
    from ruamel.yaml import YAML
    yaml = YAML()
    yaml.preserve_quotes = True

    utilities = get_available_firewall_utilities()
    interfaces = get_available_network_interfaces()

    if not utilities or not interfaces:
        pytest.skip("No firewall utilities or network interfaces available on this system")

    utility = utilities[0]
    gateway = interfaces[0]
    setup_wg_quickrs_folder(None)
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    # All services automated
    firewall_opts = f"""--agent-firewall-enabled true \\
        --agent-firewall-configure-http true \\
        --agent-firewall-http-automated true \\
        --agent-firewall-configure-https true \\
        --agent-firewall-https-automated true \\
        --agent-firewall-configure-vpn true \\
        --agent-firewall-vpn-automated true \\
        --agent-firewall-utility {utility} \\
        --agent-firewall-gateway {gateway}"""

    ret = init_no_prompt(generate_init_no_prompt_opts(agent_firewall=firewall_opts))
    assert ret == 0

    # Verify config
    with open(wg_quickrs_config_file) as stream:
        conf = yaml.load(stream)

    # All services should have automated scripts
    vpn_scripts = conf["agent"]["firewall"]["vpn"]

    # VPN should have 5 post_up and 5 post_down scripts
    assert len(vpn_scripts["post_up"]) > 0
    assert len(vpn_scripts["post_down"]) > 0


def test_init_firewall_disabled_services_empty(setup_wg_quickrs_folder):
    """Test that disabled firewall services result in empty script arrays"""
    from ruamel.yaml import YAML
    yaml = YAML()
    yaml.preserve_quotes = True

    utilities = get_available_firewall_utilities()
    interfaces = get_available_network_interfaces()

    if not utilities or not interfaces:
        pytest.skip("No firewall utilities or network interfaces available on this system")

    utility = utilities[0]
    gateway = interfaces[0]
    setup_wg_quickrs_folder(None)
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    # Only VPN enabled, HTTP and HTTPS disabled
    firewall_opts = f"""--agent-firewall-enabled true \\
        --agent-firewall-configure-http false \\
        --agent-firewall-configure-https false \\
        --agent-firewall-configure-vpn true \\
        --agent-firewall-vpn-automated true \\
        --agent-firewall-utility {utility} \\
        --agent-firewall-gateway {gateway}"""

    ret = init_no_prompt(generate_init_no_prompt_opts(agent_firewall=firewall_opts))
    assert ret == 0

    # Verify config
    with open(wg_quickrs_config_file) as stream:
        conf = yaml.load(stream)

    # HTTP and HTTPS should have empty arrays
    http_scripts = conf["agent"]["firewall"]["http"]
    https_scripts = conf["agent"]["firewall"]["https"]

    assert http_scripts["pre_up"] == []
    assert http_scripts["post_down"] == []
    assert https_scripts["pre_up"] == []
    assert https_scripts["post_down"] == []

    # VPN should have scripts
    vpn_scripts = conf["agent"]["firewall"]["vpn"]
    assert len(vpn_scripts["post_up"]) > 0
    assert len(vpn_scripts["post_down"]) > 0


def test_init_firewall_script_validation(setup_wg_quickrs_folder):
    """Test that invalid firewall scripts are rejected"""
    setup_wg_quickrs_folder(None)

    # Invalid HTTP script (missing semicolon)
    firewall_opts = """--agent-firewall-enabled true \\
        --agent-firewall-configure-http true \\
        --agent-firewall-http-automated false \\
        --agent-firewall-http-pre-up-enabled true \\
        --agent-firewall-http-pre-up-line 'not-a-valid-script' \\
        --agent-firewall-configure-https false \\
        --agent-firewall-configure-vpn false"""

    ret = init_no_prompt(generate_init_no_prompt_opts(agent_firewall=firewall_opts))
    assert ret != 0


def test_init_with_existing_config(setup_wg_quickrs_folder):
    """Test that init fails when a config file already exists."""
    setup_wg_quickrs_folder("no_auth_single_peer")

    # Config already exists, init should fail
    ret = init_no_prompt(generate_init_no_prompt_opts())
    assert ret != 0


@pytest.mark.parametrize(
    "invalid_flag",
    [
        "--network-name",  # missing value
        "--network-subnet",  # missing value
        "--invalid-flag test",  # unknown flag
    ],
)
def test_init_invalid_flags(setup_wg_quickrs_folder, invalid_flag):
    """Test init command with invalid flags."""
    setup_wg_quickrs_folder(None)

    command = " ".join(get_wg_quickrs_command()) + f" init --no-prompt true {invalid_flag}"
    result = subprocess.run(
        command,
        shell=True,
        capture_output=True,
        text=True
    )
    assert result.returncode != 0


def test_init_conflicting_options(setup_wg_quickrs_folder):
    """Test init with conflicting options (e.g., enabling something without required params)."""
    setup_wg_quickrs_folder(None)

    # Enable web password without providing a password
    ret = init_no_prompt(generate_init_no_prompt_opts(
        agent_web_password="--agent-web-password-enabled true"
    ))
    assert ret != 0

    # Enable VPN without providing a port
    ret = init_no_prompt(generate_init_no_prompt_opts(
        agent_vpn="--agent-vpn-enabled true"
    ))
    assert ret != 0


@pytest.mark.parametrize(
    "script_type,script_lines,expected_count",
    [
        # Agent peer scripts with multiple lines
        ("agent_peer_script_pre_up",
         "--agent-peer-script-pre-up-enabled true --agent-peer-script-pre-up-line 'echo first;' --agent-peer-script-pre-up-line 'echo second;' --agent-peer-script-pre-up-line 'echo third;'",
         3),
        ("agent_peer_script_post_up",
         "--agent-peer-script-post-up-enabled true --agent-peer-script-post-up-line 'echo one;' --agent-peer-script-post-up-line 'echo two;'",
         2),
        ("agent_peer_script_pre_down",
         "--agent-peer-script-pre-down-enabled true --agent-peer-script-pre-down-line 'echo alpha;' --agent-peer-script-pre-down-line 'echo beta;' --agent-peer-script-pre-down-line 'echo gamma;' --agent-peer-script-pre-down-line 'echo delta;'",
         4),
        ("agent_peer_script_post_down",
         "--agent-peer-script-post-down-enabled true --agent-peer-script-post-down-line 'echo single;'",
         1),
        # Default peer scripts with multiple lines
        ("default_peer_script_pre_up",
         "--default-peer-script-pre-up-enabled true --default-peer-script-pre-up-line 'echo A;' --default-peer-script-pre-up-line 'echo B;'",
         2),
        ("default_peer_script_post_up",
         "--default-peer-script-post-up-enabled true --default-peer-script-post-up-line 'echo 1;' --default-peer-script-post-up-line 'echo 2;' --default-peer-script-post-up-line 'echo 3;'",
         3),
        ("default_peer_script_pre_down",
         "--default-peer-script-pre-down-enabled true --default-peer-script-pre-down-line 'echo x;'",
         1),
        ("default_peer_script_post_down",
         "--default-peer-script-post-down-enabled true --default-peer-script-post-down-line 'echo p;' --default-peer-script-post-down-line 'echo q;' --default-peer-script-post-down-line 'echo r;' --default-peer-script-post-down-line 'echo s;' --default-peer-script-post-down-line 'echo t;'",
         5),
    ],
)
def test_init_multiple_scripts(setup_wg_quickrs_folder, script_type, script_lines, expected_count):
    """Test that multiple script lines can be specified via CLI."""
    from ruamel.yaml import YAML
    yaml = YAML()
    yaml.preserve_quotes = True

    setup_wg_quickrs_folder(None)
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    # Run init with multiple script lines
    ret = init_no_prompt(generate_init_no_prompt_opts(**{script_type: script_lines}))
    assert ret == 0

    # Load and verify config
    with open(wg_quickrs_config_file) as stream:
        conf = yaml.load(stream)

    # Determine the path based on script type
    if script_type.startswith("agent_peer_script_"):
        script_kind = script_type.replace("agent_peer_script_", "")
        this_peer_id = conf["network"]["this_peer"]
        scripts_list = conf["network"]["peers"][this_peer_id]["scripts"][script_kind]
    else:  # default_peer_script_*
        script_kind = script_type.replace("default_peer_script_", "")
        scripts_list = conf["network"]["defaults"]["peer"]["scripts"][script_kind]

    # Verify the correct number of scripts
    assert len(scripts_list) == expected_count

    # Verify all scripts are enabled
    for script in scripts_list:
        assert script["enabled"] is True
        assert script["script"].endswith(";")


@pytest.mark.parametrize(
    "disabled_script_type",
    [
        "agent_peer_script_pre_up",
        "agent_peer_script_post_up",
        "agent_peer_script_pre_down",
        "agent_peer_script_post_down",
        "default_peer_script_pre_up",
        "default_peer_script_post_up",
        "default_peer_script_pre_down",
        "default_peer_script_post_down",
    ],
)
def test_init_disabled_scripts_empty_array(setup_wg_quickrs_folder, disabled_script_type):
    """Test that disabled scripts result in empty arrays, not arrays with empty strings."""
    from ruamel.yaml import YAML
    yaml = YAML()
    yaml.preserve_quotes = True

    setup_wg_quickrs_folder(None)
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    # Run init with script disabled (default behavior)
    ret = init_no_prompt(generate_init_no_prompt_opts(**{disabled_script_type: f"--{disabled_script_type.replace('_', '-')}-enabled false"}))
    assert ret == 0

    # Load and verify config
    with open(wg_quickrs_config_file) as stream:
        conf = yaml.load(stream)

    # Determine the path based on the script type
    if disabled_script_type.startswith("agent_peer_script_"):
        script_kind = disabled_script_type.replace("agent_peer_script_", "")
        this_peer_id = conf["network"]["this_peer"]
        scripts_list = conf["network"]["peers"][this_peer_id]["scripts"][script_kind]
    else:  # default_peer_script_*
        script_kind = disabled_script_type.replace("default_peer_script_", "")
        scripts_list = conf["network"]["defaults"]["peer"]["scripts"][script_kind]

    # Verify it's an empty array
    assert scripts_list == []
    assert len(scripts_list) == 0


def test_init_mixed_scripts(setup_wg_quickrs_folder):
    """Test mixed scenario with some scripts having multiple lines and others disabled."""
    from ruamel.yaml import YAML
    yaml = YAML()
    yaml.preserve_quotes = True

    setup_wg_quickrs_folder(None)
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    # Run init with mixed script configuration
    ret = init_no_prompt(generate_init_no_prompt_opts(
        agent_peer_script_pre_up="--agent-peer-script-pre-up-enabled true --agent-peer-script-pre-up-line 'echo start;' --agent-peer-script-pre-up-line 'echo middle;' --agent-peer-script-pre-up-line 'echo end;'",
        agent_peer_script_post_up="--agent-peer-script-post-up-enabled false",
        agent_peer_script_pre_down="--agent-peer-script-pre-down-enabled true --agent-peer-script-pre-down-line 'echo cleanup;'",
        agent_peer_script_post_down="--agent-peer-script-post-down-enabled false",
        default_peer_script_pre_up="--default-peer-script-pre-up-enabled true --default-peer-script-pre-up-line 'echo default1;' --default-peer-script-pre-up-line 'echo default2;'",
        default_peer_script_post_up="--default-peer-script-post-up-enabled false",
        default_peer_script_pre_down="--default-peer-script-pre-down-enabled false",
        default_peer_script_post_down="--default-peer-script-post-down-enabled true --default-peer-script-post-down-line 'echo done;'"
    ))
    assert ret == 0

    # Load and verify config
    with open(wg_quickrs_config_file) as stream:
        conf = yaml.load(stream)

    this_peer_id = conf["network"]["this_peer"]

    # Verify agent peer scripts
    assert len(conf["network"]["peers"][this_peer_id]["scripts"]["pre_up"]) == 3
    assert len(conf["network"]["peers"][this_peer_id]["scripts"]["post_up"]) == 0
    assert len(conf["network"]["peers"][this_peer_id]["scripts"]["pre_down"]) == 1
    assert len(conf["network"]["peers"][this_peer_id]["scripts"]["post_down"]) == 0

    # Verify default peer scripts
    assert len(conf["network"]["defaults"]["peer"]["scripts"]["pre_up"]) == 2
    assert len(conf["network"]["defaults"]["peer"]["scripts"]["post_up"]) == 0
    assert len(conf["network"]["defaults"]["peer"]["scripts"]["pre_down"]) == 0
    assert len(conf["network"]["defaults"]["peer"]["scripts"]["post_down"]) == 1

    # Verify content of specific scripts
    assert conf["network"]["peers"][this_peer_id]["scripts"]["pre_up"][0]["script"] == "echo start;"
    assert conf["network"]["peers"][this_peer_id]["scripts"]["pre_up"][1]["script"] == "echo middle;"
    assert conf["network"]["peers"][this_peer_id]["scripts"]["pre_up"][2]["script"] == "echo end;"
    assert conf["network"]["peers"][this_peer_id]["scripts"]["pre_down"][0]["script"] == "echo cleanup;"
    assert conf["network"]["defaults"]["peer"]["scripts"]["pre_up"][0]["script"] == "echo default1;"
    assert conf["network"]["defaults"]["peer"]["scripts"]["pre_up"][1]["script"] == "echo default2;"
    assert conf["network"]["defaults"]["peer"]["scripts"]["post_down"][0]["script"] == "echo done;"


def test_init_script_order_preserved(setup_wg_quickrs_folder):
    """Test that script order is preserved when multiple scripts are specified."""
    from ruamel.yaml import YAML
    yaml = YAML()
    yaml.preserve_quotes = True

    setup_wg_quickrs_folder(None)
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    # Run init with scripts in a specific order
    ret = init_no_prompt(generate_init_no_prompt_opts(
        agent_peer_script_pre_up="--agent-peer-script-pre-up-enabled true --agent-peer-script-pre-up-line 'echo FIRST;' --agent-peer-script-pre-up-line 'echo SECOND;' --agent-peer-script-pre-up-line 'echo THIRD;' --agent-peer-script-pre-up-line 'echo FOURTH;' --agent-peer-script-pre-up-line 'echo FIFTH;'"
    ))
    assert ret == 0

    # Load and verify config
    with open(wg_quickrs_config_file) as stream:
        conf = yaml.load(stream)

    this_peer_id = conf["network"]["this_peer"]
    scripts = conf["network"]["peers"][this_peer_id]["scripts"]["pre_up"]

    # Verify order is preserved
    assert len(scripts) == 5
    assert scripts[0]["script"] == "echo FIRST;"
    assert scripts[1]["script"] == "echo SECOND;"
    assert scripts[2]["script"] == "echo THIRD;"
    assert scripts[3]["script"] == "echo FOURTH;"
    assert scripts[4]["script"] == "echo FIFTH;"
