//! File watcher. Polls `packages/engine/src/**.ts` (and any handler-source
//! globs the workspace declares) and surfaces a `WatchEvent` per change so
//! the dev-server's reload loop can recompile + re-register the affected
//! handler.
//!
//! ## Phase-2a scope
//!
//! Phase-2a ships a *poll-based* watcher rather than depending on
//! `notify` / inotify / FSEvents. Rationale: the build matrix already
//! gates on no-new-deps (`cargo deny`), and the watcher is invoked from
//! a developer-only binary path; a 250ms polling cadence is acceptable
//! for the inner-loop. Phase-2b can swap in `notify` if profiling
//! identifies the poll as the wall-clock bottleneck.
//!
//! The poll snapshot is `(path, modified-time, length)`; comparing
//! length alongside mtime catches the "edit-and-save-back-in-same-second"
//! case that plain mtime polling misses on coarse filesystems (HFS+,
//! some networked FS).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// File event surfaced to the reload loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchEvent {
    /// A handler source file was created.
    Created(PathBuf),
    /// A handler source file was modified.
    Modified(PathBuf),
    /// A handler source file was deleted.
    Removed(PathBuf),
}

/// Per-file fingerprint used to detect changes between polls.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Fingerprint {
    mtime: SystemTime,
    len: u64,
}

/// Polling watcher.
#[derive(Debug)]
pub struct Watcher {
    /// Root of the directory tree to watch.
    root: PathBuf,
    /// File extension to consider (e.g., `"ts"`).
    extension: String,
    /// Last-seen fingerprints, keyed by canonicalized path.
    last: BTreeMap<PathBuf, Fingerprint>,
}

impl Watcher {
    /// Construct a watcher over `root`, considering only files whose
    /// extension matches `extension` (no leading dot).
    #[must_use]
    pub fn new(root: &Path, extension: &str) -> Self {
        Self {
            root: root.to_path_buf(),
            extension: extension.to_string(),
            last: BTreeMap::new(),
        }
    }

    /// Walk the watched root and return any `WatchEvent`s observed since
    /// the previous call. The first call after construction reports every
    /// existing file as `Created`.
    pub fn poll(&mut self) -> Vec<WatchEvent> {
        let mut current: BTreeMap<PathBuf, Fingerprint> = BTreeMap::new();
        collect_files(&self.root, &self.extension, &mut current);

        let mut events = Vec::new();

        // Detect Created / Modified.
        for (path, fp) in &current {
            match self.last.get(path) {
                Some(prev) if prev == fp => {}
                Some(_) => events.push(WatchEvent::Modified(path.clone())),
                None => events.push(WatchEvent::Created(path.clone())),
            }
        }

        // Detect Removed.
        for path in self.last.keys() {
            if !current.contains_key(path) {
                events.push(WatchEvent::Removed(path.clone()));
            }
        }

        self.last = current;
        events
    }

    /// Root of the watched tree.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }
}

fn collect_files(dir: &Path, extension: &str, out: &mut BTreeMap<PathBuf, Fingerprint>) {
    let entries = match fs::read_dir(dir) {
        Ok(it) => it,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(meta) = entry.metadata() else { continue };
        if meta.is_dir() {
            collect_files(&path, extension, out);
        } else if meta.is_file()
            && path
                .extension()
                .and_then(|e| e.to_str())
                .is_some_and(|e| e.eq_ignore_ascii_case(extension))
        {
            let mtime = meta.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let fp = Fingerprint {
                mtime,
                len: meta.len(),
            };
            out.insert(path, fp);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn watcher_detects_created_modified_removed() {
        let dir = tempdir().unwrap();
        let mut w = Watcher::new(dir.path(), "ts");
        assert!(w.poll().is_empty(), "empty dir polls clean");

        let f = dir.path().join("h1.ts");
        fs::write(&f, b"v1").unwrap();
        let evs = w.poll();
        assert!(matches!(evs.as_slice(), [WatchEvent::Created(_)]));

        // Modify with different length so the fingerprint changes
        // regardless of mtime granularity on the host filesystem.
        let mut handle = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&f)
            .unwrap();
        handle.write_all(b"v_modified_with_more_bytes").unwrap();
        drop(handle);
        let evs = w.poll();
        assert!(matches!(evs.as_slice(), [WatchEvent::Modified(_)]));

        fs::remove_file(&f).unwrap();
        let evs = w.poll();
        assert!(matches!(evs.as_slice(), [WatchEvent::Removed(_)]));
    }

    #[test]
    fn watcher_filters_by_extension() {
        let dir = tempdir().unwrap();
        let mut w = Watcher::new(dir.path(), "ts");
        fs::write(dir.path().join("a.ts"), b"x").unwrap();
        fs::write(dir.path().join("b.md"), b"x").unwrap();
        let evs = w.poll();
        assert_eq!(evs.len(), 1);
        match &evs[0] {
            WatchEvent::Created(p) => assert!(p.ends_with("a.ts")),
            _ => panic!("expected Created"),
        }
    }
}
