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

    root.summarize();
    let mut previous_folder = root.clone();

    loop {
        println!();
        let mut new_path = String::new();
        io::stdin().read_line(&mut new_path).expect("error reading input");
        match previous_folder.navigate(new_path.trim()) {
            Ok(new_folder) => {
                new_folder.summarize();
                previous_folder = new_folder;
            },
            Err(err) => {
                eprintln!("error: {}", err);
                previous_folder.summarize();
            }
        }
    }
}
