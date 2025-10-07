from datetime import datetime
from fnmatch import fnmatch
from os import symlink, unlink
from re import match as re_match

from ccflow import BaseModel
from fsspec.implementations.local import LocalFileSystem
from fsspec_pydantic import DirectoryPath, FilePath, Path as BasePath

__all__ = (
    "Directory",
    "File",
    "Path",
)


class SharedAPI:
    def __str__(self) -> str:
        return str(self.path)

    def str(self) -> str:
        return str(self)

    def __hash__(self) -> str:
        return hash(str(self))

    def __lt__(self, other) -> bool:
        return str(self) < str(other)

    def exists(self) -> bool:
        return self.path.fs.exists(self.path.path)

    def as_posix(self) -> str:
        return self.path.path.as_posix()

    def _can_link(self) -> bool:
        return hasattr(self.path.fs, "link")

    def link(self, other, soft: bool = True):
        if not self._can_link() or not other._can_link() or self.path.fs.__class__ != other.path.fs.__class__ or self.__class__ != other.__class__:
            raise RuntimeError(f"Cannot link incompatible filesystems or types: {self} and {other}")
        if other.exists() and not other._can_link():
            raise RuntimeError(f"Cannot link to {other}, exists!")
        elif other.exists() and other.path.fs.islink(other.path.path):
            other.unlink()
        elif other.exists() and not other.path.fs.islink(other.path.path):
            raise RuntimeError(f"Cannot link to {other}!")
        if soft:
            symlink(self.path.path, other.path.path)
        else:
            self.path.fs.link(self.path.path, other.path.path, soft=soft)

    def unlink(self):
        if not isinstance(self.path.fs, LocalFileSystem):
            raise NotImplementedError(f"Unlink not implemented for {self.path.fs}")
        if self._can_link():
            unlink(str(self.path.path))

    def rm(self):
        self.path.fs.rm(self.path.path, recursive=isinstance(self, Directory), maxdepth=None)

    def resolve(self):
        path = self.path.resolve()
        if path.isdir():
            return Directory(path=path)
        return File(path=path)

    def match(self, pattern: str, *, name_only: bool = True, invert: bool = False) -> bool:
        if name_only:
            return fnmatch(self.name, pattern) ^ invert
        return fnmatch(str(self), pattern) ^ invert

    def all_match(self, pattern: str, *, name_only: bool = True, invert: bool = False) -> bool:
        if isinstance(self, Directory):
            return [_ for _ in self.ls() if _.match(pattern, name_only=name_only, invert=invert)]
        return self.match(pattern, name_only=name_only, invert=invert)

    def rematch(self, re: str, *, name_only: bool = True, invert: bool = False) -> bool:
        if name_only:
            return (re_match(re, self.name) is not None) ^ invert
        return (re_match(re, str(self)) is not None) ^ invert

    def all_rematch(self, re: str, *, name_only: bool = True, invert: bool = False) -> bool:
        if isinstance(self, Directory):
            return [_ for _ in self.ls() if _.rematch(re, name_only=name_only, invert=invert)]
        return self.rematch(re, name_only=name_only, invert=invert)

    # Convenience
    @property
    def fs(self):
        return self.path.fs

    @property
    def localpath(self):
        return self.path.path

    # Builtins
    @property
    def name(self):
        """The final path component, if any."""
        return self.path.path.name

    @property
    def suffix(self):
        """
        The final component's last suffix, if any.

        This includes the leading period. For example: '.txt'
        """
        return self.path.path.suffix

    @property
    def stem(self):
        """The final path component, minus its last suffix."""
        return self.path.path.stem

    @property
    def parts(self):
        """An object providing sequence-like access to the
        components in the filesystem path."""
        return self.path.path.parts

    @property
    def parent(self):
        """The logical parent of the path."""
        return Directory(path=BasePath(fs=self.path.fs, path=self.path.path.parent))

    def modified(self) -> datetime:
        return self.path.fs.modified(self.path.path)

    # Overlapping
    def __truediv__(self, other):
        return Path(BasePath(fs=self.path.fs, path=self.path.path / other)).resolve()


class Path(SharedAPI, BaseModel):
    def __new__(cls, path):
        # Opt for File, unless we can tell its a dir
        return File(path=path).resolve()


class Directory(SharedAPI, BaseModel):
    path: DirectoryPath

    def __repr__(self) -> str:
        return f"Directory(path={str(self.path)})"

    def ls(self):
        # TODO: sort?
        # TODO: make fast for large dirs?
        # this will autoresolve types correctly
        paths = sorted(BasePath(fs=self.path.fs, path=path) for path in self.path.fs.ls(self.path.path))
        return [Directory(path=path) if path.isdir() else File(path=path) for path in paths]

    def list(self):
        return self.path.fs.listdir(self.path.path)

    def __len__(self) -> int:
        return len(self.list())

    def _recurse_gen(self):
        # this will autoresolve types correctly
        for file_or_dir in self.ls():
            if file_or_dir.path.isdir():
                for path in file_or_dir._recurse_gen():
                    yield path
            else:
                yield file_or_dir

    def recurse(self):
        # this will autoresolve types correctly
        return list(self._recurse_gen())

    def size(self, block_size: int = 4096) -> int:
        size = 0
        for elem in self.ls():
            try:
                size += elem.size(block_size=block_size)
            except FileNotFoundError:
                # File removed or bad symlink, ignore
                continue
        return size

    def apply(self, foo, include_dir: bool = False, recursive: bool = False): ...


class File(SharedAPI, BaseModel):
    path: FilePath

    def __repr__(self) -> str:
        return f"File(path={str(self.path)})"

    def size(self, block_size: int = 4096) -> int:
        return max(self.path.fs.size(self.path.path), block_size)
