use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::mpsc::channel;
use std::time::Duration;
use notify::{DebouncedEvent, Watcher, RecursiveMode, watcher};

struct Cli {
    repository: String,
    path: PathBuf,
    watch: bool,
}

fn main() {
    let repository = std::env::args().nth(1).expect("no repository given");
    let path = std::env::args().nth(2).expect("no path (file or folder) given");
    let watch: bool = match std::env::args().nth(3) {
        Some(v) => {
            if v == "--watch" {
                true
            } else {
                panic!("unexpected argument {}", v)
            }
        },
        None => false
    };

    let args = Cli {
        repository: repository,
        path: PathBuf::from(path),
        watch: watch,
    };

    if !Path::new("./sync-repository").exists() {
        Command::new("git")
                .arg("clone")
                .arg(&args.repository)
                .arg("sync-repository")
                .status()
                .unwrap();
    }

    if args.watch {
        let (tx, rx) = channel();
        let mut watcher = watcher(tx, Duration::from_secs(10)).unwrap();
        watcher.watch(args.path, RecursiveMode::Recursive).unwrap();
        loop {
            match rx.recv() {
                Ok(event) => {
                    match event {
                        DebouncedEvent::Write(path) => sync(&path),
                        _ => (),
                    }
                },
                Err(e) => println!("watch error: {:?}", e),
            }
        }
    } else {
        sync(&args.path)
    }
}

fn sync(path: &PathBuf) {
    let file_name = path.file_name().unwrap().to_str().unwrap();
    let path_to = format!("./sync-repository/{}", file_name);
    fs::copy(&path, path_to).unwrap();
    Command::new("git")
            .arg("-C")
            .arg("./sync-repository")
            .arg("add")
            .arg(".")
            .status()
            .unwrap();

    Command::new("git")
            .arg("-C")
            .arg("./sync-repository")
            .arg("commit")
            .arg("-m")
            .arg(format!("Sync {}", file_name))
            .status()
            .unwrap();

    Command::new("git")
            .arg("-C")
            .arg("./sync-repository")
            .arg("push")
            .arg("origin")
            .arg("master")
            .status()
            .unwrap();
}
