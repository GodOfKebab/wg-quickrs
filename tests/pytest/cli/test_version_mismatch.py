from tests.pytest.conftest import setup_wg_quickrs_folder
from tests.pytest.helpers import get_wg_quickrs_command, get_paths
import subprocess
from ruamel.yaml import YAML

yaml = YAML()
yaml.preserve_quotes = True


def test_version_mismatch_error(setup_wg_quickrs_folder):
    """Test that a non-matching major version in conf.yml throws an error."""
    setup_wg_quickrs_folder("no_auth_single_peer")
    pytest_folder, wg_quickrs_config_folder, wg_quickrs_config_file = get_paths()

    # Modify the version to a non-matching major version
    with open(wg_quickrs_config_file) as stream:
        conf = yaml.load(stream)

    conf["version"] = "999.0.0"

    with open(wg_quickrs_config_file, 'w') as stream:
        yaml.dump(conf, stream)

    # Try to run agent, should fail
    result = subprocess.run(
        get_wg_quickrs_command() + ['agent', 'run'],
        capture_output=True,
        text=True,
        timeout=2
    )

    assert result.returncode != 0
    assert "version" in result.stdout.lower() or "version" in result.stderr.lower()
