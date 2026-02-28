use std::cmp::Ordering;
use std::fmt;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path as StdPath, PathBuf};
use std::time::SystemTime;

use regex::Regex;

// =========================================================================
// Utilities
// =========================================================================

/// Strip `file://` or `local://` protocol prefixes from a path string.
pub fn parse_path(s: &str) -> PathBuf {
    if let Some(rest) = s.strip_prefix("file://") {
        PathBuf::from(rest)
    } else if let Some(rest) = s.strip_prefix("local://") {
        PathBuf::from(rest)
    } else {
        PathBuf::from(s)
    }
}

/// Format a path with a `file://` protocol prefix.
pub fn format_path(p: &StdPath) -> String {
    format!("file://{}", p.to_string_lossy().replace('\\', "/"))
}

/// Normalize a path by resolving `.` and `..` components without touching the filesystem.
fn normalize_path(path: &StdPath) -> PathBuf {
    use std::path::Component;
    let mut components = Vec::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                if !components.is_empty() {
                    components.pop();
                }
            }
            Component::CurDir => {}
            c => components.push(c),
        }
    }
    components.iter().collect()
}

/// fnmatch-style glob matching (equivalent to Python's `fnmatch.fnmatch`).
pub fn fnmatch(name: &str, pattern: &str) -> bool {
    fnmatch_recursive(name.as_bytes(), pattern.as_bytes())
}

fn fnmatch_recursive(name: &[u8], pattern: &[u8]) -> bool {
    if pattern.is_empty() {
        return name.is_empty();
    }
    if name.is_empty() {
        return pattern.iter().all(|&b| b == b'*');
    }
    match pattern[0] {
        b'*' => {
            // Skip consecutive stars
            let mut pi = 0;
            while pi < pattern.len() && pattern[pi] == b'*' {
                pi += 1;
            }
            let rest = &pattern[pi..];
            if rest.is_empty() {
                return true;
            }
            for i in 0..=name.len() {
                if fnmatch_recursive(&name[i..], rest) {
                    return true;
                }
            }
            false
        }
        b'?' => fnmatch_recursive(&name[1..], &pattern[1..]),
        b'[' => {
            let c = name[0];
            let mut i = 1;
            let negate = i < pattern.len() && pattern[i] == b'!';
            if negate {
                i += 1;
            }
            let mut matched = false;
            while i < pattern.len() && pattern[i] != b']' {
                if i + 2 < pattern.len() && pattern[i + 1] == b'-' && pattern[i + 2] != b']' {
                    if c >= pattern[i] && c <= pattern[i + 2] {
                        matched = true;
                    }
                    i += 3;
                } else {
                    if c == pattern[i] {
                        matched = true;
                    }
                    i += 1;
                }
            }
            // skip closing ']'
            if i < pattern.len() {
                i += 1;
            }
            if matched != negate {
                fnmatch_recursive(&name[1..], &pattern[i..])
            } else {
                false
            }
        }
        c => {
            if c == name[0] {
                fnmatch_recursive(&name[1..], &pattern[1..])
            } else {
                false
            }
        }
    }
}

// =========================================================================
// Shared trait
// =========================================================================

/// Common behaviour shared by [`File`] and [`Directory`].
pub trait PathLike {
    fn raw_path(&self) -> &PathBuf;

    fn display_path(&self) -> String {
        format_path(self.raw_path())
    }

    fn exists(&self) -> bool {
        self.raw_path().exists()
    }

    fn as_posix(&self) -> String {
        self.raw_path().to_string_lossy().replace('\\', "/")
    }

