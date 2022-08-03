use eyre::Result;
// use log_watcher::LogWatcherEvent;
use std::io::prelude::*;
use std::path::Path;
use tauri::async_runtime::block_on;

fn main() -> Result<()> {
    // let mut watcher = notify::recommended_watcher(on_event)?;

    // watcher.watch(Path::new(r"."), RecursiveMode::NonRecursive)?;

    block_on(task());

    println!("Press any key to exit...");
    std::io::stdin().lock().read_exact(&mut [0u8])?;

    Ok(())
}

async fn task() -> Result<()> {
    use log_reader::LogReader;
    let mut log_reader = LogReader::new(Path::new("./test.txt")).await?;

    loop {
        let lines = log_reader.read_new_lines().await?;
        if lines.is_empty() {
            println!("No new lines");
        } else {
            println!("Got new lines:");
            for line in lines {
                println!(" - '{}'", line);
            }
        }

        println!("\nPress any key to read again...");
        std::io::stdin().lock().read_exact(&mut [0u8])?;
    }

    Ok(())
}

async fn watch_logs() -> Result<()> {
    // let mut log_watcher = log_watcher::LogWatcher::new(Path::new("./test.txt"))?;

    // let mut rx = log_watcher.watch()?;

    // while let Some(event) = rx.recv().await {
    //     match event {
    //         LogWatcherEvent::Line(line) => println!("New line: {line}"),
    //         // Err(e) => println!("Log watcher error: {e}"),
    //     }
    // }

    Ok(())
}

// fn on_event(res: notify::Result<notify::Event>) {
//     match res {
//         Ok(event) => println!("event: {event:?}"),
//         Err(err) => println!("error: {err:?}"),
//     }
// }

mod file_watcher {
    use std::path::{Path, PathBuf};

    use eyre::Result;
    use notify::{recommended_watcher, EventHandler, RecommendedWatcher, Watcher};
    use tauri::async_runtime::{self, block_on, spawn, Receiver};

    pub struct FileWatcher {
        file_path: PathBuf,
        watcher: Option<RecommendedWatcher>,
    }

    #[derive(Debug, Clone)]
    pub enum FileWatcherEvent {
        Created,
        Deleted,
        Modified,
    }

    impl FileWatcher {
        pub fn new<P: AsRef<Path>>(file_path: P) -> Result<Self> {
            Ok(Self {
                file_path: file_path.as_ref().to_path_buf().canonicalize()?,
                watcher: None,
            })
        }

        pub fn watch(&mut self) -> Result<Receiver<FileWatcherEvent>> {
            let (tx, rx) = async_runtime::channel(1);
            let file_path = self.file_path.to_owned();

            let watcher = self.watcher.insert(recommended_watcher(
                move |event: notify::Result<notify::Event>| {
                    if let Ok(event) = event {
                        if event.paths.contains(&file_path) {
                            // if let Some(log_watcher_event) = Self::handle_log_file_event(event) {
                            // let log_watcher_event = log_watcher_event.clone();
                            // let tx = tx.clone();
                            // spawn(async move {
                            //     tx.send(log_watcher_event).await.unwrap();
                            // });
                            // }
                        };
                    }
                },
            )?);

            watcher.watch(&self.file_path, notify::RecursiveMode::NonRecursive)?;

            Ok(rx)
        }
    }
}

mod log_reader {

    struct LogReader {}

    impl LogReader{}

}

mod log_reader1 {
    use std::io::SeekFrom;
    use std::path::{Path, PathBuf};

    use eyre::Result;
    use notify::{recommended_watcher, RecommendedWatcher, Watcher};
    use tauri::async_runtime::{self, block_on, spawn, Receiver};
    use tokio::fs::File;
    use tokio::io::{AsyncReadExt, AsyncSeekExt};

    pub struct LogReader {
        path: PathBuf,
        file: File,
        offset: u64,
        buffer: String,
    }

    impl LogReader {
        pub async fn new(path: impl AsRef<Path>) -> Result<Self> {
            let path = path.as_ref().to_owned();
            let mut file = File::open(&path).await?;

            // Seek to end of file and store that as the current offset
            let offset = file.seek(SeekFrom::End(0)).await?;
            println!("Seeking to {offset}");

            Ok(Self {
                path,
                file,
                offset,
                buffer: String::new(),
            })
        }

        pub async fn read_new_lines<'a>(&'a mut self) -> Result<Vec<&'a str>> {
            // Read all of file from current offset
            self.file.sync_all();
            self.file.read_to_string(&mut self.buffer).await?;
            println!("Offset is now {}", self.file.stream_position().await?);
            // self.file.
            // let last_newline = self.buffer.rfind("\n").unwrap_or_default();
            // let delta_offset = (self.buffer.len() - last_newline).try_into()?;
            // self.file.seek(SeekFrom::Current(delta_offset)).await?;

            Ok(self.buffer.lines().collect())
        }
    }
}

// mod log_watcher1 {
//     use eyre::{eyre::eyre, Result};
//     use notify::{recommended_watcher, Event, RecommendedWatcher, RecursiveMode, Watcher};
//     use std::{
//         path::{Path, PathBuf},
//         sync::mpsc::{self, Receiver}, thread,
//     };
//     use tauri::async_runtime::block_on;

//     pub struct LogWatcher {
//         log_path: PathBuf,
//         parent_path: PathBuf,
//         watcher: Option<RecommendedWatcher>,
//     }

//     pub enum LogWatcherEvent {
//         Line(String),
//     }

//     impl LogWatcher {
//         pub fn new(log_path: &Path) -> Result<Self> {
//             let log_path: PathBuf = log_path.to_path_buf();
//             let parent_path = log_path
//                 .parent()
//                 .ok_or(eyre!("no parent for log_path"))?
//                 .to_path_buf();
//             Ok(Self {
//                 log_path,
//                 parent_path,
//                 watcher: None,
//             })
//         }

//         pub fn watch(&mut self) -> Result<Receiver<LogWatcherEvent>> {
//             let (watcher_tx, watcher_rx) = mpsc::channel();
//             let (tx, rx) = mpsc::channel();

//             let mut watcher = self.watcher.insert(RecommendedWatcher::new(watcher_tx)?);

//             thread::spawn(f)

//             watcher.watch(&self.parent_path, RecursiveMode::NonRecursive)?;
//             Ok(rx)
//         }

//         fn on_event(self, event: notify::Result<Event>) {
//             todo!()
//         }
//     }
// }
