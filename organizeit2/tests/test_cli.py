from unittest.mock import patch

from typer import Exit
from typer.testing import CliRunner

from organizeit2.cli import main, match, rematch

runner = CliRunner()


def test_match(directory_str):
    app = main(_test=True)
    directory_str = str(directory_str)
    assert runner.invoke(app, ["match", directory_str, "directory*"]).exit_code == 0
    assert runner.invoke(app, ["match", directory_str, "directory*", "--invert"]).exit_code == 1
    assert runner.invoke(app, ["match", directory_str, "directory", "--no-name-only"]).exit_code == 1
    assert runner.invoke(app, ["match", directory_str, "directory", "--no-name-only", "--invert"]).exit_code == 0
    assert runner.invoke(app, ["match", directory_str, "*organizeit2*directory", "--no-name-only"]).exit_code == 0
    assert runner.invoke(app, ["match", directory_str, "*organizeit2*directory", "--no-name-only", "--invert"]).exit_code == 1


def test_all_match(directory_str):
    app = main(_test=True)
    directory_str = f"{directory_str}/"
    assert runner.invoke(app, ["match", directory_str, "subdir*"]).exit_code == 0
    assert runner.invoke(app, ["match", directory_str, "dir*"]).exit_code == 1

    with patch("organizeit2.cli.print") as print_mock:
        assert runner.invoke(app, ["match", directory_str, "subdir*", "--list", "--invert"]).exit_code == 1
        assert print_mock.call_count == 4


def test_rematch(directory_str):
    app = main(_test=True)
    directory_str = str(directory_str)
    assert runner.invoke(app, ["rematch", directory_str, "directory"]).exit_code == 0
    assert runner.invoke(app, ["rematch", directory_str, "directory", "--invert"]).exit_code == 1
    assert runner.invoke(app, ["rematch", directory_str, "directory", "--no-name-only"]).exit_code == 1
    assert runner.invoke(app, ["rematch", directory_str, "directory", "--no-name-only", "--invert"]).exit_code == 0
    assert runner.invoke(app, ["rematch", directory_str, "file://[a-zA-Z0-9/]*", "--no-name-only"]).exit_code == 0


def test_all_rematch(directory_str):
    app = main(_test=True)
    directory_str = f"{directory_str}/"
    assert runner.invoke(app, ["rematch", directory_str, "subdir[0-9]+"]).exit_code == 0
    assert runner.invoke(app, ["rematch", directory_str, "subdir[0-3]+"]).exit_code == 1

    with patch("organizeit2.cli.print") as print_mock:
        assert runner.invoke(app, ["rematch", directory_str, "subdir*", "--list", "--invert"]).exit_code == 1
        assert print_mock.call_count == 4


def test_match_limit_leaves(directory_str):
    directory_str = directory_str + "/subdir1/"
    with patch("organizeit2.cli._unmatched_table") as table_mock:
        try:
            match(directory_str, "*", list=False, limit=2, leaves=7, invert=True, by="age")
        except Exit:
            pass
        assert len(table_mock.call_args_list[0][0][0]) == 2
        assert str(table_mock.call_args_list[0][0][0][0]).endswith("subdir1/file1.png")
        assert str(table_mock.call_args_list[0][0][0][1]).endswith("subdir1/file2.png")


def test_rematch_limit_leaves(directory_str):
    directory_str = directory_str + "/subdir1/"
    with patch("organizeit2.cli._unmatched_table") as table_mock:
        try:
            rematch(directory_str, ".*", list=False, limit=2, leaves=7, invert=True, by="age")
        except Exit:
            pass
        assert len(table_mock.call_args_list[0][0][0]) == 2
        assert str(table_mock.call_args_list[0][0][0][0]).endswith("subdir1/file1.png")
        assert str(table_mock.call_args_list[0][0][0][1]).endswith("subdir1/file2.png")


def test_match_limit_leaves_by_size(directory_str):
    directory_str = directory_str + "/subdir1/"
    with patch("organizeit2.cli._unmatched_table") as table_mock:
        try:
            match(directory_str, "*", list=False, limit=5, invert=True, by="size", desc=True)
        except Exit:
            pass
        assert len(table_mock.call_args_list[0][0][0]) == 5
        assert str(table_mock.call_args_list[0][0][0][0]).endswith("subdir1/file2.txt")
        assert str(table_mock.call_args_list[0][0][0][1]).endswith("subdir1/file2.md")
        assert str(table_mock.call_args_list[0][0][0][2]).endswith("subdir1/file2")
        assert str(table_mock.call_args_list[0][0][0][3]).endswith("subdir1/file1.txt")
        assert str(table_mock.call_args_list[0][0][0][4]).endswith("subdir1/file1.md")


