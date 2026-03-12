# Rust

`organizeit2` provides a pure-Rust implementation of the core types and operations,
along with Python bindings via PyO3.

## Crate structure

| Crate path           | Purpose                                  |
| -------------------- | ---------------------------------------- |
| `rust/src/lib.rs`    | Pure Rust library (`organizeit2` crate)  |
| `rust/python/lib.rs` | PyO3 Python bindings (root `Cargo.toml`) |

## Core types

### `File`

Represents a file on the local filesystem.

```rust
use organizeit2::{File, PathLike};

let f = File::new("file:///tmp/data.csv");
assert_eq!(f.name(), "data.csv");
assert_eq!(f.suffix(), ".csv");
assert_eq!(f.stem(), "data");
```

### `Directory`

Represents a directory on the local filesystem.

```rust
use organizeit2::{Directory, PathLike};

let d = Directory::new("file:///tmp/mydir");
for entry in d.ls() {
    println!("{}", entry);
}
// Recursive file listing
let all_files = d.recurse();
```

### `Entry`

An enum over `File` and `Directory`, returned by operations
that may yield either type (e.g. `ls`, `resolve`).

### `OrganizeIt`

Entry-point helper that expands a directory path string.

```rust
use organizeit2::OrganizeIt;

let oi = OrganizeIt::new();
let d = oi.expand("/tmp/mydir");
```

## Shared trait – `PathLike`

Both `File` and `Directory` implement the `PathLike` trait which provides:

| Method                                   | Description                                               |
| ---------------------------------------- | --------------------------------------------------------- |
| `display_path()`                         | `file://`-prefixed string                                 |
| `as_posix()`                             | Raw POSIX path string                                     |
| `exists()`                               | Whether the path exists on disk                           |
| `name()`                                 | Final path component                                      |
| `suffix()`                               | File extension including the dot                          |
| `stem()`                                 | Final component without the extension                     |
| `parts()`                                | All path components as strings                            |
| `parent()`                               | Parent directory                                          |
| `modified()`                             | Last-modified time (`SystemTime`)                         |
| `resolve()`                              | Canonicalize and return the correct `Entry` variant       |
| `match_glob(pattern, name_only, invert)` | fnmatch-style glob matching                               |
| `match_re(re, name_only, invert)`        | Regex matching (anchored at start like Python `re.match`) |
| `link_to(target, soft)`                  | Create a symlink or hard link                             |
| `unlink()`                               | Remove a symlink                                          |
| `join(other)`                            | Join a path component and resolve                         |

## Utility functions

| Function                 | Description                                     |
| ------------------------ | ----------------------------------------------- |
| `parse_path(s)`          | Strip `file://` / `local://` prefix → `PathBuf` |
| `format_path(p)`         | Add `file://` prefix                            |
| `fnmatch(name, pattern)` | Python-compatible glob matching                 |
| `resolve_path(s)`        | Parse + resolve → `Entry`                       |

## Python bindings

The compiled extension exposes `Directory`, `File`, `OrganizeIt`, and a
`Path` factory function with the same API as the pure-Python types.

```python
from organizeit2 import Directory, File, OrganizeIt

d = Directory(path="/tmp/mydir")
for entry in d.ls():
    print(entry)
```
