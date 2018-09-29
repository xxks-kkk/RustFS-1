use file::File;
use file::File::Directory;

// Traits are similar to a feature often called interfaces in other languages,
// although with some differences.
// Sized: https://doc.rust-lang.org/std/marker/trait.Sized.html
pub trait DirectoryHandle<'r>: Sized {
  // The Self keyword is an alias for the type we?re implementing the traits or methods on.
  fn is_dir(&self) -> bool;
  fn insert(&mut self, name: &'r str, file: Self);
  fn remove(&mut self, name: &'r str);
  // return Option<Self> idnicates that whatever type implements the trait,
  // we will return the type wrapped as an option
  // https://doc.rust-lang.org/book/2018-edition/ch17-02-trait-objects.html
  fn get(&self, name: &'r str) -> Option<Self>;
}

impl<'r> DirectoryHandle<'r> for File<'r> {
  fn is_dir(&self) -> bool {
    match self {
      &Directory(_) => true,
      _ => false
    }
  }

  fn insert(&mut self, name: &'r str, file: File<'r>) {
    let rc = self.get_dir_rc();
    let mut content = rc.borrow_mut();
    content.entries.insert(name, file);
  }

  fn remove(&mut self, name: &'r str) {
    let rc = self.get_dir_rc();
    let mut content = rc.borrow_mut();
    content.entries.remove(&name);
  }

  fn get(&self, name: &'r str) -> Option<File<'r>> {
    let rc = self.get_dir_rc();
    let content = rc.borrow();
    match content.entries.get(&name) {
      None => None,
      Some(ref file) => Some((*file).clone()) // It's RC
    }
  }
}
