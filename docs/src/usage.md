# Usage

## Installation

```bash
pip install organizeit2
```

## Python Library

### Working with Directories

```python
from organizeit2 import Directory

# Create a directory reference (supports file://, local://, or plain paths)
d = Directory(path="file:///tmp/myproject")
d = Directory(path="/tmp/myproject")

# List contents
for entry in d.ls():
    print(entry.name, type(entry).__name__)

# Recursive file listing
all_files = d.recurse()
print(f"{len(all_files)} files found")

# Directory size (each file rounded up to block_size)
print(f"Size: {d.size()} bytes")
print(f"Size (1-byte blocks): {d.size(0)} bytes")
```

### Working with Files

```python
from organizeit2 import File

f = File(path="file:///tmp/myproject/README.md")
print(f.name)    # "README.md"
print(f.suffix)  # ".md"
print(f.stem)    # "README"
print(f.parent)  # Directory for /tmp/myproject
```

### Path Resolution

Use `Path` to auto-detect whether a path is a file or directory:

```python
from organizeit2 import Path

entry = Path(path="/tmp/myproject")
# Returns a Directory if the path is a directory, File otherwise
```

Use `resolve()` to canonicalize and detect the type:

```python
from organizeit2 import File

f = File(path="file:///tmp/myproject")
resolved = f.resolve()  # Returns Directory if path is actually a directory
```

### Glob Matching

```python
from organizeit2 import Directory

d = Directory(path="/tmp/myproject/")

# Match the directory name itself
d.match("myproject*")                         # True
d.match("other*")                             # False
d.match("*myproject*", name_only=False)       # Match against full path
d.match("myproject*", invert=True)            # Inverted match

# Match children
txt_files = d.all_match("*.txt")
non_txt = d.all_match("*.txt", invert=True)
```

### Regex Matching

```python
d.rematch(r"myproject")                        # Anchored at start like re.match
d.rematch(r"file://.*myproject", name_only=False)

# Match children with regex
numbered = d.all_rematch(r"file[0-9]+\.txt")
```

### Symlinks

```python
from organizeit2 import Directory

src = Directory(path="/tmp/source")
dst = Directory(path="/tmp/link_target")
src.link(dst)        # Create symlink
dst.unlink()         # Remove symlink
```

### Path Navigation

```python
from organizeit2 import File

f = File(path="file:///tmp/project/src/main.py")

# Navigate with /
parent = (f / "..").resolve()
sibling = (f / ".." / "utils.py").resolve()

# Properties
f.parent             # Directory for /tmp/project/src
f.parts              # ['/', 'tmp', 'project', 'src', 'main.py']
```

### OrganizeIt

```python
from organizeit2 import OrganizeIt

oi = OrganizeIt()                              # Uses local filesystem
oi = OrganizeIt(fs="local:///tmp/data")        # Explicit filesystem

d = oi.expand("/tmp/data")                     # Returns a Directory
```

## CLI

`organizeit2` provides a command-line interface via `oi2`.

### Match (glob)

```bash
# Check if a path matches a glob pattern
oi2 match /tmp/myproject/ "subdir*"

# Inverted match (show unmatched)
oi2 match /tmp/myproject/ "*.log" --invert

# List unmatched paths
oi2 match /tmp/myproject/ "*.py" --invert --list

# Full path matching (not just name)
oi2 match /tmp/myproject/ "*src*main*" --no-name-only
```

### Match with regex

```bash
oi2 rematch /tmp/myproject/ "subdir[0-9]+"
oi2 rematch /tmp/myproject/ "file://.*" --no-name-only
```

### Sorting and Limiting

```bash
# Oldest 5 unmatched files
oi2 match /tmp/data/ "*.keep" --invert --by age --limit 5

# Largest files, descending
oi2 match /tmp/data/ "*.keep" --invert --by size --desc --limit 10

# Keep newest 7, show the rest
oi2 match /tmp/data/ "*" --invert --by age --leaves 7
```

### Operations

```bash
# Dry run — print what would be removed
oi2 rematch /tmp/data/ ".*" --invert --by size --desc --limit 5 --op rm --dry-run

# Actually remove
oi2 rematch /tmp/data/ ".*" --invert --by size --desc --limit 5 --op rm
```
