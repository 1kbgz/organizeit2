from pathlib import Path as BasePath

import pytest
from fsspec import AbstractFileSystem
from fsspec_pydantic import DirectoryPath

from organizeit2 import Directory, File, OrganizeIt, Path


class TestTypes:
    def test_oit2(self, tempdir):
        oi = OrganizeIt(fs=f"local://{tempdir}")
        expected = f"file://{tempdir.replace(chr(92), '/')}"
        assert str(oi.expand(tempdir)) == expected

    def test_directory(self, tempdir):
        d = Directory(path=f"local://{tempdir}")
        assert isinstance(d.path.fs, AbstractFileSystem)
        assert isinstance(d.path.path, BasePath)
        expected_path = f"file://{tempdir.replace(chr(92), '/')}"
        assert d.model_dump_json() == f'{{"path":"{expected_path}","type_":"organizeit2.types.Directory"}}'
        assert repr(d) == f"Directory(path={expected_path})"
        assert str(d) == expected_path
        assert str(d.path) == expected_path

    def test_directory_file_resolve(self, directory_str):
        assert isinstance(File(path=directory_str).resolve(), Directory)
        assert isinstance(Path(path=directory_str).resolve(), Directory)
        assert isinstance(Directory(path=f"file://{__file__}").resolve(), File)
        assert isinstance(Path(path=f"file://{__file__}").resolve(), File)

    def test_directory_from_directorypath(self):
        Directory(path=DirectoryPath("local:///tmp"))

    def test_directory_ls(self, directory_str):
        d = Directory(path=directory_str)
        root = str(d)
        assert [str(_) for _ in d.ls()] == [
            f"{root}/subdir1",
            f"{root}/subdir2",
            f"{root}/subdir3",
            f"{root}/subdir4",
        ]

    def test_directory_recurse(self, directory_str):
        d = Directory(path=directory_str)
        assert len([str(_) for _ in d.recurse()]) == 64

    def test_path_hashable(self, directory_str):
        d = Directory(path=directory_str)
        assert len(set(d.recurse())) == 64

    def test_size(self, directory_str):
        d = Directory(path=directory_str)
        assert d.size() == 262144

    def test_link(self, directory_str):
        d = Directory(path=directory_str)
        d2 = Directory(path=f"{directory_str}_link")
        d.link(d2)
        d2.unlink()

    def test_cant_link(self, directory_str):
        d = Directory(path=directory_str)
        d2 = Directory(path=directory_str)
        with pytest.raises(RuntimeError):
            d.link(d2)

    def test_match(self, directory_str):
        d = Directory(path=directory_str)
        assert d.match("directory*")
        assert d.match("directory*", invert=True) is False
        assert d.match("directory", name_only=False) is False
        assert d.match("directory", name_only=False, invert=True)
        assert d.match("*organizeit2*directory", name_only=False)
        assert d.match("*organizeit2*directory", name_only=False, invert=True) is False

    def test_all_match(self, directory_str):
        d = Directory(path=directory_str)
        assert len(d.all_match("subdir*")) == 4
        assert len(d.all_match("dir*")) == 0

    def test_rematch(self, directory_str):
        d = Directory(path=directory_str)
        assert d.rematch("directory")
        assert d.rematch("directory", invert=True) is False
        assert d.rematch("directory", name_only=False) is False
        assert d.rematch("directory", name_only=False, invert=True)
        assert d.rematch("file://[a-zA-Z0-9/]*", name_only=False)

    def test_all_rematch(self, directory_str):
        d = Directory(path=directory_str)
        assert len(d.all_rematch("subdir[0-9]+")) == 4
        assert len(d.all_rematch("subdir[0-3]+")) == 3

    def test_convenience(self):
        d = File(path=f"file://{__file__}")
        assert str(d).endswith("test_types.py")
        assert (d / "..").resolve().str().endswith("tests")
        assert (d / "..").resolve() == d.parent.resolve()
        assert (d / ".." / ".." / "tests").resolve() == d.parent.resolve()
