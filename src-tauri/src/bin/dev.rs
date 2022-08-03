use std::path::Path;

use eyre::Result;
use file_watcher::{FileWatcher, FileWatcherEvent};
use tail::Tail;
use tauri::async_runtime::block_on;
use tokio_stream::{Stream, StreamExt};

fn main() -> Result<()> {
    block_on(task())
}

async fn task() -> Result<()> {
    let file_path = Path::new("./test.txt");
    let watcher = file_watcher::FileWatcher::new(file_path)?;
    let t = tail::from_file_watcher(watcher)?;

    t.tail().await?;

    // let s = watcher.watch()?;

    // let t = Tail::new(file_path, s);

    // tokio::pin!(s);

    // while let Some(event) = s.next().await {
    //     println!("Got event: {event:?}");
    // }

    Ok(())
}

// Parse client.txt log events
mod log_parser {}

// Stream events when a file is changed
mod file_watcher {
    use std::path::{Path, PathBuf};

    use eyre::{eyre, Result};
    use notify::{recommended_watcher, Config, EventHandler, RecommendedWatcher, Watcher};
    use tokio::sync::mpsc::{self, Receiver, Sender};
    use tokio_stream::{wrappers::ReceiverStream, Stream};

    pub struct FileWatcher {
        file_path: PathBuf,
        watcher: RecommendedWatcher,
        rx: Option<Receiver<FileWatcherEvent>>,
    }

    impl FileWatcher {
        pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self> {
            let file_path = file_path.as_ref().to_path_buf().canonicalize()?;
            let (tx, rx) = mpsc::channel(1);
            let event_handler = FileWatcherEventHandler::new(&file_path, tx);

            Ok(Self {
                file_path,
                watcher: recommended_watcher(event_handler)?,
                rx: Some(rx),
            })
        }

        pub fn file_path(&self) -> &Path {
            &self.file_path
        }

        pub fn watch(mut self) -> Result<impl Stream<Item = FileWatcherEvent>> {
            self.watcher.configure(Config::PreciseEvents(true))?;
            self.watcher
                .watch(&self.file_path, notify::RecursiveMode::NonRecursive)?;

            self.rx
                .take()
                .ok_or(eyre!("Stream already used."))
                .map(ReceiverStream::new)
        }
    }

    #[derive(Debug, Clone)]
    pub enum FileWatcherEvent {
        Created,
        Removed,
        Modified,
    }

    impl FileWatcherEvent {
        pub fn from_event(event: &notify::Event) -> Option<Self> {
            if event.kind.is_create() {
                Some(FileWatcherEvent::Created)
            } else if event.kind.is_remove() {
                Some(FileWatcherEvent::Removed)
            } else if event.kind.is_modify() {
                Some(FileWatcherEvent::Modified)
            } else {
                None
            }
        }
    }

    struct FileWatcherEventHandler {
        file_path: PathBuf,
        tx: Sender<FileWatcherEvent>,
    }

    impl FileWatcherEventHandler {
        pub fn new(file_path: impl AsRef<Path>, tx: Sender<FileWatcherEvent>) -> Self {
            Self {
                file_path: file_path.as_ref().to_path_buf(),
                tx,
            }
        }
    }

    impl EventHandler for FileWatcherEventHandler {
        fn handle_event(&mut self, event: notify::Result<notify::Event>) {
            if let Ok(event) = event {
                if event.paths.contains(&self.file_path) {
                    if let Some(file_watcher_event) = FileWatcherEvent::from_event(&event) {
                        self.tx.blocking_send(file_watcher_event).unwrap();
                    }
                }
            }
        }
    }
}

// Given a path to a file, watch the file and emit bytes of the appended content
mod tail {
    use eyre::Result;
    use std::{
        io::SeekFrom,
        path::{Path, PathBuf},
    };
    use tauri::api::file;

    use tokio::{
        fs::{self, File},
        io::AsyncRead,
    };
    use tokio_stream::{Stream, StreamExt};

    use crate::file_watcher::{self, FileWatcher, FileWatcherEvent};

    pub struct Tail<S: Stream<Item = FileWatcherEvent>> {
        file_path: PathBuf,
        events: S,
        file: Option<File>,
        offset: Option<u64>,
    }

    pub fn from_file_watcher(
        watcher: FileWatcher,
    ) -> Result<Tail<impl Stream<Item = FileWatcherEvent>>> {
        let file_path = watcher.file_path().to_path_buf();
        let s = watcher.watch()?;

        Ok(Tail::new(file_path, s))
    }

    impl<S: Stream<Item = FileWatcherEvent> + Unpin> Tail<S> {
        pub fn new(file_path: impl AsRef<Path>, events: S) -> Self {
            Self {
                file_path: file_path.as_ref().to_path_buf(),
                events,
                file: None,
                offset: None,
            }
        }

        pub async fn tail(mut self) -> Result<impl Stream<Item = ()>> {
            let metadata = fs::metadata(&self.file_path).await?;
            if metadata.is_file() {
                self.file = Some(File::open(&self.file_path).await?);
                self.offset = Some(metadata.len());
            };

            Ok(self.events.then(|e| self.handle_event(e)))

            // while let Some(event) = self.events.next().await {
            //     match event {
            //         FileWatcherEvent::Created => self.handle_created(),
            //         FileWatcherEvent::Removed => self.handle_removed(),
            //         FileWatcherEvent::Modified => self.handle_modified(),
            //     }
            // }

            // Ok(())
        }

        async fn handle_event(&mut self, event: FileWatcherEvent) -> () {
            match event {
                FileWatcherEvent::Created => self.handle_created().await,
                FileWatcherEvent::Removed => self.handle_removed().await,
                FileWatcherEvent::Modified => self.handle_modified().await,
            }
        }

        async fn handle_created(&mut self) -> () {
            println!("Created: {:?}", &self.file_path)
        }

        async fn handle_removed(&mut self) -> () {
            println!("Removed: {:?}", &self.file_path)
        }

        async fn handle_modified(&mut self) -> () {
            println!("Modified: {:?}", &self.file_path)
        }
    }
}

// The following two might be implementable using tokio codec

// Given a stream of bytes, emit a stream of lines, buffering up to one line
mod tail_lines_stream {}

// Given a stream of lines, emit a stream of parsed log events
mod log_stream {}
