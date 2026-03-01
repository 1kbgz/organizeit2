from fnmatch import fnmatch
from os import symlink, unlink
from re import match as re_match

from ccflow import BaseModel
from fsspec.implementations.local import LocalFileSystem
from fsspec_pydantic import DirectoryPath, FilePath, Path as BasePath

from .organizeit2 import Directory as _RustDirectory, File as _RustFile

__all__ = (
    "Directory",
    "File",
    "Path",
)


def _is_local(path_str):
    s = str(path_str)
    return s.startswith(("file://", "local://", "/", ".")) or "://" not in s


# --- Fsspec backend for remote filesystems ---


class _FsspecMixin:
    def __str__(self):
        return str(self.path)

    def str(self):
        return str(self)

    def __hash__(self):
        return hash(str(self))

    def __lt__(self, other):
        return str(self) < str(other)

    def exists(self):
        return self.path.fs.exists(self.path.path)

    def as_posix(self):
        return self.path.path.as_posix()

    def _can_link(self):
        return hasattr(self.path.fs, "link")

    def link(self, other, soft=True):
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
        self.path.fs.rm(self.path.path, recursive=isinstance(self, _FsspecDirectory), maxdepth=None)

    def resolve(self):
        path = self.path.resolve()
        if path.isdir():
            return _FsspecDirectory(path=path)
        return _FsspecFile(path=path)

    def match(self, pattern, *, name_only=True, invert=False):
        if name_only:
            return fnmatch(self.name, pattern) ^ invert
        return fnmatch(str(self), pattern) ^ invert

    def all_match(self, pattern, *, name_only=True, invert=False):
        if isinstance(self, _FsspecDirectory):
            return [_ for _ in self.ls() if _.match(pattern, name_only=name_only, invert=invert)]
        return self.match(pattern, name_only=name_only, invert=invert)

    def rematch(self, re, *, name_only=True, invert=False):
        if name_only:
            return (re_match(re, self.name) is not None) ^ invert
        return (re_match(re, str(self)) is not None) ^ invert

    def all_rematch(self, re, *, name_only=True, invert=False):
        if isinstance(self, _FsspecDirectory):
            return [_ for _ in self.ls() if _.rematch(re, name_only=name_only, invert=invert)]
        return self.rematch(re, name_only=name_only, invert=invert)

    @property
    def name(self):
        return self.path.path.name

    @property
    def suffix(self):
        return self.path.path.suffix

    @property
    def stem(self):
        return self.path.path.stem

    @property
    def parts(self):
        return self.path.path.parts

    @property
    def parent(self):
        return _FsspecDirectory(path=BasePath(fs=self.path.fs, path=self.path.path.parent))

    def modified(self):
        return self.path.fs.modified(self.path.path)

    def __truediv__(self, other):
        return _FsspecFile(path=BasePath(fs=self.path.fs, path=self.path.path / other)).resolve()


class _FsspecDirectory(_FsspecMixin, BaseModel):
    path: DirectoryPath

    def __repr__(self):
        return f"Directory(path={str(self.path)})"

    def ls(self):
        paths = sorted(BasePath(fs=self.path.fs, path=path) for path in self.path.fs.ls(self.path.path))
        return [_FsspecDirectory(path=path) if path.isdir() else _FsspecFile(path=path) for path in paths]

    def list(self):
        return self.path.fs.listdir(self.path.path)

    def __len__(self):
        return len(self.list())

    def _recurse_gen(self):
        for file_or_dir in self.ls():
            if file_or_dir.path.isdir():
                for path in file_or_dir._recurse_gen():
                    yield path
            else:
                yield file_or_dir

    def recurse(self):
        return list(self._recurse_gen())

    def size(self, block_size=4096):
        size = 0
        for elem in self.ls():
            try:
                size += elem.size(block_size=block_size)
            except FileNotFoundError:
                continue
        return size


class _FsspecFile(_FsspecMixin, BaseModel):
    path: FilePath

    def __repr__(self):
        return f"File(path={str(self.path)})"

    def size(self, block_size=4096):
        return max(self.path.fs.size(self.path.path), block_size)


# --- Wrapping helpers ---


def _wrap(obj):
    if isinstance(obj, (_RustDirectory, _FsspecDirectory)):
        d = object.__new__(Directory)
        d._inner = obj
        return d
    if isinstance(obj, (_RustFile, _FsspecFile)):
        f = object.__new__(File)
        f._inner = obj
        return f
    return obj


def _wrap_list(lst):
    return [_wrap(x) for x in lst]


