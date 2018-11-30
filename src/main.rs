use std::fs;
use std::io;
use std::fmt::Debug;
use std::convert::From;
use std::path::Display;
use std::cmp::Ord;
use std::sync::mpsc;
use std::sync::mpsc::{Sender, Receiver};
use std::thread;

fn main() {
    let dir = std::env::args().nth(1);
    let mut files = analyzse_dir(&dir.unwrap()).unwrap();
    files.sort_by(|a, b| b.size.cmp(&a.size));
    for file in files {
        println!("{}", file);
    }
}

fn analyzse_dir(dir: &str) -> io::Result<Vec<File>> {
    let (tx, rx) = mpsc::channel();

    let mut data: Vec<File> = Vec::new();
    let files = fs::read_dir(dir)?;
    for file in files {
        let file = file?;
        let metadata = file.metadata()?;
        if metadata.is_dir() {
            thread::spawn(|| {
                let result1 = analyzse_dir(&format!("{}", &file.path().display())).unwrap();
                tx.send(Ok(result1));
            });
        } else {
            let name = String::from(file.path().file_name().unwrap().to_str().unwrap());
            let f = File { name, size: metadata.len(), is_dir: metadata.is_dir() };
            data.push(f);
        }
    }
    let result = rx.recv().unwrap();
    data.append(&mut result.unwrap());
    return Ok(data);
}

#[derive(Debug, Eq)]
struct File {
    name: String,
    size: u64,
    is_dir: bool,
}

unsafe impl Send for File {}

unsafe impl Sync for File {}

impl std::fmt::Display for File {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (size, units) = if self.size > 1000000000 {
            (self.size / 1000000000, "Gb")
        } else if self.size > 1000000 {
            (self.size / 100000, "Mb")
        } else if self.size > 1000 {
            (self.size / 100, "Kb")
        } else {
            (self.size, "b")
        };
        write!(f, "{}: {} {}", self.name, size, units)
    }
}

impl PartialOrd for File {
    fn partial_cmp(&self, other: &File) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for File {
    fn eq(&self, other: &File) -> bool {
        self.name == other.name
    }
}

impl Ord for File {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.size.cmp(&other.size)
    }
}