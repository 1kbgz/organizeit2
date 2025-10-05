from __future__ import annotations

from ccflow import BaseModel
from fsspec.implementations.local import LocalFileSystem
from fsspec_pydantic import FileSystem
from pydantic import Field

from .types import Directory

__all__ = ("OrganizeIt",)


# TODO: inherit from directory?
class OrganizeIt(BaseModel):
    fs: FileSystem = Field(default_factory=LocalFileSystem)

    def expand(self, directory) -> Directory:
        return Directory(path=self.fs.unstrip_protocol(directory))
