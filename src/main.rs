mod folder;

use std::{io, path::Path, sync::{atomic::AtomicUsize, Arc}};

use folder::Folder;

fn main() -> Result<(), io::Error>{

    println!("Enter the path");
    let mut path = String::new();
    io::stdin().read_line(&mut path).expect("error reading input");
    let root = Folder::from(
        Path::new(path.trim()).canonicalize().unwrap().as_path(),
        Arc::new(AtomicUsize::new(4))
    )?;

    root.summarize(Some(1));
    let mut previous_folder = root.clone();

    loop {
        println!();
        let mut new_path = String::new();
        io::stdin().read_line(&mut new_path).expect("error reading input");
        let (new_path, depth) = match new_path.split_once(',') {
            Some((p, d)) => {
                (p.trim(), d.trim().parse().ok())
            },
            None => (new_path.trim(), None)
        };
        match previous_folder.navigate(new_path) {
            Ok(new_folder) => {
                new_folder.summarize(depth);
                previous_folder = new_folder;
            },
            Err(err) => {
                eprintln!("error: {}", err);
                previous_folder.summarize(depth);
            }
        }
    }
}
