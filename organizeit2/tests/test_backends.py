import os

import pytest

from organizeit2 import Directory


class TestBackends:
    # file:// (Rust native LocalFs)
    def test_file(self, directory_str):
        d = Directory(path=directory_str)
        root = str(d)
        assert root.startswith("file://")
        assert [str(_) for _ in d.ls()] == [
            f"{root}/subdir1",
            f"{root}/subdir2",
            f"{root}/subdir3",
            f"{root}/subdir4",
        ]
        assert len(d.recurse()) == 64

    # file-rs:// (fsspec-rs Python LocalFileSystem via adapter)
    def test_file_rs(self, directory_str):
        import fsspec_rs  # noqa: F401 — ensure file-rs:// protocol is registered with fsspec

        # Replace file:// with file-rs://
        rs_path = directory_str.replace("file://", "file-rs://", 1)
        d = Directory(path=rs_path)
        root = str(d)
        assert root.startswith("file-rs://")
        assert [str(_) for _ in d.ls()] == [
            f"{root}/subdir1",
            f"{root}/subdir2",
            f"{root}/subdir3",
            f"{root}/subdir4",
        ]
        assert len(d.recurse()) == 64

    # s3:// (Rust native S3Fs)
    @pytest.mark.skipif(os.environ.get("FSSPEC_S3_ENDPOINT_URL") is None, reason="Skipping test that require S3 credentials")
    def test_s3(self):
        d = Directory(path="s3://timkpaine-public/projects/organizeit2")
        root = str(d)
        assert [str(_) for _ in d.ls()] == [
            f"{root}/subdir1",
            f"{root}/subdir2",
            f"{root}/subdir3",
            f"{root}/subdir4",
        ]
        assert len(d.recurse()) == 64

    # s3-rs:// (fsspec-rs Python S3FileSystem via adapter)
    @pytest.mark.skipif(os.environ.get("FSSPEC_S3_ENDPOINT_URL") is None, reason="Skipping test that require S3 credentials")
    def test_s3_rs(self):
        import fsspec_rs  # noqa: F401 — ensure s3-rs:// protocol is registered with fsspec

        d = Directory(path="s3-rs://timkpaine-public/projects/organizeit2")
        root = str(d)
        assert [str(_) for _ in d.ls()] == [
            f"{root}/subdir1",
            f"{root}/subdir2",
            f"{root}/subdir3",
            f"{root}/subdir4",
        ]
        assert len(d.recurse()) == 64
