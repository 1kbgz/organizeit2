use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path as StdPath, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::SystemTime;

use fsspec_rs::{FileSystem, FileType, LocalFs, S3Config, S3Fs};
use regex::Regex;

// =========================================================================
// Utilities
// =========================================================================

/// Extract the protocol scheme from a path string.
/// Returns `("file", rest)` for local paths, `("s3", rest)` for `s3://`, etc.
/// Protocols like `file-rs://`, `s3-rs://` are returned as-is and handled by the
/// Python fsspec adapter in the binding layer.
pub fn extract_protocol(path: &str) -> (&str, &str) {
    if let Some(rest) = path.strip_prefix("file://") {
        ("file", rest)
    } else if let Some(rest) = path.strip_prefix("local://") {
        ("file", rest)
    } else if let Some(idx) = path.find("://") {
        (&path[..idx], &path[idx + 3..])
    } else {
        ("file", path)
    }
}

/// Check whether a filesystem is local (file/local protocol).
pub fn is_local_fs(fs: &Arc<dyn FileSystem + Send + Sync>) -> bool {
    fs.protocol().iter().any(|p| *p == "file" || *p == "local")
}

/// Strip `file://` or `local://` protocol prefixes from a path string (local paths only).
pub fn parse_path(s: &str) -> PathBuf {
    if let Some(rest) = s.strip_prefix("file://") {
        PathBuf::from(rest)
    } else if let Some(rest) = s.strip_prefix("local://") {
        PathBuf::from(rest)
    } else {
        PathBuf::from(s)
    }
}

/// Format a path with a `file://` protocol prefix (local paths only).
pub fn format_path(p: &StdPath) -> String {
    format!("file://{}", p.to_string_lossy().replace('\\', "/"))
}

