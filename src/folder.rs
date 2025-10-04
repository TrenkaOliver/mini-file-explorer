use std::{cell::RefCell, ffi::OsStr, fs, io, path::Path, rc::{Rc, Weak}};

#[derive(Debug)]
pub struct Folder {
    name: String,
    parent: RefCell<Weak<Folder>>,
    children: RefCell<Vec<Rc<Folder>>>,
}

impl Folder {
    pub fn new(name: &str) -> Folder {
        Folder { 
            name: String::from(name), 
            parent: RefCell::new(Weak::new()), 
            children: RefCell::new(vec![])
        }
    }

    pub fn from(path: &Path) -> Result<Rc<Folder>, io::Error> {
        let folder = Rc::new(Folder::new(path.file_name().unwrap_or(&OsStr::new("root")).to_str().unwrap()));
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let entry = entry?;
                if let Ok(sub_folder) = Folder::from(entry.path().as_path()) {
                    *sub_folder.parent.borrow_mut() = Rc::downgrade(&folder);
                    folder.children.borrow_mut().push(sub_folder);
                }
            }
        }
        Ok(folder)
    }

    pub fn get_path_rec(&self) -> String {
        if let Some(parent) = self.parent.borrow().upgrade() {
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
        for (idx, folder) in self.children.borrow().iter().enumerate() {
            let last = idx + 1 == self.children.borrow().len();
            let new_prefix = if last {"└──"} else {"├──"};
            let new_spaceholder = if last {"   "} else {"│  "};
            folder.print_tree_rec(
                format!("{}{}", space_holder, new_prefix).as_str(),
                format!("{}{}", space_holder, new_spaceholder).as_str()
            );
        }
    }

    pub fn navigate(self: &Rc<Folder>, direction: &str) -> Result<Rc<Folder>, String> {
        let mut current = direction;
        let mut folder = Rc::clone(self);

        while let Some((part, rest)) = current.split_once("/") {
            folder = Folder::nav_logic(part, &folder)?;
            current = rest;
        }

        if !current.is_empty() {
            folder = Folder::nav_logic(current, &folder)?;
        }

        Ok(folder)
    }

    fn nav_logic(dir: &str, current_folder: &Rc<Folder>) -> Result<Rc<Folder>, String> {
        match dir {
            "." => Ok(Rc::clone(current_folder)),
            ".." => {
                if let Some(parent) = current_folder.parent.borrow().upgrade() {
                    Ok(Rc::clone(&parent))
                } else {
                    Err(format!(
                        "the folder named {} doesn't have a parent",
                        current_folder.name
                    ))
                }
            }
            other => {
                if let Some(sub_folder) = current_folder.children.borrow().iter().find(|child| child.name == other) {
                    Ok(Rc::clone(sub_folder))
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