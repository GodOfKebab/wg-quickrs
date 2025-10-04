from tests.pytest.conftest import setup_wg_quickrs_folder
from tests.pytest.helpers import get_wg_quickrs_command
import subprocess


def test_help(setup_wg_quickrs_folder):
    setup_wg_quickrs_folder("no_auth_single_peer")

    result = subprocess.run(
        get_wg_quickrs_command() + ['--help'],
        capture_output=True,
        text=True
    )
    assert result.returncode == 0
    assert "Usage: wg-quickrs" in result.stdout