/// On Windows, `std::fs::canonicalize` returns verbatim extended-length paths prefixed with
/// `\\?\`. This helper strips that prefix so paths compare and display as conventional Windows
/// paths (e.g. `C:\foo` instead of `\\?\C:\foo`).
fn strip_verbatim_prefix(path: PathBuf) -> PathBuf {
    #[cfg(windows)]
    {
        let s = path.to_string_lossy();
        if let Some(stripped) = s.strip_prefix(r"\\?\") {
            return PathBuf::from(stripped.to_string());
        }
    }
    path
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

/// Detect protocol from a path string and return the appropriate filesystem.
/// Handles local (`file://`, `local://`, bare paths) and S3 (`s3://`) natively.
/// Returns `Err` for unknown protocols; callers (e.g. PyO3 bindings) can fall back to
/// a Python fsspec backend.
///
/// Filesystem instances are cached globally so that many `File`/`Directory` objects
/// sharing the same protocol and credentials reuse a single connection pool rather than
/// each opening their own, which would exhaust OS file descriptors.
static LOCAL_FS_INSTANCE: OnceLock<Arc<dyn FileSystem + Send + Sync>> = OnceLock::new();
static S3_FS_CACHE: OnceLock<Mutex<HashMap<String, Arc<dyn FileSystem + Send + Sync>>>> =
    OnceLock::new();

fn s3_fs_cache() -> &'static Mutex<HashMap<String, Arc<dyn FileSystem + Send + Sync>>> {
    S3_FS_CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn fs_for_path(path: &str) -> Result<Arc<dyn FileSystem + Send + Sync>, String> {
    let (protocol, rest) = extract_protocol(path);
    match protocol {
        "file" | "local" => Ok(Arc::clone(
            LOCAL_FS_INSTANCE.get_or_init(|| Arc::new(LocalFs::new())),
        )),
        "s3" => {
            // Extract bucket name from the first path component
            let trimmed = rest.trim_start_matches('/');
            let bucket = trimmed.split('/').next().unwrap_or(trimmed);
            if bucket.is_empty() {
                return Err("S3 path must include a bucket name".to_string());
            }
            // Helper: read env var, treating empty as unset
            let env_non_empty =
                |key: &str| -> Option<String> { std::env::var(key).ok().filter(|v| !v.is_empty()) };
            let endpoint_url = env_non_empty("FSSPEC_S3_ENDPOINT_URL")
                .or_else(|| env_non_empty("AWS_ENDPOINT_URL"));
            let access_key_id =
                env_non_empty("AWS_ACCESS_KEY_ID").or_else(|| env_non_empty("FSSPEC_S3_KEY"));
            let secret_access_key = env_non_empty("AWS_SECRET_ACCESS_KEY")
                .or_else(|| env_non_empty("FSSPEC_S3_SECRET"));
            let session_token = env_non_empty("AWS_SESSION_TOKEN");
            let region =
                env_non_empty("AWS_REGION").or_else(|| env_non_empty("AWS_DEFAULT_REGION"));
            let anon = access_key_id.is_none() && secret_access_key.is_none();

            // Cache key uniquely identifies a (bucket × credentials × endpoint) combination.
            let cache_key = format!(
                "s3:{}:{}:{}:{}:{}:{}",
                bucket,
                endpoint_url.as_deref().unwrap_or(""),
                access_key_id.as_deref().unwrap_or(""),
                session_token.as_deref().unwrap_or(""),
                region.as_deref().unwrap_or(""),
                anon,
            );

            {
                let cache = s3_fs_cache().lock().unwrap();
                if let Some(fs) = cache.get(&cache_key) {
                    return Ok(Arc::clone(fs));
                }
            }

            let cfg = S3Config {
                bucket: bucket.to_string(),
                endpoint_url,
                access_key_id,
                secret_access_key,
                session_token,
                region,
                anon,
                virtual_hosted_style_request: false,
            };
            let fs = S3Fs::new(cfg)
                .map(|fs| Arc::new(fs) as Arc<dyn FileSystem + Send + Sync>)
                .map_err(|e| format!("Failed to create S3 filesystem: {e}"))?;
            s3_fs_cache()
                .lock()
                .unwrap()
                .insert(cache_key, Arc::clone(&fs));
            Ok(fs)
        }
        _ => Err(format!("Unsupported protocol: {protocol}://")),
    }
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
    fn fs(&self) -> &Arc<dyn FileSystem + Send + Sync>;

    fn display_path(&self) -> String {
        // Normalize to forward slashes so paths display consistently across platforms
        // (on Windows, PathBuf uses `\` internally).
        let path_str = self.raw_path().to_string_lossy().replace('\\', "/");
        self.fs().unstrip_protocol(&path_str)
    }

    fn exists(&self) -> bool {
        let path_str = self.raw_path().to_string_lossy().to_string();
        self.fs().exists(&path_str).unwrap_or(false)
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
            filesystem: Arc::clone(self.fs()),
        }
    }

    fn modified(&self) -> io::Result<SystemTime> {
        let path_str = self.raw_path().to_string_lossy().to_string();
        let info = self
            .fs()
            .info(&path_str)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))?;
        info.modified
            .ok_or_else(|| io::Error::new(io::ErrorKind::Other, "modified time not available"))
    }

    fn resolve(&self) -> Entry {
        let fs = Arc::clone(self.fs());
        let is_local = is_local_fs(&fs);

        let resolved = if is_local {
            // Try to canonicalize for local paths; strip Windows verbatim prefix `\\?\`.
            std::fs::canonicalize(self.raw_path())
                .map(strip_verbatim_prefix)
                .unwrap_or_else(|_| {
                    if self.raw_path().is_absolute() {
                        self.raw_path().clone()
                    } else {
                        std::env::current_dir()
                            .map(|cwd| cwd.join(self.raw_path()))
                            .unwrap_or_else(|_| self.raw_path().clone())
                    }
                })
        } else {
            // For remote filesystems, just normalize (no local canonicalize)
            normalize_path(self.raw_path())
        };

        let resolved_str = resolved.to_string_lossy().to_string();
        let is_dir = fs.isdir(&resolved_str).unwrap_or(false);

        if is_dir {
            Entry::Directory(Directory {
                path: resolved,
                filesystem: fs,
            })
        } else {
            Entry::File(File {
                path: resolved,
                filesystem: fs,
            })
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
        if !is_local_fs(self.fs()) {
            return Err("Linking is not supported on remote filesystems".to_string());
        }
        if target_path.symlink_metadata().is_ok() {
            if target_path
                .symlink_metadata()
                .map(|m| m.file_type().is_symlink())
                .unwrap_or(false)
            {
                std::fs::remove_file(target_path).map_err(|e| e.to_string())?;
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
            std::fs::hard_link(self.raw_path(), target_path).map_err(|e| e.to_string())?;
        }
        Ok(())
    }

    fn unlink(&self) -> io::Result<()> {
        if !is_local_fs(self.fs()) {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "Unlinking is not supported on remote filesystems",
            ));
        }
        if let Ok(meta) = std::fs::symlink_metadata(self.raw_path()) {
            if meta.file_type().is_symlink() {
                return std::fs::remove_file(self.raw_path());
            }
        }
        Ok(())
    }

    fn join(&self, other: &str) -> Entry {
        let new_path = self.raw_path().join(other);
        let fs = Arc::clone(self.fs());
        let is_local = is_local_fs(&fs);
        let resolved = if is_local {
            std::fs::canonicalize(&new_path)
                .map(strip_verbatim_prefix)
                .unwrap_or_else(|_| normalize_path(&new_path))
        } else {
            normalize_path(&new_path)
        };
        let path_str = resolved.to_string_lossy().to_string();
        let is_dir = fs.isdir(&path_str).unwrap_or(false);
        if is_dir {
            Entry::Directory(Directory {
                path: resolved,
                filesystem: fs,
            })
        } else {
            Entry::File(File {
                path: resolved,
                filesystem: fs,
            })
        }
    }
}

