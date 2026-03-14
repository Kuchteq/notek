use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::Sender;

use inotify::{EventMask, Inotify, WatchDescriptor, WatchMask};

use crate::app::AppEvent;

const WATCH_MASK: WatchMask =
    WatchMask::from_bits_truncate(WatchMask::MOVE.bits() | WatchMask::CREATE.bits());

/// Recursively add inotify watches for `dir` and all its subdirectories.
/// Populates wd_to_dir: WatchDescriptor -> directory path relative to base_dir.
fn watch_recursive(
    inotify: &mut Inotify,
    dir: &Path,
    base_dir: &Path,
    wd_to_dir: &mut HashMap<WatchDescriptor, PathBuf>,
) {
    let wd = inotify
        .watches()
        .add(dir, WATCH_MASK)
        .unwrap_or_else(|_| panic!("Failed to add inotify watch for {:?}", dir));

    let rel = dir
        .strip_prefix(base_dir)
        .unwrap_or(Path::new(""))
        .to_path_buf();
    wd_to_dir.insert(wd, rel);

    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                watch_recursive(inotify, &path, base_dir, wd_to_dir);
            }
        }
    }
}

pub fn monitor_updates(tx: Sender<AppEvent>, base_dir: &Path) {
    let mut inotify = Inotify::init().expect("Failed to initialize inotify");

    // Map from watch descriptor to directory path (relative to base_dir)
    let mut wd_to_dir: HashMap<WatchDescriptor, PathBuf> = HashMap::new();

    watch_recursive(&mut inotify, base_dir, base_dir, &mut wd_to_dir);

    println!("Watching {:?} (and subdirs) for activity...", base_dir);

    let mut pending_moves: HashMap<u32, PathBuf> = HashMap::new();
    let mut buffer = [0u8; 4096];

    loop {
        let events = inotify
            .read_events_blocking(&mut buffer)
            .expect("Failed to read inotify events");

        for event in events {
            let file_name = match event.name {
                Some(n) => PathBuf::from(n),
                None => continue,
            };

            // Look up which directory this watch descriptor corresponds to
            let parent_rel = match wd_to_dir.get(&event.wd) {
                Some(d) => d.clone(),
                None => continue,
            };

            // Build the full relative path (e.g. "school/math/note.md")
            let rel_path = if parent_rel.as_os_str().is_empty() {
                file_name
            } else {
                parent_rel.join(file_name)
            };

            // Handle new subdirectory: add a recursive watch for it
            let is_dir = event.mask.contains(EventMask::ISDIR);
            if is_dir
                && (event.mask.contains(EventMask::CREATE)
                    || event.mask.contains(EventMask::MOVED_TO))
            {
                let abs_path = base_dir.join(&rel_path);
                watch_recursive(&mut inotify, &abs_path, base_dir, &mut wd_to_dir);
                continue;
            }

            // Only care about .md files (skip .md.structure and others)
            let is_md = rel_path.extension().and_then(|s| s.to_str()) == Some("md");
            if !is_md {
                continue;
            }

            if event.mask.contains(EventMask::CREATE) {
                println!("New file detected: {:?}", rel_path);
                let _ = tx.send(AppEvent::FileCreated(rel_path));
            } else if event.mask.contains(EventMask::MOVED_FROM) {
                pending_moves.insert(event.cookie, rel_path);
            } else if event.mask.contains(EventMask::MOVED_TO) {
                if let Some(src) = pending_moves.remove(&event.cookie) {
                    println!("Renamed: {:?} -> {:?}", src, rel_path);
                    let _ = tx.send(AppEvent::FileRenamed {
                        from: src,
                        to: rel_path,
                    });
                } else {
                    // Moved in from outside the watched directory — treat as new file
                    println!("File moved into watched dir: {:?}", rel_path);
                    let _ = tx.send(AppEvent::FileCreated(rel_path));
                }
            }
        }
    }
}
