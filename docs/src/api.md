# API

## `Directory`

Represents a directory on the local filesystem. Backed by Rust via PyO3.

### Constructor

`Directory(path)` — Create a directory reference from a path string. Accepts `file://`, `local://` prefixed paths or plain filesystem paths.

### Methods

| Method                                                | Description                                                                            |
| ----------------------------------------------------- | -------------------------------------------------------------------------------------- |
| `ls()`                                                | List immediate children, sorted alphabetically. Returns `list[Directory \| File]`.     |
| `list()`                                              | List child names as strings.                                                           |
| `recurse()`                                           | Recursively list all files under this directory. Returns `list[File]`.                 |
| `size(block_size=4096)`                               | Total size in bytes (each file is rounded up to `block_size`).                         |
| `resolve()`                                           | Canonicalize the path; returns `Directory` or `File` depending on what exists on disk. |
| `match(pattern, *, name_only=True, invert=False)`     | fnmatch-style glob test.                                                               |
| `all_match(pattern, *, name_only=True, invert=False)` | Return children matching the glob.                                                     |
| `rematch(re, *, name_only=True, invert=False)`        | Regex match (anchored at start, like Python `re.match`).                               |
| `all_rematch(re, *, name_only=True, invert=False)`    | Return children matching the regex.                                                    |
| `link(other, *, soft=True)`                           | Create a symlink (or hard link) from `self` to `other`.                                |
| `unlink()`                                            | Remove a symlink.                                                                      |
| `rm()`                                                | Remove the directory recursively.                                                      |
| `exists()`                                            | Whether the path exists on disk.                                                       |
| `as_posix()`                                          | Raw POSIX path string (no `file://` prefix).                                           |
| `str()`                                               | Same as `str(directory)` — the `file://`-prefixed path.                                |
| `modified()`                                          | Last-modified time as `datetime.datetime`.                                             |

### Properties

| Property | Description                               |
| -------- | ----------------------------------------- |
| `name`   | Final path component.                     |
| `suffix` | File extension including the leading dot. |
| `stem`   | Final component without the extension.    |
| `parts`  | All path components as a list of strings. |
| `parent` | Parent as a `Directory`.                  |

### Operators

| Operator      | Description                           |
| ------------- | ------------------------------------- |
| `d / "child"` | Join a path component and resolve.    |
| `len(d)`      | Number of immediate children.         |
| `str(d)`      | `file://`-prefixed path string.       |
| `repr(d)`     | `Directory(path=...)` representation. |
| `hash(d)`     | Hash based on display path.           |
| `d1 == d2`    | Equality based on path.               |
| `d1 < d2`     | Lexicographic ordering.               |

______________________________________________________________________

## `File`

Represents a file on the local filesystem. Backed by Rust via PyO3.

### Constructor

`File(path)` — Create a file reference from a path string. Accepts `file://`, `local://` prefixed paths or plain filesystem paths.

### Methods

| Method                                            | Description                                                                            |
| ------------------------------------------------- | -------------------------------------------------------------------------------------- |
| `size(block_size=4096)`                           | File size in bytes, rounded up to `block_size`.                                        |
| `resolve()`                                       | Canonicalize the path; returns `Directory` or `File` depending on what exists on disk. |
| `match(pattern, *, name_only=True, invert=False)` | fnmatch-style glob test.                                                               |
| `rematch(re, *, name_only=True, invert=False)`    | Regex match (anchored at start).                                                       |
| `link(other, *, soft=True)`                       | Create a symlink (or hard link) from `self` to `other`.                                |
| `unlink()`                                        | Remove a symlink.                                                                      |
| `rm()`                                            | Remove the file.                                                                       |
| `exists()`                                        | Whether the path exists on disk.                                                       |
| `as_posix()`                                      | Raw POSIX path string.                                                                 |
| `str()`                                           | Same as `str(file)`.                                                                   |
| `modified()`                                      | Last-modified time as `datetime.datetime`.                                             |

### Properties

| Property | Description                                  |
| -------- | -------------------------------------------- |
| `name`   | Final path component (e.g. `"data.csv"`).    |
| `suffix` | Extension including the dot (e.g. `".csv"`). |
| `stem`   | Name without the extension (e.g. `"data"`).  |
| `parts`  | All path components as a list of strings.    |
| `parent` | Parent as a `Directory`.                     |

### Operators

| Operator      | Description                        |
| ------------- | ---------------------------------- |
| `f / "child"` | Join a path component and resolve. |
| `str(f)`      | `file://`-prefixed path string.    |
| `repr(f)`     | `File(path=...)` representation.   |
| `hash(f)`     | Hash based on display path.        |
| `f1 == f2`    | Equality based on path.            |
| `f1 < f2`     | Lexicographic ordering.            |

______________________________________________________________________

## `Path`

Factory function that resolves a path string to either a `File` or `Directory`.

```python
from organizeit2 import Path

entry = Path(path="/tmp/mydir")  # returns Directory if path is a dir, else File
```

______________________________________________________________________

## `OrganizeIt`

Entry-point class for filesystem organization. Uses [fsspec](https://filesystem-spec.readthedocs.io/) for filesystem abstraction.

### Constructor

`OrganizeIt(fs=LocalFileSystem())` — Create an organizer with the given filesystem backend.

### Methods

| Method              | Description                                                                       |
| ------------------- | --------------------------------------------------------------------------------- |
| `expand(directory)` | Convert a directory path string to a `Directory` using the configured filesystem. |