// =========================================================================
// File
// =========================================================================

#[derive(Clone)]
pub struct File {
    pub path: PathBuf,
    pub filesystem: Arc<dyn FileSystem + Send + Sync>,
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("File").field("path", &self.path).finish()
    }
}

impl PathLike for File {
    fn raw_path(&self) -> &PathBuf {
        &self.path
    }
    fn fs(&self) -> &Arc<dyn FileSystem + Send + Sync> {
        &self.filesystem
    }
}

impl File {
    pub fn new(path_str: &str) -> Result<Self, String> {
        let filesystem = fs_for_path(path_str)?;
        let stripped = filesystem.strip_protocol(path_str);
        let path = PathBuf::from(stripped);
        Ok(Self { path, filesystem })
    }

    pub fn with_fs(path_str: &str, filesystem: Arc<dyn FileSystem + Send + Sync>) -> Self {
        let stripped = filesystem.strip_protocol(path_str);
        let path = PathBuf::from(stripped);
        Self { path, filesystem }
    }

    pub fn repr(&self) -> String {
        format!("File(path={})", self.display_path())
    }

    pub fn size(&self, block_size: u64) -> Option<u64> {
        let path_str = self.path.to_string_lossy().to_string();
        let file_size = self.filesystem.size(&path_str).ok()?;
        Some(file_size.max(block_size))
    }

    pub fn rm(&self) -> io::Result<()> {
        let path_str = self.path.to_string_lossy().to_string();
        self.filesystem
            .rm_file(&path_str)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
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

#[derive(Clone)]
pub struct Directory {
    pub path: PathBuf,
    pub filesystem: Arc<dyn FileSystem + Send + Sync>,
}

impl fmt::Debug for Directory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Directory")
            .field("path", &self.path)
            .finish()
    }
}

impl PathLike for Directory {
    fn raw_path(&self) -> &PathBuf {
        &self.path
    }
    fn fs(&self) -> &Arc<dyn FileSystem + Send + Sync> {
        &self.filesystem
    }
}

impl Directory {
    pub fn new(path_str: &str) -> Result<Self, String> {
        let filesystem = fs_for_path(path_str)?;
        let stripped = filesystem.strip_protocol(path_str);
        let path = PathBuf::from(stripped);
        Ok(Self { path, filesystem })
    }

    pub fn with_fs(path_str: &str, filesystem: Arc<dyn FileSystem + Send + Sync>) -> Self {
        let stripped = filesystem.strip_protocol(path_str);
        let path = PathBuf::from(stripped);
        Self { path, filesystem }
    }

    pub fn repr(&self) -> String {
        format!("Directory(path={})", self.display_path())
    }

    pub fn ls(&self) -> Vec<Entry> {
        let path_str = self.path.to_string_lossy().to_string();
        let entries = match self.filesystem.ls(&path_str, true) {
            Ok(entries) => entries,
            Err(_) => return vec![],
        };

        let mut result: Vec<Entry> = entries
            .into_iter()
            .map(|info| {
                // Normalize returned names through strip_protocol so the
                // stored path is in the same convention the fs expects.
                let stripped = self.filesystem.strip_protocol(&info.name);
                let entry_path = PathBuf::from(&stripped);
                match info.file_type {
                    FileType::Directory => Entry::Directory(Directory {
                        path: entry_path,
                        filesystem: Arc::clone(&self.filesystem),
                    }),
                    _ => Entry::File(File {
                        path: entry_path,
                        filesystem: Arc::clone(&self.filesystem),
                    }),
                }
            })
            .collect();
        result.sort();
        result
    }