# --- Public API: dispatch local→Rust, remote→fsspec ---


class Directory:
    def __init__(self, path=None):
        if path is None:
            raise ValueError("path is required")
        path_str = str(path)
        if _is_local(path_str):
            self._inner = _RustDirectory(path=path_str)
        else:
            self._inner = _FsspecDirectory(path=path)

    def __str__(self):
        return str(self._inner)

    def __repr__(self):
        return repr(self._inner)

    def __hash__(self):
        return hash(str(self._inner))

    def __eq__(self, other):
        if isinstance(other, Directory):
            return str(self._inner) == str(other._inner)
        return NotImplemented

    def __lt__(self, other):
        if isinstance(other, Directory):
            return str(self._inner) < str(other._inner)
        return NotImplemented

    def __len__(self):
        return len(self._inner)

    def __truediv__(self, other):
        return _wrap(self._inner / other)

    def str(self):
        return self._inner.str()

    def exists(self):
        return self._inner.exists()

    def as_posix(self):
        return self._inner.as_posix()

    @property
    def name(self):
        return self._inner.name

    @property
    def suffix(self):
        return self._inner.suffix

    @property
    def stem(self):
        return self._inner.stem

    @property
    def parts(self):
        return self._inner.parts

    @property
    def parent(self):
        return _wrap(self._inner.parent)

    def modified(self):
        return self._inner.modified()

    def resolve(self):
        return _wrap(self._inner.resolve())

    def match(self, pattern, *, name_only=True, invert=False):
        return self._inner.match(pattern, name_only=name_only, invert=invert)

    def all_match(self, pattern, *, name_only=True, invert=False):
        result = self._inner.all_match(pattern, name_only=name_only, invert=invert)
        return _wrap_list(result) if isinstance(result, list) else result

    def rematch(self, re, *, name_only=True, invert=False):
        return self._inner.rematch(re, name_only=name_only, invert=invert)

    def all_rematch(self, re, *, name_only=True, invert=False):
        result = self._inner.all_rematch(re, name_only=name_only, invert=invert)
        return _wrap_list(result) if isinstance(result, list) else result

    def link(self, other, soft=True):
        other_inner = other._inner if isinstance(other, Directory) else other
        return self._inner.link(other_inner, soft=soft)

    def unlink(self):
        return self._inner.unlink()

    def rm(self):
        return self._inner.rm()

    def ls(self):
        return _wrap_list(self._inner.ls())

    def list(self):
        return self._inner.list()

    def recurse(self):
        return _wrap_list(self._inner.recurse())

    def size(self, block_size=4096):
        return self._inner.size(block_size=block_size)


class File:
    def __init__(self, path=None):
        if path is None:
            raise ValueError("path is required")
        path_str = str(path)
        if _is_local(path_str):
            self._inner = _RustFile(path=path_str)
        else:
            self._inner = _FsspecFile(path=path)

    def __str__(self):
        return str(self._inner)

    def __repr__(self):
        return repr(self._inner)

    def __hash__(self):
        return hash(str(self._inner))

    def __eq__(self, other):
        if isinstance(other, File):
            return str(self._inner) == str(other._inner)
        return NotImplemented

    def __lt__(self, other):
        if isinstance(other, File):
            return str(self._inner) < str(other._inner)
        return NotImplemented

    def __truediv__(self, other):
        return _wrap(self._inner / other)

    def str(self):
        return self._inner.str()

    def exists(self):
        return self._inner.exists()

    def as_posix(self):
        return self._inner.as_posix()

    @property
    def name(self):
        return self._inner.name

    @property
    def suffix(self):
        return self._inner.suffix

    @property
    def stem(self):
        return self._inner.stem

    @property
    def parts(self):
        return self._inner.parts

    @property
    def parent(self):
        return _wrap(self._inner.parent)

    def modified(self):
        return self._inner.modified()

    def resolve(self):
        return _wrap(self._inner.resolve())

    def match(self, pattern, *, name_only=True, invert=False):
        return self._inner.match(pattern, name_only=name_only, invert=invert)

    def rematch(self, re, *, name_only=True, invert=False):
        return self._inner.rematch(re, name_only=name_only, invert=invert)

    def link(self, other, soft=True):
        other_inner = other._inner if isinstance(other, File) else other
        return self._inner.link(other_inner, soft=soft)

    def unlink(self):
        return self._inner.unlink()

    def rm(self):
        return self._inner.rm()

    def size(self, block_size=4096):
        return self._inner.size(block_size=block_size)


def Path(path):
    return File(path=path).resolve()
