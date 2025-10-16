from tests.pytest.conftest import setup_wg_quickrs_folder
from tests.pytest.helpers import get_wg_quickrs_command
import subprocess


def test_init_no_prompt_simple_http(setup_wg_quickrs_folder):
    setup_wg_quickrs_folder(None)

    command = " ".join(get_wg_quickrs_command()) + " init --no-prompt true \\"
    command += \
    """
    --network-identifier wg-quickrs-home \\
    --network-subnet     10.0.34.0/24    \\
    --agent-web-address          0.0.0.0                          \\
    --agent-web-http-enabled     true                             \\
    --agent-web-http-port        80                               \\
    --agent-web-https-enabled    false                            \\
    --agent-web-password-enabled false                            \\
    --agent-vpn-enabled          false                            \\
    --agent-firewall-enabled     false                            \\
    --agent-peer-name                     wg-quickrs-host                \\
    --agent-peer-vpn-endpoint             YOUR_SERVER:51820              \\
    --agent-peer-kind                     server                         \\
    --agent-peer-icon-enabled             false                          \\
    --agent-peer-vpn-internal-address     10.0.34.1                      \\
    --agent-peer-dns-enabled              true                           \\
    --agent-peer-dns-server               1.1.1.1                        \\
    --agent-peer-mtu-enabled              false                          \\
    --agent-peer-script-pre-up-enabled    false                          \\
    --agent-peer-script-post-up-enabled   false                          \\
    --agent-peer-script-pre-down-enabled  false                          \\
    --agent-peer-script-post-down-enabled false                          \\
    --default-peer-kind                               laptop  \\
    --default-peer-icon-enabled                       false   \\
    --default-peer-dns-enabled                        true    \\
    --default-peer-dns-server                         1.1.1.1 \\
    --default-peer-mtu-enabled                        false   \\
    --default-peer-script-pre-up-enabled              false   \\
    --default-peer-script-post-up-enabled             false   \\
    --default-peer-script-pre-down-enabled            false   \\
    --default-peer-script-post-down-enabled           false   \\
    --default-connection-persistent-keepalive-enabled true    \\
    --default-connection-persistent-keepalive-period  25
    """

    result = subprocess.run(
        command,
        shell=True,
        capture_output=True,
        text=True
    )
    print(result.stdout)
    print(result.stderr)
    assert result.returncode == 0

