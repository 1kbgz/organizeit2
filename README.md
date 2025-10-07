# organizeit2

Engage with Zorp!

[![Build Status](https://github.com/1kbgz/organizeit2/actions/workflows/build.yaml/badge.svg?branch=main&event=push)](https://github.com/1kbgz/organizeit2/actions/workflows/build.yaml)
[![codecov](https://codecov.io/gh/1kbgz/organizeit2/branch/main/graph/badge.svg)](https://codecov.io/gh/1kbgz/organizeit2)
[![License](https://img.shields.io/github/license/1kbgz/organizeit2)](https://github.com/1kbgz/organizeit2)
[![PyPI](https://img.shields.io/pypi/v/organizeit2.svg)](https://pypi.python.org/pypi/organizeit2)

> This morning at dawn, you will take a new form - that of a fleshless, chattering skeleton when Zorp the Surveyor arrives and burns your flesh off with his volcano mouth ~Lou Prozotovich

`OrganizeIt2` is a python library for managing large numbers of files and directories. It is type- and configuration-driven with [pydantic](https://docs.pydantic.dev/latest/).

The name is because `organizeit` was [taken on pypi](https://pypi.org/project/organizeit/), and is thus a reference to the [joke from Parks and Rec](https://parksandrecreation.fandom.com/wiki/The_Reasonabilists).

## Overview

`OrganizeIt2` has the following models and types:

- `FileSystem`: `pydantic` wrapper of an `fsspec` `AbstractFileSystem`
- `Path`: wrapper of an `fsspec` path
- `FilePath`: specialization of a `Path` for files
- `DirectoryPath`: specialization of a `Path` for directories
- `OrganizeIt`: Top-level `pydantic` model representiing an `fsspec` directory
- `Directory`: `pydantic` model representing an `fsspec` directory
- `File`: `pydantic` model representing an `fsspec` file

## CLI Examples

There are two main commands: `match` and `rematch`, which perform glob-based matching and regex-based matching respectively.

```bash
> organizeit match --help

 Usage: organizeit match [OPTIONS] DIRECTORY PATTERN

╭─ Arguments ──────────────────────────────────────────────────────────────────────────╮
│ *    directory      TEXT  [required]                                                 │
│ *    pattern        TEXT  [required]                                                 │
╰──────────────────────────────────────────────────────────────────────────────────────╯
╭─ Options ────────────────────────────────────────────────────────────────────────────╮
│ --list           -l  --no-list           -L              [default: no-list]          │
│ --name-only      -n  --no-name-only      -N              [default: name-only]        │
│ --invert         -i  --no-invert         -I              [default: no-invert]        │
│ --size           -s  --no-size           -S              [default: no-size]          │
│ --modified       -m  --no-modified       -M              [default: no-modified]      │
│ --limit                                      INTEGER                                 │
│ --leaves                                     INTEGER                                 │
│ --by                                         TEXT                                    │
│ --desc                                                                               │
│ --block-size                                 INTEGER     [default: 0]                │
│ --op                                         [rm|touch]                              │
│ --dry-run        -d  --no-dry-run        -D              [default: no-dry-run]       │
│ --ignore-errors      --no-ignore-errors                  [default: no-ignore-errors] │
│ --retries                                    INTEGER     [default: 1]                │
│ --help                                                   Show this message and exit. │
╰──────────────────────────────────────────────────────────────────────────────────────╯
```

```bash
> organizeit rematch --help

 Usage: organizeit rematch [OPTIONS] DIRECTORY PATTERN

╭─ Arguments ──────────────────────────────────────────────────────────────────────────╮
│ *    directory      TEXT  [required]                                                 │
│ *    pattern        TEXT  [required]                                                 │
╰──────────────────────────────────────────────────────────────────────────────────────╯
╭─ Options ────────────────────────────────────────────────────────────────────────────╮
│ --list           -l  --no-list           -L              [default: no-list]          │
│ --name-only      -n  --no-name-only      -N              [default: name-only]        │
│ --invert         -i  --no-invert         -I              [default: no-invert]        │
│ --size           -s  --no-size           -S              [default: no-size]          │
│ --modified       -m  --no-modified       -M              [default: no-modified]      │
│ --limit                                      INTEGER                                 │
│ --leaves                                     INTEGER                                 │
│ --by                                         TEXT                                    │
│ --desc                                                                               │
│ --block-size                                 INTEGER     [default: 0]                │
│ --op                                         [rm|touch]                              │
│ --dry-run        -d  --no-dry-run        -D              [default: no-dry-run]       │
│ --ignore-errors      --no-ignore-errors                  [default: no-ignore-errors] │
│ --retries                                    INTEGER     [default: 1]                │
│ --help                                                   Show this message and exit. │
╰──────────────────────────────────────────────────────────────────────────────────────╯
```

Here are some example commands and outputs, based on the test suite

```bash
> find organizeit2/tests/directory
.
organizeit2/tests/directory
organizeit2/tests/directory/subdir1
organizeit2/tests/directory/subdir1/file1.md
organizeit2/tests/directory/subdir1/file1.png
organizeit2/tests/directory/subdir1/file2.png
organizeit2/tests/directory/subdir1/file2.txt
organizeit2/tests/directory/subdir1/file2.md
organizeit2/tests/directory/subdir1/subsubdir1
organizeit2/tests/directory/subdir1/subsubdir1/file1
organizeit2/tests/directory/subdir1/subsubdir1/file1.md
organizeit2/tests/directory/subdir1/subsubdir1/file1.png
organizeit2/tests/directory/subdir1/subsubdir1/file1.txt
organizeit2/tests/directory/subdir1/subsubdir1/file2
organizeit2/tests/directory/subdir1/subsubdir1/file2.md
organizeit2/tests/directory/subdir1/subsubdir1/file2.png
organizeit2/tests/directory/subdir1/subsubdir1/file2.txt
organizeit2/tests/directory/subdir1/subsubdir2
organizeit2/tests/directory/subdir1/subsubdir2/file1
organizeit2/tests/directory/subdir1/subsubdir2/file1.md
organizeit2/tests/directory/subdir1/subsubdir2/file1.png
organizeit2/tests/directory/subdir1/subsubdir2/file1.txt
organizeit2/tests/directory/subdir1/subsubdir2/file2
organizeit2/tests/directory/subdir1/subsubdir2/file2.md
organizeit2/tests/directory/subdir1/subsubdir2/file2.png
organizeit2/tests/directory/subdir1/subsubdir2/file2.txt
organizeit2/tests/directory/subdir2
organizeit2/tests/directory/subdir2/file1.md
organizeit2/tests/directory/subdir2/file1.png
organizeit2/tests/directory/subdir2/file2.png
organizeit2/tests/directory/subdir2/file2.txt
organizeit2/tests/directory/subdir2/file2.md
organizeit2/tests/directory/subdir2/subsubdir1
organizeit2/tests/directory/subdir2/subsubdir1/file1
organizeit2/tests/directory/subdir2/subsubdir1/file1.md
organizeit2/tests/directory/subdir2/subsubdir1/file1.png
organizeit2/tests/directory/subdir2/subsubdir1/file1.txt
organizeit2/tests/directory/subdir2/subsubdir1/file2
organizeit2/tests/directory/subdir2/subsubdir1/file2.md
organizeit2/tests/directory/subdir2/subsubdir1/file2.png
organizeit2/tests/directory/subdir2/subsubdir1/file2.txt
organizeit2/tests/directory/subdir2/subsubdir2
organizeit2/tests/directory/subdir2/subsubdir2/file1
organizeit2/tests/directory/subdir2/subsubdir2/file1.md
organizeit2/tests/directory/subdir2/subsubdir2/file1.png
organizeit2/tests/directory/subdir2/subsubdir2/file1.txt
organizeit2/tests/directory/subdir2/subsubdir2/file2
organizeit2/tests/directory/subdir2/subsubdir2/file2.md
organizeit2/tests/directory/subdir2/subsubdir2/file2.png
organizeit2/tests/directory/subdir2/subsubdir2/file2.txt
organizeit2/tests/directory/subdir3
organizeit2/tests/directory/subdir3/file1.md
organizeit2/tests/directory/subdir3/file1.png
organizeit2/tests/directory/subdir3/file2.png
organizeit2/tests/directory/subdir3/file2.txt
organizeit2/tests/directory/subdir3/file2.md
organizeit2/tests/directory/subdir3/subsubdir1
organizeit2/tests/directory/subdir3/subsubdir1/file1
organizeit2/tests/directory/subdir3/subsubdir1/file1.md
organizeit2/tests/directory/subdir3/subsubdir1/file1.png
organizeit2/tests/directory/subdir3/subsubdir1/file1.txt
organizeit2/tests/directory/subdir3/subsubdir1/file2
organizeit2/tests/directory/subdir3/subsubdir1/file2.md
organizeit2/tests/directory/subdir3/subsubdir1/file2.png
organizeit2/tests/directory/subdir3/subsubdir1/file2.txt
organizeit2/tests/directory/subdir3/subsubdir2
organizeit2/tests/directory/subdir3/subsubdir2/file1
organizeit2/tests/directory/subdir3/subsubdir2/file1.md
organizeit2/tests/directory/subdir3/subsubdir2/file1.png
organizeit2/tests/directory/subdir3/subsubdir2/file1.txt
organizeit2/tests/directory/subdir3/subsubdir2/file2
organizeit2/tests/directory/subdir3/subsubdir2/file2.md
organizeit2/tests/directory/subdir3/subsubdir2/file2.png
organizeit2/tests/directory/subdir3/subsubdir2/file2.txt
organizeit2/tests/directory/subdir4
organizeit2/tests/directory/subdir4/file1.md
organizeit2/tests/directory/subdir4/file1.png
organizeit2/tests/directory/subdir4/file2.png
organizeit2/tests/directory/subdir4/file2.txt
organizeit2/tests/directory/subdir4/file2.md
organizeit2/tests/directory/subdir4/subsubdir1
organizeit2/tests/directory/subdir4/subsubdir1/file1
organizeit2/tests/directory/subdir4/subsubdir1/file1.md
organizeit2/tests/directory/subdir4/subsubdir1/file1.png
organizeit2/tests/directory/subdir4/subsubdir1/file1.txt
organizeit2/tests/directory/subdir4/subsubdir1/file2
organizeit2/tests/directory/subdir4/subsubdir1/file2.md
organizeit2/tests/directory/subdir4/subsubdir1/file2.png
organizeit2/tests/directory/subdir4/subsubdir1/file2.txt
organizeit2/tests/directory/subdir4/subsubdir2
organizeit2/tests/directory/subdir4/subsubdir2/file1
organizeit2/tests/directory/subdir4/subsubdir2/file1.md
organizeit2/tests/directory/subdir4/subsubdir2/file1.png
organizeit2/tests/directory/subdir4/subsubdir2/file1.txt
organizeit2/tests/directory/subdir4/subsubdir2/file2
organizeit2/tests/directory/subdir4/subsubdir2/file2.md
organizeit2/tests/directory/subdir4/subsubdir2/file2.png
organizeit2/tests/directory/subdir4/subsubdir2/file2.txt
```

<!--
#!/bin/bash

for cmd in \
    'organizeit match file://organizeit2/tests/directory/ "directory*"' \
    'organizeit match file://organizeit2/tests/directory/ "directory*" --invert' \
    'organizeit match file://organizeit2/tests/directory/ "directory" --no-name-only' \
    'organizeit match file://organizeit2/tests/directory/ "directory" --no-name-only --invert' \
    'organizeit match file://organizeit2/tests/directory/ "*organizeit2*directory" --no-name-only' \
    'organizeit match file://organizeit2/tests/directory/ "*organizeit2*directory" --no-name-only --invert' \
    'organizeit match file://organizeit2/tests/directory/ "subdir*"' \
    'organizeit match file://organizeit2/tests/directory/ "dir*"' \
    'organizeit match file://organizeit2/tests/directory/ "subdir*" --list --invert' \
    'organizeit rematch file://organizeit2/tests/directory/ "directory"' \
    'organizeit rematch file://organizeit2/tests/directory/ "directory" --invert' \
    'organizeit rematch file://organizeit2/tests/directory/ "directory" --no-name-only' \
    'organizeit rematch file://organizeit2/tests/directory/ "directory" --no-name-only --invert' \
    'organizeit rematch file://organizeit2/tests/directory/ "file://[a-zA-Z0-9/]*" --no-name-only' \
    'organizeit rematch file://organizeit2/tests/directory/ "subdir[0-9]+"' \
    'organizeit rematch file://organizeit2/tests/directory/ "subdir[0-3]+"' \
    'organizeit rematch file://organizeit2/tests/directory/ "subdir*" --list --invert' \
    'organizeit match file://organizeit2/tests/directory/subdir1/ "*" --list --limit=2 --leaves=7 --invert --by="age"' \
    'organizeit rematch file://organizeit2/tests/directory/subdir1/ ".*" --no-list --limit=2 --leaves=7 --invert --by="age"' \
    'organizeit match file://organizeit2/tests/directory/subdir1/ "*" --no-list --limit=5 --invert --by="size" --desc' \
    'organizeit rematch file://organizeit2/tests/directory/subdir1/ ".*" --no-list --limit=5 --invert --by="size" --desc' \
    'organizeit match file://organizeit2/tests/directory/subdir1/ "*" --no-list --limit=2 --leaves=7 --invert --desc --by="age"' \
    'organizeit rematch file://organizeit2/tests/directory/subdir1/ ".*" --no-list --limit=2 --leaves=7 --invert --desc --by="age"' \
    'organizeit match file://organizeit2/tests/directory/subdir1/ "*" --no-list --limit=3 --invert --by="age"' \
    'organizeit rematch file://organizeit2/tests/directory/subdir1/ ".*" --no-list --limit=3 --invert --by="age"' \
    'organizeit match file://organizeit2/tests/directory/subdir1/ "*" --no-list --leaves=8 --invert --by="age"' \
    'organizeit rematch file://organizeit2/tests/directory/subdir1/ ".*" --no-list --leaves=8 --invert --by="age"' \
    'organizeit rematch file://organizeit2/tests/directory/subdir1/ ".*" --no-list --limit=5 --invert --by="size" --desc --op="touch" --dry-run' \
    'organizeit rematch file://organizeit2/tests/directory/subdir1/ ".*" --no-list --limit=5 --invert --by="size" --desc --op="rm" --dry-run' \
; do
    printf '```bash\n%s\necho $?\n```\n\n' "$cmd" >> README.md
    printf '```raw\n' >> README.md
    eval "$cmd" >> README.md
    echo $? >> README.md
    printf '\n```\n\n' >> README.md
done
-->

```bash
organizeit match file://organizeit2/tests/directory/ "directory*"
echo $?
```

```raw
Unmatched
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Path                                                   ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ file://organizeit2/organizeit2/tests/directory/subdir2 │
│ file://organizeit2/organizeit2/tests/directory/subdir4 │
│ file://organizeit2/organizeit2/tests/directory/subdir3 │
│ file://organizeit2/organizeit2/tests/directory/subdir1 │
└────────────────────────────────────────────────────────┘
1

```

```bash
organizeit match file://organizeit2/tests/directory/ "directory*" --invert
echo $?
```

```raw
All matched
0

```

```bash
organizeit match file://organizeit2/tests/directory/ "directory" --no-name-only
echo $?
```

```raw
Unmatched
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Path                                                   ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ file://organizeit2/organizeit2/tests/directory/subdir3 │
│ file://organizeit2/organizeit2/tests/directory/subdir1 │
│ file://organizeit2/organizeit2/tests/directory/subdir2 │
│ file://organizeit2/organizeit2/tests/directory/subdir4 │
└────────────────────────────────────────────────────────┘
1

```

```bash
organizeit match file://organizeit2/tests/directory/ "directory" --no-name-only --invert
echo $?
```

```raw
All matched
0

```

```bash
organizeit match file://organizeit2/tests/directory/ "*organizeit2*directory" --no-name-only
echo $?
```

```raw
Unmatched
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Path                                                   ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ file://organizeit2/organizeit2/tests/directory/subdir3 │
│ file://organizeit2/organizeit2/tests/directory/subdir4 │
│ file://organizeit2/organizeit2/tests/directory/subdir2 │
│ file://organizeit2/organizeit2/tests/directory/subdir1 │
└────────────────────────────────────────────────────────┘
1

```

```bash
organizeit match file://organizeit2/tests/directory/ "*organizeit2*directory" --no-name-only --invert
echo $?
```

```raw
All matched
0

```

```bash
organizeit match file://organizeit2/tests/directory/ "subdir*"
echo $?
```

```raw
All matched
0

```

```bash
organizeit match file://organizeit2/tests/directory/ "dir*"
echo $?
```

```raw
Unmatched
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Path                                                   ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ file://organizeit2/organizeit2/tests/directory/subdir3 │
│ file://organizeit2/organizeit2/tests/directory/subdir1 │
│ file://organizeit2/organizeit2/tests/directory/subdir2 │
│ file://organizeit2/organizeit2/tests/directory/subdir4 │
└────────────────────────────────────────────────────────┘
1

```

```bash
organizeit match file://organizeit2/tests/directory/ "subdir*" --list --invert
echo $?
```

```raw
organizeit2/organizeit2/tests/directory/subdir2
organizeit2/organizeit2/tests/directory/subdir4
organizeit2/organizeit2/tests/directory/subdir1
organizeit2/organizeit2/tests/directory/subdir3
1

```

```bash
organizeit rematch file://organizeit2/tests/directory/ "directory"
echo $?
```

```raw
Unmatched
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Path                                                   ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ file://organizeit2/organizeit2/tests/directory/subdir2 │
│ file://organizeit2/organizeit2/tests/directory/subdir3 │
│ file://organizeit2/organizeit2/tests/directory/subdir1 │
│ file://organizeit2/organizeit2/tests/directory/subdir4 │
└────────────────────────────────────────────────────────┘
1

```

```bash
organizeit rematch file://organizeit2/tests/directory/ "directory" --invert
echo $?
```

```raw
All matched
0

```

```bash
organizeit rematch file://organizeit2/tests/directory/ "directory" --no-name-only
echo $?
```

```raw
Unmatched
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Path                                                   ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ file://organizeit2/organizeit2/tests/directory/subdir2 │
│ file://organizeit2/organizeit2/tests/directory/subdir3 │
│ file://organizeit2/organizeit2/tests/directory/subdir1 │
│ file://organizeit2/organizeit2/tests/directory/subdir4 │
└────────────────────────────────────────────────────────┘
1

```

```bash
organizeit rematch file://organizeit2/tests/directory/ "directory" --no-name-only --invert
echo $?
```

```raw
All matched
0

```

```bash
organizeit rematch file://organizeit2/tests/directory/ "file://[a-zA-Z0-9/]*" --no-name-only
echo $?
```

```raw
All matched
0

```

```bash
organizeit rematch file://organizeit2/tests/directory/ "subdir[0-9]+"
echo $?
```

```raw
All matched
0

```

```bash
organizeit rematch file://organizeit2/tests/directory/ "subdir[0-3]+"
echo $?
```

```raw
Unmatched
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Path                                                   ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ file://organizeit2/organizeit2/tests/directory/subdir4 │
└────────────────────────────────────────────────────────┘
1

```

```bash
organizeit rematch file://organizeit2/tests/directory/ "subdir*" --list --invert
echo $?
```

```raw
organizeit2/organizeit2/tests/directory/subdir4
organizeit2/organizeit2/tests/directory/subdir1
organizeit2/organizeit2/tests/directory/subdir3
organizeit2/organizeit2/tests/directory/subdir2
1

```

```bash
organizeit match file://organizeit2/tests/directory/subdir1/ "*" --list --limit=2 --leaves=7 --invert --by="age"
echo $?
```

```raw
organizeit2/organizeit2/tests/directory/subdir1/file2.png
organizeit2/organizeit2/tests/directory/subdir1/file1.png
1

```

```bash
organizeit rematch file://organizeit2/tests/directory/subdir1/ ".*" --no-list --limit=2 --leaves=7 --invert --by="age"
echo $?
```

```raw
Unmatched
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Path                                                             ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2.png │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file1.png │
└──────────────────────────────────────────────────────────────────┘
1

```

```bash
organizeit match file://organizeit2/tests/directory/subdir1/ "*" --no-list --limit=5 --invert --by="size" --desc
echo $?
```

```raw
Unmatched
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┳━━━━━━┓
┃ Path                                                             ┃ Size ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━╇━━━━━━┩
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2.txt │ 7    │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2.md  │ 6    │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2     │ 5    │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file1.txt │ 3    │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file1.md  │ 2    │
└──────────────────────────────────────────────────────────────────┴──────┘
1

```

```bash
organizeit rematch file://organizeit2/tests/directory/subdir1/ ".*" --no-list --limit=5 --invert --by="size" --desc
echo $?
```

```raw
Unmatched
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┳━━━━━━┓
┃ Path                                                             ┃ Size ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━╇━━━━━━┩
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2.txt │ 7    │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2.md  │ 6    │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2     │ 5    │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file1.txt │ 3    │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file1.md  │ 2    │
└──────────────────────────────────────────────────────────────────┴──────┘
1

```

```bash
organizeit match file://organizeit2/tests/directory/subdir1/ "*" --no-list --limit=2 --leaves=7 --invert --desc --by="age"
echo $?
```

```raw
Unmatched
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Path                                                             ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2.txt │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2.md  │
└──────────────────────────────────────────────────────────────────┘
1

```

```bash
organizeit rematch file://organizeit2/tests/directory/subdir1/ ".*" --no-list --limit=2 --leaves=7 --invert --desc --by="age"
echo $?
```

```raw
Unmatched
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Path                                                             ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2.txt │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2.md  │
└──────────────────────────────────────────────────────────────────┘
1

```

```bash
organizeit match file://organizeit2/tests/directory/subdir1/ "*" --no-list --limit=3 --invert --by="age"
echo $?
```

```raw
Unmatched
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Path                                                              ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ file://organizeit2/organizeit2/tests/directory/subdir1/file1.png  │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2.png  │
│ file://organizeit2/organizeit2/tests/directory/subdir1/subsubdir1 │
└───────────────────────────────────────────────────────────────────┘
1

```

```bash
organizeit rematch file://organizeit2/tests/directory/subdir1/ ".*" --no-list --limit=3 --invert --by="age"
echo $?
```

```raw
Unmatched
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Path                                                              ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ file://organizeit2/organizeit2/tests/directory/subdir1/file1.png  │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2.png  │
│ file://organizeit2/organizeit2/tests/directory/subdir1/subsubdir1 │
└───────────────────────────────────────────────────────────────────┘
1

```

```bash
organizeit match file://organizeit2/tests/directory/subdir1/ "*" --no-list --leaves=8 --invert --by="age"
echo $?
```

```raw
Unmatched
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Path                                                             ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ file://organizeit2/organizeit2/tests/directory/subdir1/file1.png │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2.png │
└──────────────────────────────────────────────────────────────────┘
1

```

```bash
organizeit rematch file://organizeit2/tests/directory/subdir1/ ".*" --no-list --leaves=8 --invert --by="age"
echo $?
```

```raw
Unmatched
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┓
┃ Path                                                             ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┩
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2.png │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file1.png │
└──────────────────────────────────────────────────────────────────┘
1

```

```bash
organizeit rematch file://organizeit2/tests/directory/subdir1/ ".*" --no-list --limit=5 --invert --by="size" --desc --op="touch" --dry-run
echo $?
```

```raw
Unmatched
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┳━━━━━━┓
┃ Path                                                             ┃ Size ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━╇━━━━━━┩
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2.txt │ 7    │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2.md  │ 6    │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2     │ 5    │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file1.txt │ 3    │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file1.md  │ 2    │
└──────────────────────────────────────────────────────────────────┴──────┘
touch organizeit2/organizeit2/tests/directory/subdir1/file2.txt
touch organizeit2/organizeit2/tests/directory/subdir1/file2.md
touch organizeit2/organizeit2/tests/directory/subdir1/file2
touch organizeit2/organizeit2/tests/directory/subdir1/file1.txt
touch organizeit2/organizeit2/tests/directory/subdir1/file1.md
1

```

```bash
organizeit rematch file://organizeit2/tests/directory/subdir1/ ".*" --no-list --limit=5 --invert --by="size" --desc --op="rm" --dry-run
echo $?
```

```raw
Unmatched
┏━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┳━━━━━━┓
┃ Path                                                             ┃ Size ┃
┡━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━╇━━━━━━┩
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2.txt │ 7    │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2.md  │ 6    │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file2     │ 5    │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file1.txt │ 3    │
│ file://organizeit2/organizeit2/tests/directory/subdir1/file1.md  │ 2    │
└──────────────────────────────────────────────────────────────────┴──────┘
rm organizeit2/organizeit2/tests/directory/subdir1/file2.txt
rm organizeit2/organizeit2/tests/directory/subdir1/file2.md
rm organizeit2/organizeit2/tests/directory/subdir1/file2
rm organizeit2/organizeit2/tests/directory/subdir1/file1.txt
rm organizeit2/organizeit2/tests/directory/subdir1/file1.md
1

```

> [!NOTE]
> This library was generated using [copier](https://copier.readthedocs.io/en/stable/) from the [Base Python Project Template repository](https://github.com/python-project-templates/base)
