import os
import sys

import pytest

from organizeit2 import Directory


class TestRegressions:
    @pytest.mark.skipif(sys.platform == "win32", reason="Symlinks behave differently on Windows")
    def test_bad_symlink(self, tempdir, directory_str):
        # Make a bad symlink in tempdir
        bad_symlink_path = os.path.join(tempdir, "bad_symlink")
        os.symlink("/tmp/whatever/non_existent_file", bad_symlink_path)

        assert "bad_symlink" in os.listdir(tempdir)
        d = Directory(path=f"local://{tempdir}")
        assert len(d.ls()) == 1
        assert d.size() == 0