def test_rematch_limit_leaves_by_size(directory_str):
    directory_str = directory_str + "/subdir1/"
    with patch("organizeit2.cli._unmatched_table") as table_mock:
        try:
            rematch(directory_str, ".*", list=False, limit=5, invert=True, by="size", desc=True)
        except Exit:
            pass
        assert len(table_mock.call_args_list[0][0][0]) == 5
        assert str(table_mock.call_args_list[0][0][0][0]).endswith("subdir1/file2.txt")
        assert str(table_mock.call_args_list[0][0][0][1]).endswith("subdir1/file2.md")
        assert str(table_mock.call_args_list[0][0][0][2]).endswith("subdir1/file2")
        assert str(table_mock.call_args_list[0][0][0][3]).endswith("subdir1/file1.txt")
        assert str(table_mock.call_args_list[0][0][0][4]).endswith("subdir1/file1.md")


def test_match_limit_leaves_desc(directory_str):
    directory_str = directory_str + "/subdir1/"
    with patch("organizeit2.cli._unmatched_table") as table_mock:
        try:
            match(directory_str, "*", list=False, limit=2, leaves=7, invert=True, desc=True, by="age")
        except Exit:
            pass
        assert len(table_mock.call_args_list[0][0][0]) == 2
        assert str(table_mock.call_args_list[0][0][0][0]).endswith("subdir1/file2.txt")
        assert str(table_mock.call_args_list[0][0][0][1]).endswith("subdir1/file2.md")


def test_rematch_limit_leaves_desc(directory_str):
    directory_str = directory_str + "/subdir1/"
    with patch("organizeit2.cli._unmatched_table") as table_mock:
        try:
            rematch(directory_str, ".*", list=False, limit=2, leaves=7, invert=True, desc=True, by="age")
        except Exit:
            pass
        assert len(table_mock.call_args_list[0][0][0]) == 2
        assert str(table_mock.call_args_list[0][0][0][0]).endswith("subdir1/file2.txt")
        assert str(table_mock.call_args_list[0][0][0][1]).endswith("subdir1/file2.md")


def test_match_limit(directory_str):
    directory_str = directory_str + "/subdir1/"
    with patch("organizeit2.cli._unmatched_table") as table_mock:
        try:
            match(directory_str, "*", list=False, limit=3, invert=True, by="age")
        except Exit:
            pass
        assert len(table_mock.call_args_list[0][0][0]) == 3
        assert str(table_mock.call_args_list[0][0][0][0]).endswith("subdir1/file1.png")
        assert str(table_mock.call_args_list[0][0][0][1]).endswith("subdir1/file2.png")
        assert str(table_mock.call_args_list[0][0][0][2]).endswith("subdir1/subsubdir1")


def test_rematch_limit(directory_str):
    directory_str = directory_str + "/subdir1/"
    with patch("organizeit2.cli._unmatched_table") as table_mock:
        try:
            rematch(directory_str, ".*", list=False, limit=3, invert=True, by="age")
        except Exit:
            pass
        assert len(table_mock.call_args_list[0][0][0]) == 3
        assert str(table_mock.call_args_list[0][0][0][0]).endswith("subdir1/file1.png")
        assert str(table_mock.call_args_list[0][0][0][1]).endswith("subdir1/file2.png")
        assert str(table_mock.call_args_list[0][0][0][2]).endswith("subdir1/subsubdir1")


def test_match_leaves(directory_str):
    directory_str = directory_str + "/subdir1/"
    with patch("organizeit2.cli._unmatched_table") as table_mock:
        try:
            match(directory_str, "*", list=False, leaves=8, invert=True, by="age")
        except Exit:
            pass
        assert len(table_mock.call_args_list[0][0][0]) == 2
        assert str(table_mock.call_args_list[0][0][0][0]).endswith("subdir1/file1.png")
        assert str(table_mock.call_args_list[0][0][0][1]).endswith("subdir1/file2.png")


def test_rematch_leaves(directory_str):
    directory_str = directory_str + "/subdir1/"
    with patch("organizeit2.cli._unmatched_table") as table_mock:
        try:
            rematch(directory_str, ".*", list=False, leaves=8, invert=True, by="age")
        except Exit:
            pass
        assert len(table_mock.call_args_list[0][0][0]) == 2
        assert str(table_mock.call_args_list[0][0][0][0]).endswith("subdir1/file1.png")
        assert str(table_mock.call_args_list[0][0][0][1]).endswith("subdir1/file2.png")
