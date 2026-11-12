import fsspec_rs

from organizeit2.organizeit2 import linked_to_libfsspec


def test_organizeit2_shares_the_registry_with_fsspec_rs():
    assert linked_to_libfsspec() == fsspec_rs.fsspec_rs.linked_to_libfsspec()
