from tests.pytest.conftest import setup_wg_quickrs_folder
from tests.pytest.helpers import get_wg_quickrs_command
from tests.pytest.helpers import get_paths
import yaml
import subprocess
from deepdiff import DeepDiff


def test_agent_set_web_address(setup_wg_quickrs_folder):
    setup_wg_quickrs_folder("no_auth_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    with open(wg_quickrs_config_file) as stream:
        old_conf = yaml.safe_load(stream)

    result = subprocess.run(
        get_wg_quickrs_command() + ['agent', 'set-web-address', '192.168.10.10'],
        capture_output=True,
        text=True
    )

    assert result.returncode == 0

    with open(wg_quickrs_config_file) as stream:
        new_conf = yaml.safe_load(stream)

    assert new_conf['agent']['web']['address'] == '192.168.10.10'

    new_conf['agent']['web']['address'] = old_conf['agent']['web']['address']
    assert DeepDiff(new_conf, old_conf, ignore_order=True) == dict()


