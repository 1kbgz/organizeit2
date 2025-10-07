import os
from tempfile import TemporaryDirectory

from pytest import fixture


@fixture(scope="module", autouse=True)
def tempdir():
    with TemporaryDirectory() as td:
        yield td


@fixture(scope="module", autouse=True)
def directory_str():
    os.system("touch organizeit2/tests/directory/subdir1/file1.png")
    os.system("touch organizeit2/tests/directory/subdir1/file2.png")
    os.system("touch organizeit2/tests/directory/subdir1/subsubdir1")
    os.system("touch organizeit2/tests/directory/subdir1/subsubdir2")
    os.system("touch organizeit2/tests/directory/subdir1/file1")
    os.system("touch organizeit2/tests/directory/subdir1/file1.md")
    os.system("touch organizeit2/tests/directory/subdir1/file1.txt")
    os.system("touch organizeit2/tests/directory/subdir1/file2")
    os.system("touch organizeit2/tests/directory/subdir1/file2.md")
    os.system("touch organizeit2/tests/directory/subdir1/file2.txt")
    return "file://organizeit2/tests/directory"


@fixture(scope="module", autouse=True)
def directory_str_extra():
    os.system("mkdir organizeit2/tests/directory2")
    os.system("touch organizeit2/tests/directory2/file1.png")
    os.system("touch organizeit2/tests/directory2/file2.png")
    os.system("touch organizeit2/tests/directory2/subsubdir1")
    os.system("touch organizeit2/tests/directory2/subsubdir2")
    os.system("touch organizeit2/tests/directory2/file1")
    os.system("touch organizeit2/tests/directory2/file1.md")
    os.system("touch organizeit2/tests/directory2/file1.txt")
    os.system("touch organizeit2/tests/directory2/file2")
    os.system("touch organizeit2/tests/directory2/file2.md")
    os.system("touch organizeit2/tests/directory2/file2.txt")
    return "file://organizeit2/tests/directory2/"
