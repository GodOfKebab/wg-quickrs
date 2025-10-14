from tests.pytest.conftest import setup_wg_quickrs_agent
from tests.pytest.helpers import get_paths, get_wg_quickrs_command
import subprocess


def test_bruno(setup_wg_quickrs_agent):
    base_url = setup_wg_quickrs_agent("test_pwd_single_peer")

    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    agent = subprocess.Popen(get_wg_quickrs_command() + ['agent', 'run'])

    ca_path = wg_quickrs_config_folder / "certs/root/rootCA.crt"
    result = subprocess.run(f"bru run --env-var base-url={base_url} --env-var password='test' --cacert {ca_path}",
                            cwd=pytest_folder.parent / "bruno/wg-quickrs",
                            shell=True,
                            capture_output=True,
                            text=True)
    print(result.stdout)
    assert result.returncode == 0

    # terminate agent when the test is over
    agent.terminate()
    agent.wait()

