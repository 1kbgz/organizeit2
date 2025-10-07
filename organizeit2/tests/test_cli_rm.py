from unittest.mock import patch

from typer import Exit
from typer.testing import CliRunner

from organizeit2 import Directory
from organizeit2.cli import rematch

runner = CliRunner()


def test_rematch_op_rm_real(directory_str_extra):
    directory_str = directory_str_extra
    with patch("organizeit2.cli.print") as print_mock:
        try:
            rematch(directory_str, ".*", list=False, invert=True, by="size", desc=True, op="rm")
        except Exit:
            pass
        assert print_mock.call_count == 0
    assert Directory(path=directory_str).ls() == []
