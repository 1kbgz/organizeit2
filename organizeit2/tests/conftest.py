import os
import time
from pathlib import Path
from tempfile import TemporaryDirectory

from pytest import fixture

_FNAMES = ["file1.png", "file2.png", "subsubdir1", "subsubdir2", "file1", "file1.md", "file1.txt", "file2", "file2.md", "file2.txt"]


@fixture(scope="module", autouse=True)
def tempdir():
    with TemporaryDirectory() as td:
        yield td


@fixture(scope="module", autouse=True)
def directory_str():
    _base = Path("organizeit2/tests/directory/subdir1")
    base_time = time.time() - 200
    for i, fname in enumerate(_FNAMES):
        p = _base / fname
        p.touch()
        os.utime(p, (base_time + i * 10, base_time + i * 10))
    return "file://organizeit2/tests/directory"


@fixture(scope="module", autouse=True)
def directory_str_extra():
    Path("organizeit2/tests/directory2").mkdir(exist_ok=True)
    base_time = time.time() - 200
    for i, fname in enumerate(_FNAMES):
        p = Path(f"organizeit2/tests/directory2/{fname}")
        p.touch()
        os.utime(p, (base_time + i * 10, base_time + i * 10))
    return "file://organizeit2/tests/directory2/"