    fn name(&self) -> String {
        self.raw_path()
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    fn suffix(&self) -> String {
        self.raw_path()
            .extension()
            .map(|e| format!(".{}", e.to_string_lossy()))
            .unwrap_or_default()
    }

    fn stem(&self) -> String {
        self.raw_path()
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default()
    }

    fn parts(&self) -> Vec<String> {
        self.raw_path()
            .components()
            .map(|c| c.as_os_str().to_string_lossy().to_string())
            .collect()
    }

    fn parent(&self) -> Directory {
        Directory {
            path: self
                .raw_path()
                .parent()
                .map(|p| p.to_path_buf())
                .unwrap_or_else(|| self.raw_path().clone()),
        }
    }

    fn modified(&self) -> io::Result<SystemTime> {
        fs::metadata(self.raw_path())?.modified()
    }

    fn resolve(&self) -> Entry {
        let resolved = fs::canonicalize(self.raw_path()).unwrap_or_else(|_| {
            if self.raw_path().is_absolute() {
                self.raw_path().clone()
            } else {
                std::env::current_dir()
                    .map(|cwd| cwd.join(self.raw_path()))
                    .unwrap_or_else(|_| self.raw_path().clone())
            }
        });
        if resolved.is_dir() {
            Entry::Directory(Directory { path: resolved })
        } else {
            Entry::File(File { path: resolved })
        }
    }

    fn match_glob(&self, pattern: &str, name_only: bool, invert: bool) -> bool {
        let target = if name_only {
            self.name()
        } else {
            self.display_path()
        };
        fnmatch(&target, pattern) ^ invert
    }

    fn match_re(&self, re: &str, name_only: bool, invert: bool) -> bool {
        let target = if name_only {
            self.name()
        } else {
            self.display_path()
        };
        // Python re.match anchors at the start of the string
        let anchored = if re.starts_with('^') {
            re.to_string()
        } else {
            format!("^(?:{})", re)
        };
        let matched = Regex::new(&anchored)
            .map(|regex| regex.is_match(&target))
            .unwrap_or(false);
        matched ^ invert
    }

    fn link_to(&self, target_path: &StdPath, soft: bool) -> Result<(), String> {
        if target_path.symlink_metadata().is_ok() {
            if target_path
                .symlink_metadata()
                .map(|m| m.file_type().is_symlink())
                .unwrap_or(false)
            {
                fs::remove_file(target_path).map_err(|e| e.to_string())?;
            } else {
                return Err(format!("Cannot link to {}!", format_path(target_path)));
            }
        }
        if soft {
            #[cfg(unix)]
            std::os::unix::fs::symlink(self.raw_path(), target_path).map_err(|e| e.to_string())?;
            #[cfg(not(unix))]
            return Err("Soft links not supported on this platform".to_string());
        } else {
            fs::hard_link(self.raw_path(), target_path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn unlink(&self) -> io::Result<()> {
        if let Ok(meta) = fs::symlink_metadata(self.raw_path()) {
            if meta.file_type().is_symlink() {
                return fs::remove_file(self.raw_path());
            }
        }
        Ok(())
    }

    fn join(&self, other: &str) -> Entry {
        let new_path = self.raw_path().join(other);
        let resolved = fs::canonicalize(&new_path).unwrap_or_else(|_| normalize_path(&new_path));
        if resolved.is_dir() {
            Entry::Directory(Directory { path: resolved })
        } else {
            Entry::File(File { path: resolved })
        }
    }
}

// =========================================================================
// File
// =========================================================================

#[derive(Clone, Debug)]
pub struct File {
    pub path: PathBuf,
}

impl PathLike for File {
    fn raw_path(&self) -> &PathBuf {
        &self.path
    }
}

impl File {
    pub fn new(path_str: &str) -> Self {
        File {
            path: parse_path(path_str),
        }
    }

    pub fn repr(&self) -> String {
        format!("File(path={})", self.display_path())
    }

    pub fn size(&self, block_size: u64) -> io::Result<u64> {
        let actual = fs::metadata(&self.path)?.len();
        Ok(std::cmp::max(actual, block_size))
    }

    pub fn rm(&self) -> io::Result<()> {
        fs::remove_file(&self.path)
    }
}

impl fmt::Display for File {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.display_path())
    }
}

impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
impl Eq for File {}

impl Hash for File {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.display_path().hash(state);
    }
}

impl PartialOrd for File {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for File {
    fn cmp(&self, other: &Self) -> Ordering {
        self.display_path().cmp(&other.display_path())
    }
}

// =========================================================================
// Directory
// =========================================================================

#[derive(Clone, Debug)]
pub struct Directory {
    pub path: PathBuf,
}

impl PathLike for Directory {
    fn raw_path(&self) -> &PathBuf {
        &self.path
    }
}

impl Directory {
    pub fn new(path_str: &str) -> Self {
        Directory {
            path: parse_path(path_str),
        }
    }

