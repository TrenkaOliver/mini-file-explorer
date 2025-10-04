use std::{ffi::OsStr, fs, io, path::{Path, PathBuf}, sync::{atomic::{AtomicUsize, Ordering}, Arc, Mutex, Weak}, thread::{self, JoinHandle}};

#[derive(Debug)]
pub struct Folder {
    name: String,
    parent: Mutex<Weak<Folder>>,
    children: Mutex<Vec<Arc<Folder>>>,
}

impl Folder {
    pub fn new(name: &str) -> Folder {
        Folder { 
            name: String::from(name), 
            parent: Mutex::new(Weak::new()), 
            children: Mutex::new(vec![])
        }
    }


    pub fn from(path: &Path, thread_count: Arc<AtomicUsize>) -> Result<Arc<Folder>, io::Error> {
        let mut handles = vec![];
        let folder = Arc::new(Folder::new(path.file_name().unwrap_or(&OsStr::new("/")).to_str().unwrap()));
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?.path();
                Folder::process_entry(
                    Arc::clone(&folder), entry, Arc::clone(&thread_count), &mut handles);
            }
        }
        for handle in handles {
            if let Err(err) = handle.join() {
                eprintln!("thread paniced: {:?}", err);
            }
        }
        Ok(folder)
    }

    fn process_entry(parent_folder: Arc<Folder>, path: PathBuf, thread_count: Arc<AtomicUsize>, handles: &mut Vec<JoinHandle<()>>) {
        loop {
            let current = thread_count.load(Ordering::SeqCst);
            if current == 0 {break;}

            match thread_count.compare_exchange(
                current, current - 1, 
                Ordering::SeqCst, Ordering::SeqCst
            ) {
                Ok(_) => {
                    let tc = Arc::clone(&thread_count);
                    let pf = Arc::clone(&parent_folder);
                    let handle = thread::spawn(move || {
                        if let Ok(sub_folder) = Folder::from(&path, Arc::clone(&tc)) {
                            if let Ok(mut parent_lock) = sub_folder.parent.lock() {
                                *parent_lock = Arc::downgrade(&pf);
                            }
                            if let Ok(mut children_lock) = parent_folder.children.lock() {
                                children_lock.push(sub_folder);
                            }
                        }
                        tc.fetch_add(1, Ordering::SeqCst);
                    });

                    handles.push(handle);
                    return;
                }

                Err(_) => continue
            }
        }

        if let Ok(sub_folder) = Folder::from(&path, Arc::clone(&thread_count)) {
            if let Ok(mut parent_lock) = sub_folder.parent.lock() {
                *parent_lock = Arc::downgrade(&parent_folder);
            }
            if let Ok(mut children_lock) = parent_folder.children.lock() {
                children_lock.push(sub_folder);
            }
        }
    }

    pub fn get_path_rec(&self) -> String {
        if let Some(parent) = self.parent.lock().unwrap().upgrade() {
            format!("{}/{}", parent.get_path_rec(), self.name.as_str())
        } else {
            format!("path: {}", self.name)
        }
    }

    pub fn print_tree(&self) {
        self.print_tree_rec("", "")
    }

    fn print_tree_rec(&self, prefix: &str, space_holder: &str) {
        println!("{}{}", prefix, self.name);
        let children = self.children.lock().unwrap();
        for (idx, folder) in children.iter().enumerate() {
            let last = idx + 1 == children.len();
            let new_prefix = if last {"└──"} else {"├──"};
            let new_spaceholder = if last {"   "} else {"│  "};
            folder.print_tree_rec(
                format!("{}{}", space_holder, new_prefix).as_str(),
                format!("{}{}", space_holder, new_spaceholder).as_str()
            );
        }
    }

    pub fn navigate(self: &Arc<Folder>, direction: &str) -> Result<Arc<Folder>, String> {
        let mut current = direction;
        let mut folder = Arc::clone(self);

        while let Some((part, rest)) = current.split_once("/") {
            folder = Folder::nav_logic(part, &folder)?;
            current = rest;
        }

        if !current.is_empty() {
            folder = Folder::nav_logic(current, &folder)?;
        }

        Ok(folder)
    }

    fn nav_logic(dir: &str, current_folder: &Arc<Folder>) -> Result<Arc<Folder>, String> {
        match dir {
            "." => Ok(Arc::clone(&current_folder)),
            ".." => {
                if let Some(parent) = current_folder.parent.lock().unwrap().upgrade() {
                    Ok(Arc::clone(&parent))
                } else {
                    Err(format!(
                        "the folder named {} doesn't have a parent",
                        current_folder.name
                    ))
                }
            }
            other => {
                if let Some(sub_folder) = current_folder.children.lock().unwrap().iter().find(|child| child.name == other) {
                    Ok(Arc::clone(sub_folder))
                } else {
                    return Err(format!(
                        "the folder named {} doesn't have a child named {}",
                        current_folder.name, other
                    )
                    );
                }
            }
        }
    }

    pub fn summarize(&self) {
        println!("{}", self.get_path_rec());
        println!();
        self.print_tree();
    }
}