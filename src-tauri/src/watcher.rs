use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub struct ShortcutWatcher {
    _watcher: RecommendedWatcher,
    _worker: thread::JoinHandle<()>,
}

impl ShortcutWatcher {
    pub fn new(
        dir: PathBuf,
        on_change: impl Fn(PathBuf) + Send + Sync + 'static,
    ) -> notify::Result<Self> {
        let (tx, rx) = mpsc::channel();
        let callback = Arc::new(on_change);
        let watch_dir = dir.clone();

        let mut watcher = RecommendedWatcher::new(
            move |result| {
                let _ = tx.send(result);
            },
            Config::default(),
        )?;

        watcher.watch(&dir, RecursiveMode::Recursive)?;

        let worker = thread::spawn(move || {
            loop {
                let event = match rx.recv() {
                    Ok(event) => event,
                    Err(_) => break,
                };

                if event.is_err() {
                    continue;
                }

                // Collapse bursts of writes into a single rescan.
                thread::sleep(Duration::from_millis(300));
                while rx.recv_timeout(Duration::from_millis(50)).is_ok() {}

                callback(watch_dir.clone());
            }
        });

        Ok(Self {
            _watcher: watcher,
            _worker: worker,
        })
    }
}