    pub fn repr(&self) -> String {
        format!("Directory(path={})", self.display_path())
    }

    pub fn ls(&self) -> Vec<Entry> {
        let mut entries = Vec::new();
        if let Ok(read_dir) = fs::read_dir(&self.path) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                // Use entry metadata (follows symlinks).  For broken
                // symlinks metadata() fails → treat as file.
                let is_dir = entry.metadata().map(|m| m.is_dir()).unwrap_or(false);
                if is_dir {
                    entries.push(Entry::Directory(Directory { path }));
                } else {
                    entries.push(Entry::File(File { path }));
                }
            }
        }
        entries.sort();
        entries
    }

    pub fn list(&self) -> io::Result<Vec<String>> {
        Ok(fs::read_dir(&self.path)?
            .flatten()
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect())
    }

    pub fn len(&self) -> usize {
        self.list().map(|l| l.len()).unwrap_or(0)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn recurse(&self) -> Vec<Entry> {
        let mut result = Vec::new();
        self.recurse_impl(&mut result);
        result
    }

    fn recurse_impl(&self, result: &mut Vec<Entry>) {
        for entry in self.ls() {
            match &entry {
                Entry::Directory(d) => d.recurse_impl(result),
                Entry::File(_) => result.push(entry),
            }
        }
    }

    pub fn size(&self, block_size: u64) -> u64 {
        let mut total = 0;
        for entry in self.ls() {
            match &entry {
                Entry::File(f) => {
                    total += f.size(block_size).unwrap_or(0);
                }
                Entry::Directory(d) => {
                    total += d.size(block_size);
                }
            }
        }
        total
    }

    pub fn all_match(&self, pattern: &str, name_only: bool, invert: bool) -> Vec<Entry> {
        self.ls()
            .into_iter()
            .filter(|e| e.match_glob(pattern, name_only, invert))
            .collect()
    }

    pub fn all_rematch(&self, re: &str, name_only: bool, invert: bool) -> Vec<Entry> {
        self.ls()
            .into_iter()
            .filter(|e| e.match_re(re, name_only, invert))
            .collect()
    }

    pub fn rm(&self) -> io::Result<()> {
        fs::remove_dir_all(&self.path)
    }
}

impl fmt::Display for Directory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.display_path())
    }
}

impl PartialEq for Directory {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
impl Eq for Directory {}

impl Hash for Directory {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.display_path().hash(state);
    }
}

impl PartialOrd for Directory {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Directory {
    fn cmp(&self, other: &Self) -> Ordering {
        self.display_path().cmp(&other.display_path())
    }
}

// =========================================================================
// Entry (union of File | Directory)
// =========================================================================

#[derive(Clone, Debug)]
pub enum Entry {
    File(File),
    Directory(Directory),
}

impl Entry {
    pub fn path(&self) -> &PathBuf {
        match self {
            Entry::File(f) => &f.path,
            Entry::Directory(d) => &d.path,
        }
    }

    pub fn is_file(&self) -> bool {
        matches!(self, Entry::File(_))
    }

    pub fn is_directory(&self) -> bool {
        matches!(self, Entry::Directory(_))
    }

    pub fn display_path(&self) -> String {
        match self {
            Entry::File(f) => f.display_path(),
            Entry::Directory(d) => d.display_path(),
        }
    }