    pub fn list(&self) -> Vec<String> {
        self.ls().iter().map(|e| e.name()).collect()
    }

    pub fn len(&self) -> usize {
        self.ls().len()
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
        let path_str = self.path.to_string_lossy().to_string();
        self.filesystem
            .rm(&path_str, true)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
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

#[derive(Clone)]
pub enum Entry {
    File(File),
    Directory(Directory),
}

impl fmt::Debug for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Entry::File(file) => f.debug_tuple("File").field(file).finish(),
            Entry::Directory(dir) => f.debug_tuple("Directory").field(dir).finish(),
        }
    }
}

impl Entry {
    pub fn path(&self) -> &PathBuf {
        match self {
            Entry::File(f) => &f.path,
            Entry::Directory(d) => &d.path,
        }
    }

    pub fn fs(&self) -> &Arc<dyn FileSystem + Send + Sync> {
        match self {
            Entry::File(f) => &f.filesystem,
            Entry::Directory(d) => &d.filesystem,
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

pub struct OrganizeIt {
    pub filesystem: Arc<dyn FileSystem + Send + Sync>,
}

impl OrganizeIt {
    pub fn new() -> Self {
        OrganizeIt {
            filesystem: Arc::new(LocalFs::new()),
        }
    }

    pub fn with_fs(filesystem: Arc<dyn FileSystem + Send + Sync>) -> Self {
        OrganizeIt { filesystem }
    }

    pub fn expand(&self, directory: &str) -> Directory {
        Directory::with_fs(directory, Arc::clone(&self.filesystem))
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

pub fn resolve_path(path_str: &str) -> Result<Entry, String> {
    let fs = fs_for_path(path_str)?;
    let stripped = fs.strip_protocol(path_str);
    let file = File {
        path: PathBuf::from(stripped),
        filesystem: fs,
    };
    Ok(file.resolve())
}

// =========================================================================
// Tests
// =========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::fs;
    use tempfile::TempDir;

    fn local_fs() -> Arc<dyn FileSystem + Send + Sync> {
        Arc::new(LocalFs::new())
    }

    // helpers

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

    // fnmatch

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

    // parse / format

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

    // File

    #[test]
    fn test_file_new() {
        let f = File::new("file:///tmp/test.txt").unwrap();
        assert_eq!(f.path, PathBuf::from("/tmp/test.txt"));
    }

    #[test]
    fn test_file_properties() {
        let f = File::new("file:///some/path/test.txt").unwrap();
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
        let f = File {
            path,
            filesystem: local_fs(),
        };
        // empty file → block_size wins
        assert_eq!(f.size(4096).unwrap(), 4096);
    }

    #[test]
    fn test_file_display_and_ordering() {
        let a = File::new("file:///a").unwrap();
        let b = File::new("file:///b").unwrap();
        assert!(a < b);
        assert_eq!(format!("{}", a), "file:///a");
    }

    #[test]
    fn test_file_hash_and_eq() {
        let a = File::new("file:///tmp/x").unwrap();
        let b = File::new("file:///tmp/x").unwrap();
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
        File {
            path: path.clone(),
            filesystem: local_fs(),
        }
        .rm()
        .unwrap();
        assert!(!path.exists());
    }

    // Directory

    #[test]
    fn test_directory_new() {
        let d = Directory::new("local:///tmp/test").unwrap();
        assert_eq!(d.path, PathBuf::from("/tmp/test"));
        assert_eq!(d.display_path(), "file:///tmp/test");
    }

    #[test]
    fn test_directory_repr() {
        let d = Directory::new("file:///tmp/test").unwrap();
        assert_eq!(d.repr(), "Directory(path=file:///tmp/test)");
    }

    #[test]
    fn test_directory_ls() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory {
            path: dir.clone(),
            filesystem: local_fs(),
        };

        let entries: Vec<String> = d.ls().iter().map(|e| e.name()).collect();
        assert_eq!(entries, vec!["subdir1", "subdir2", "subdir3", "subdir4"]);
    }

    #[test]
    fn test_directory_ls_str() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory {
            path: dir.clone(),
            filesystem: local_fs(),
        };
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
        let d = Directory {
            path: dir,
            filesystem: local_fs(),
        };
        assert_eq!(d.recurse().len(), 64);
    }

    #[test]
    fn test_directory_len() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory {
            path: dir,
            filesystem: local_fs(),
        };
        assert_eq!(d.len(), 4);
    }

    // hashable entries

    #[test]
    fn test_path_hashable() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory {
            path: dir,
            filesystem: local_fs(),
        };
        let set: HashSet<String> = d.recurse().iter().map(|e| e.to_string()).collect();
        assert_eq!(set.len(), 64);
    }