    pub fn name(&self) -> String {
        match self {
            Entry::File(f) => f.name(),
            Entry::Directory(d) => d.name(),
        }
    }

    pub fn as_posix(&self) -> String {
        match self {
            Entry::File(f) => f.as_posix(),
            Entry::Directory(d) => d.as_posix(),
        }
    }

    pub fn match_glob(&self, pattern: &str, name_only: bool, invert: bool) -> bool {
        match self {
            Entry::File(f) => f.match_glob(pattern, name_only, invert),
            Entry::Directory(d) => d.match_glob(pattern, name_only, invert),
        }
    }

    pub fn match_re(&self, re: &str, name_only: bool, invert: bool) -> bool {
        match self {
            Entry::File(f) => f.match_re(re, name_only, invert),
            Entry::Directory(d) => d.match_re(re, name_only, invert),
        }
    }

    pub fn resolve(&self) -> Entry {
        match self {
            Entry::File(f) => f.resolve(),
            Entry::Directory(d) => d.resolve(),
        }
    }

    pub fn join(&self, other: &str) -> Entry {
        match self {
            Entry::File(f) => f.join(other),
            Entry::Directory(d) => d.join(other),
        }
    }

    pub fn rm(&self) -> io::Result<()> {
        match self {
            Entry::File(f) => f.rm(),
            Entry::Directory(d) => d.rm(),
        }
    }

    pub fn size(&self, block_size: u64) -> u64 {
        match self {
            Entry::File(f) => f.size(block_size).unwrap_or(0),
            Entry::Directory(d) => d.size(block_size),
        }
    }
}

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Entry::File(file) => write!(f, "{}", file),
            Entry::Directory(dir) => write!(f, "{}", dir),
        }
    }
}

impl PartialEq for Entry {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Entry::File(a), Entry::File(b)) => a == b,
            (Entry::Directory(a), Entry::Directory(b)) => a == b,
            _ => false,
        }
    }
}
impl Eq for Entry {}

impl Hash for Entry {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.display_path().hash(state);
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.display_path().cmp(&other.display_path())
    }
}

// =========================================================================
// OrganizeIt
// =========================================================================

pub struct OrganizeIt;

impl OrganizeIt {
    pub fn new() -> Self {
        OrganizeIt
    }

    pub fn expand(&self, directory: &str) -> Directory {
        Directory::new(directory)
    }
}

impl Default for OrganizeIt {
    fn default() -> Self {
        Self::new()
    }
}

// =========================================================================
// Convenience: resolve a path string to an Entry
// =========================================================================

pub fn resolve_path(path_str: &str) -> Entry {
    let file = File::new(path_str);
    file.resolve()
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use tempfile::TempDir;

    // ---- helpers ----

    fn create_test_directory(base: &StdPath) -> PathBuf {
        let dir = base.join("directory");
        let files = [
            "file1",
            "file1.md",
            "file1.png",
            "file1.txt",
            "file2",
            "file2.md",
            "file2.png",
            "file2.txt",
        ];

        // subdir1 and subdir2 have subsubdir1 / subsubdir2 with files
        for i in 1..=2 {
            let subdir = dir.join(format!("subdir{}", i));
            fs::create_dir_all(&subdir).unwrap();
            for f in &files {
                fs::File::create(subdir.join(f)).unwrap();
            }
            for j in 1..=2 {
                let subsubdir = subdir.join(format!("subsubdir{}", j));
                fs::create_dir_all(&subsubdir).unwrap();
                for f in &files {
                    fs::File::create(subsubdir.join(f)).unwrap();
                }
            }
        }

        // subdir3 and subdir4 only have files
        for i in 3..=4 {
            let subdir = dir.join(format!("subdir{}", i));
            fs::create_dir_all(&subdir).unwrap();
            for f in &files {
                fs::File::create(subdir.join(f)).unwrap();
            }
        }

        dir
    }

    // ---- fnmatch ----

    #[test]
    fn test_fnmatch_basic() {
        assert!(fnmatch("directory", "directory*"));
        assert!(fnmatch("directory", "director?"));
        assert!(!fnmatch("directory", "dir"));
        assert!(fnmatch("subdir1", "subdir*"));
        assert!(fnmatch("subdir1", "subdir[0-9]"));
        assert!(!fnmatch("subdirA", "subdir[0-9]"));
        assert!(fnmatch("file1.txt", "*.txt"));
        assert!(fnmatch("anything", "*"));
    }

    #[test]
    fn test_fnmatch_multi_star() {
        assert!(fnmatch(
            "file://organizeit2/tests/directory",
            "*organizeit2*directory"
        ));
    }

    // ---- parse / format ----

    #[test]
    fn test_parse_path() {
        assert_eq!(parse_path("file:///tmp/test"), PathBuf::from("/tmp/test"));
        assert_eq!(parse_path("local:///tmp/test"), PathBuf::from("/tmp/test"));
        assert_eq!(parse_path("/tmp/test"), PathBuf::from("/tmp/test"));
        assert_eq!(
            parse_path("file://relative/path"),
            PathBuf::from("relative/path")
        );
    }

    #[test]
    fn test_format_path() {
        assert_eq!(format_path(StdPath::new("/tmp/test")), "file:///tmp/test");
    }

    // ---- File ----

    #[test]
    fn test_file_new() {
        let f = File::new("file:///tmp/test.txt");
        assert_eq!(f.path, PathBuf::from("/tmp/test.txt"));
    }

    #[test]
    fn test_file_properties() {
        let f = File::new("file:///some/path/test.txt");
        assert_eq!(f.name(), "test.txt");
        assert_eq!(f.suffix(), ".txt");
        assert_eq!(f.stem(), "test");
        assert_eq!(f.display_path(), "file:///some/path/test.txt");
        assert_eq!(f.as_posix(), "/some/path/test.txt");
        assert_eq!(f.repr(), "File(path=file:///some/path/test.txt)");
    }

    #[test]
    fn test_file_size() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("empty_file");
        fs::File::create(&path).unwrap();
        let f = File { path };
        // empty file → block_size wins
        assert_eq!(f.size(4096).unwrap(), 4096);
    }

    #[test]
    fn test_file_display_and_ordering() {
        let a = File::new("file:///a");
        let b = File::new("file:///b");
        assert!(a < b);
        assert_eq!(format!("{}", a), "file:///a");
    }

    #[test]
    fn test_file_hash_and_eq() {
        let a = File::new("file:///tmp/x");
        let b = File::new("file:///tmp/x");
        assert_eq!(a, b);
        let mut set = HashSet::new();
        set.insert(a);
        assert!(set.contains(&b));
    }

    #[test]
    fn test_file_rm() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("to_remove");
        fs::File::create(&path).unwrap();
        assert!(path.exists());
        File { path: path.clone() }.rm().unwrap();
        assert!(!path.exists());
    }

    // ---- Directory ----

    #[test]
    fn test_directory_new() {
        let d = Directory::new("local:///tmp/test");
        assert_eq!(d.path, PathBuf::from("/tmp/test"));
        assert_eq!(d.display_path(), "file:///tmp/test");
    }

    #[test]
    fn test_directory_repr() {
        let d = Directory::new("file:///tmp/test");
        assert_eq!(d.repr(), "Directory(path=file:///tmp/test)");
    }

    #[test]
    fn test_directory_ls() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory { path: dir.clone() };

        let entries: Vec<String> = d.ls().iter().map(|e| e.name()).collect();
        assert_eq!(entries, vec!["subdir1", "subdir2", "subdir3", "subdir4"]);
    }

    #[test]
    fn test_directory_ls_str() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory { path: dir.clone() };
        let root = d.display_path();

        let entries: Vec<String> = d.ls().iter().map(|e| e.to_string()).collect();
        assert_eq!(
            entries,
            vec![
                format!("{}/subdir1", root),
                format!("{}/subdir2", root),
                format!("{}/subdir3", root),
                format!("{}/subdir4", root),
            ]
        );
    }

    #[test]
    fn test_directory_recurse() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory { path: dir };
        assert_eq!(d.recurse().len(), 64);
    }

    #[test]
    fn test_directory_len() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory { path: dir };
        assert_eq!(d.len(), 4);
    }

    // ---- hashable entries ----

    #[test]
    fn test_path_hashable() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory { path: dir };
        let set: HashSet<String> = d.recurse().iter().map(|e| e.to_string()).collect();
        assert_eq!(set.len(), 64);
    }

    // ---- size ----

    #[test]
    fn test_directory_size() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory { path: dir };
        assert_eq!(d.size(4096), 64 * 4096); // 262144
    }

    // ---- resolve ----

    #[test]
    fn test_file_resolve_to_directory() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        // Create a File pointing at a directory → resolve should give Directory
        let f = File { path: dir.clone() };
        assert!(f.resolve().is_directory());
    }

    #[test]
    fn test_directory_resolve_to_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("some_file");
        fs::File::create(&path).unwrap();
        // Create a Directory pointing at a file → resolve should give File
        let d = Directory { path };
        assert!(d.resolve().is_file());
    }

    // ---- match_glob ----

    #[test]
    fn test_match_glob_name_only() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory { path: dir };
        assert!(d.match_glob("directory*", true, false));
        assert!(!d.match_glob("directory*", true, true)); // invert
        assert!(!d.match_glob("directory", false, false)); // full path won't match just name
    }

    #[test]
    fn test_match_glob_full_path() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory { path: dir };
        assert!(d.match_glob("*directory", false, false));
        assert!(!d.match_glob("*directory", false, true)); // invert
    }

    // ---- all_match ----

    #[test]
    fn test_all_match() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory { path: dir };
        assert_eq!(d.all_match("subdir*", true, false).len(), 4);
        assert_eq!(d.all_match("dir*", true, false).len(), 0);
    }

    // ---- rematch ----

    #[test]
    fn test_match_re_name_only() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory { path: dir };
        assert!(d.match_re("directory", true, false));
        assert!(!d.match_re("directory", true, true)); // invert
        assert!(!d.match_re("directory", false, false)); // anchored, full path starts with file://
    }

    #[test]
    fn test_match_re_full_path() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory { path: dir };
        assert!(d.match_re("file://[a-zA-Z0-9/_.-]*", false, false));
    }

    // ---- all_rematch ----

    #[test]
    fn test_all_rematch() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory { path: dir };
        assert_eq!(d.all_rematch("subdir[0-9]+", true, false).len(), 4);
        assert_eq!(d.all_rematch("subdir[0-3]+", true, false).len(), 3);
    }

    // ---- link / unlink ----

    #[test]
    #[cfg(unix)]
    fn test_link_and_unlink() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory { path: dir };
        let link_path = tmp.path().join("directory_link");
        d.link_to(&link_path, true).unwrap();
        assert!(link_path
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink());
        let link_dir = Directory { path: link_path };
        link_dir.unlink().unwrap();
        assert!(!link_dir.path.exists());
    }

    #[test]
    #[cfg(unix)]
    fn test_cant_link_existing() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory { path: dir.clone() };
        // Linking to an existing non-symlink should fail
        let result = d.link_to(&dir, true);
        assert!(result.is_err());
    }

    // ---- convenience (join / parent) ----

    #[test]
    fn test_join_parent() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let subdir1 = dir.join("subdir1");
        let file_path = subdir1.join("file1.txt");
        let f = File {
            path: file_path.clone(),
        };

        // f / ".." should resolve to subdir1
        let parent_entry = f.join("..");
        assert!(parent_entry.is_directory());
        assert_eq!(parent_entry.path(), &fs::canonicalize(&subdir1).unwrap());

        // parent should also give subdir1
        assert_eq!(f.parent().path, subdir1);

        // f / ".." / ".." should resolve to directory
        let grandparent = f.join("..").join("..");
        assert_eq!(grandparent.path(), &fs::canonicalize(&dir).unwrap());

        // f / ".." / ".." / "subdir1" should equal subdir1
        let back_to_subdir1 = f.join("..").join("..").join("subdir1");
        assert_eq!(back_to_subdir1.path(), &fs::canonicalize(&subdir1).unwrap());
    }

    #[test]
    fn test_str_method() {
        let d = Directory::new("file:///tmp/test");
        assert_eq!(d.to_string(), "file:///tmp/test");
        assert_eq!(d.display_path(), "file:///tmp/test");
    }

    // ---- bad symlink ----

    #[test]
    #[cfg(unix)]
    fn test_bad_symlink() {
        let tmp = TempDir::new().unwrap();
        let bad_link = tmp.path().join("bad_symlink");
        std::os::unix::fs::symlink("/tmp/whatever/non_existent_file", &bad_link).unwrap();

        let d = Directory {
            path: tmp.path().to_path_buf(),
        };
        assert_eq!(d.ls().len(), 1);
        assert_eq!(d.size(4096), 0);
    }

    // ---- OrganizeIt ----

    #[test]
    fn test_organizeit_expand() {
        let oi = OrganizeIt::new();
        let d = oi.expand("local:///tmp/test");
        assert_eq!(d.path, PathBuf::from("/tmp/test"));
        assert_eq!(d.display_path(), "file:///tmp/test");
    }

    // ---- Entry equality ----

    #[test]
    fn test_entry_equality() {
        let a = Entry::File(File::new("file:///tmp/x"));
        let b = Entry::File(File::new("file:///tmp/x"));
        assert_eq!(a, b);

        let c = Entry::Directory(Directory::new("file:///tmp/x"));
        assert_ne!(a, c); // different variants
    }

    // ---- resolve_path ----

    #[test]
    fn test_resolve_path() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let entry = resolve_path(&format!("file://{}", dir.display()));
        assert!(entry.is_directory());

        let file_path = dir.join("subdir1").join("file1.txt");
        let entry = resolve_path(&format!("file://{}", file_path.display()));
        assert!(entry.is_file());
    }

    // ---- Directory rm ----

    #[test]
    fn test_directory_rm() {
        let tmp = TempDir::new().unwrap();
        let rm_dir = tmp.path().join("to_remove");
        fs::create_dir(&rm_dir).unwrap();
        fs::File::create(rm_dir.join("file.txt")).unwrap();
        assert!(rm_dir.exists());
        Directory {
            path: rm_dir.clone(),
        }
        .rm()
        .unwrap();
        assert!(!rm_dir.exists());
    }

    // ---- Entry size ----

    #[test]
    fn test_entry_size() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("sized_file");
        fs::write(&path, "hello").unwrap();
        let e = Entry::File(File { path });
        // "hello" is 5 bytes, block_size 4096 → max(5, 4096) = 4096
        assert_eq!(e.size(4096), 4096);
        // with block_size 0 → actual size
        assert_eq!(e.size(0), 5);
    }

    // ---- modified ----

    #[test]
    fn test_modified() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("mod_file");
        fs::File::create(&path).unwrap();
        let f = File { path };
        let mtime = f.modified().unwrap();
        assert!(mtime.elapsed().unwrap().as_secs() < 10);
    }

    // ---- exists ----

    #[test]
    fn test_exists() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("exists_file");
        let f = File { path: path.clone() };
        assert!(!f.exists());
        fs::File::create(&path).unwrap();
        assert!(f.exists());
    }
}