    // size

    #[test]
    fn test_directory_size() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory {
            path: dir,
            filesystem: local_fs(),
        };
        assert_eq!(d.size(4096), 64 * 4096); // 262144
    }

    // resolve

    #[test]
    fn test_file_resolve_to_directory() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        // Create a File pointing at a directory → resolve should give Directory
        let f = File {
            path: dir.clone(),
            filesystem: local_fs(),
        };
        assert!(f.resolve().is_directory());
    }

    #[test]
    fn test_directory_resolve_to_file() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("some_file");
        fs::File::create(&path).unwrap();
        // Create a Directory pointing at a file → resolve should give File
        let d = Directory {
            path,
            filesystem: local_fs(),
        };
        assert!(d.resolve().is_file());
    }

    // match_glob

    #[test]
    fn test_match_glob_name_only() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory {
            path: dir,
            filesystem: local_fs(),
        };
        assert!(d.match_glob("directory*", true, false));
        assert!(!d.match_glob("directory*", true, true)); // invert
        assert!(!d.match_glob("directory", false, false)); // full path won't match just name
    }

    #[test]
    fn test_match_glob_full_path() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory {
            path: dir,
            filesystem: local_fs(),
        };
        assert!(d.match_glob("*directory", false, false));
        assert!(!d.match_glob("*directory", false, true)); // invert
    }

    // all_match

    #[test]
    fn test_all_match() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory {
            path: dir,
            filesystem: local_fs(),
        };
        assert_eq!(d.all_match("subdir*", true, false).len(), 4);
        assert_eq!(d.all_match("dir*", true, false).len(), 0);
    }

    // rematch

    #[test]
    fn test_match_re_name_only() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory {
            path: dir,
            filesystem: local_fs(),
        };
        assert!(d.match_re("directory", true, false));
        assert!(!d.match_re("directory", true, true)); // invert
        assert!(!d.match_re("directory", false, false)); // anchored, full path starts with file://
    }

    #[test]
    fn test_match_re_full_path() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory {
            path: dir,
            filesystem: local_fs(),
        };
        assert!(d.match_re("file://[a-zA-Z0-9/_.-]*", false, false));
    }

    // all_rematch

    #[test]
    fn test_all_rematch() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory {
            path: dir,
            filesystem: local_fs(),
        };
        assert_eq!(d.all_rematch("subdir[0-9]+", true, false).len(), 4);
        assert_eq!(d.all_rematch("subdir[0-3]+", true, false).len(), 3);
    }

    // link / unlink

    #[test]
    #[cfg(unix)]
    fn test_link_and_unlink() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory {
            path: dir,
            filesystem: local_fs(),
        };
        let link_path = tmp.path().join("directory_link");
        d.link_to(&link_path, true).unwrap();
        assert!(link_path
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink());
        let link_dir = Directory {
            path: link_path,
            filesystem: local_fs(),
        };
        link_dir.unlink().unwrap();
        assert!(!link_dir.path.exists());
    }

    #[test]
    #[cfg(unix)]
    fn test_cant_link_existing() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let d = Directory {
            path: dir.clone(),
            filesystem: local_fs(),
        };
        // Linking to an existing non-symlink should fail
        let result = d.link_to(&dir, true);
        assert!(result.is_err());
    }

    // convenience (join / parent)

    #[test]
    fn test_join_parent() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let subdir1 = dir.join("subdir1");
        let file_path = subdir1.join("file1.txt");
        let f = File {
            path: file_path.clone(),
            filesystem: local_fs(),
        };

        // f / ".." should resolve to subdir1
        let parent_entry = f.join("..");
        assert!(parent_entry.is_directory());
        assert_eq!(
            parent_entry.path(),
            &strip_verbatim_prefix(fs::canonicalize(&subdir1).unwrap())
        );

        // parent should also give subdir1
        assert_eq!(f.parent().path, subdir1);

        // f / ".." / ".." should resolve to directory
        let grandparent = f.join("..").join("..");
        assert_eq!(
            grandparent.path(),
            &strip_verbatim_prefix(fs::canonicalize(&dir).unwrap())
        );

        // f / ".." / ".." / "subdir1" should equal subdir1
        let back_to_subdir1 = f.join("..").join("..").join("subdir1");
        assert_eq!(
            back_to_subdir1.path(),
            &strip_verbatim_prefix(fs::canonicalize(&subdir1).unwrap())
        );
    }

    #[test]
    fn test_str_method() {
        let d = Directory::new("file:///tmp/test").unwrap();
        assert_eq!(d.to_string(), "file:///tmp/test");
        assert_eq!(d.display_path(), "file:///tmp/test");
    }

    // bad symlink

    #[test]
    #[cfg(unix)]
    fn test_bad_symlink() {
        let tmp = TempDir::new().unwrap();
        let bad_link = tmp.path().join("bad_symlink");
        std::os::unix::fs::symlink("/tmp/whatever/non_existent_file", &bad_link).unwrap();

        let d = Directory {
            path: tmp.path().to_path_buf(),
            filesystem: local_fs(),
        };
        assert_eq!(d.ls().len(), 1);
        assert_eq!(d.size(4096), 0);
    }

    // OrganizeIt

    #[test]
    fn test_organizeit_expand() {
        let oi = OrganizeIt::new();
        let d = oi.expand("local:///tmp/test");
        assert_eq!(d.path, PathBuf::from("/tmp/test"));
        assert_eq!(d.display_path(), "file:///tmp/test");
    }

    // Entry equality

    #[test]
    fn test_entry_equality() {
        let a = Entry::File(File::new("file:///tmp/x").unwrap());
        let b = Entry::File(File::new("file:///tmp/x").unwrap());
        assert_eq!(a, b);

        let c = Entry::Directory(Directory::new("file:///tmp/x").unwrap());
        assert_ne!(a, c); // different variants
    }

    // resolve_path

    #[test]
    fn test_resolve_path() {
        let tmp = TempDir::new().unwrap();
        let dir = create_test_directory(tmp.path());
        let entry = resolve_path(&format!("file://{}", dir.display())).unwrap();
        assert!(entry.is_directory());

        let file_path = dir.join("subdir1").join("file1.txt");
        let entry = resolve_path(&format!("file://{}", file_path.display())).unwrap();
        assert!(entry.is_file());
    }

    // Directory rm

    #[test]
    fn test_directory_rm() {
        let tmp = TempDir::new().unwrap();
        let rm_dir = tmp.path().join("to_remove");
        fs::create_dir(&rm_dir).unwrap();
        fs::File::create(rm_dir.join("file.txt")).unwrap();
        assert!(rm_dir.exists());
        Directory {
            path: rm_dir.clone(),
            filesystem: local_fs(),
        }
        .rm()
        .unwrap();
        assert!(!rm_dir.exists());
    }

    // Entry size

    #[test]
    fn test_entry_size() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("sized_file");
        fs::write(&path, "hello").unwrap();
        let e = Entry::File(File {
            path,
            filesystem: local_fs(),
        });
        // "hello" is 5 bytes, block_size 4096 → max(5, 4096) = 4096
        assert_eq!(e.size(4096), 4096);
        // with block_size 0 → actual size
        assert_eq!(e.size(0), 5);
    }

    // modified

    #[test]
    fn test_modified() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("mod_file");
        fs::File::create(&path).unwrap();
        let f = File {
            path,
            filesystem: local_fs(),
        };
        let mtime = f.modified().unwrap();
        assert!(mtime.elapsed().unwrap().as_secs() < 10);
    }

    // exists

    #[test]
    fn test_exists() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("exists_file");
        let f = File {
            path: path.clone(),
            filesystem: local_fs(),
        };
        assert!(!f.exists());
        fs::File::create(&path).unwrap();
        assert!(f.exists());
    }
}
