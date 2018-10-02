extern crate time;

use std::collections::HashMap;
use std::rc::Rc;
//RefCell provides with references, Cell with values.alloc
//RefCell may panic
//cell.borrow_mut().unwrap()
//Cell will never let you get a pointer to the value, RefCell would.
use std::cell::{Cell, RefCell};
use inode::{Inode};
//self??
use self::File::{DataFile, Directory};

//Rc is a shared pointer basically, RefCell has reference of underlying data on the heap
pub type RcDirContent<'r> = Rc<RefCell<Box<DirectoryContent<'r>>>>;
pub type RcInode = Rc<RefCell<Box<Inode>>>;

// File is a thing wrapper around Inodes and Directories. The whole point is to
// provide a layer of indirection. FileHandle's and Directory entries, then,
// point to these guys instead of directly to Inodes/Directories

// deriving Clone allows for copying underlying data on the heap
// Rust never automatically creates deep copies.  automatic copies gives
// very poor performance. So this derive clone allows for deep copying the
// data on the heap. for example, the following would allow for s1 and s2 to
// be printed and referenced. If the .clone() was not there, then it would give error because
// Rust's ownership concept ensures memory safety by preventing double free problem
// that would occur when both s1 and s2 go out of scope.
// let s1 = String::from("hello");
// let s2 = s1.clone();
//
// println!("s1 = {}, s2 = {}", s1, s2);
// `
#[derive(Clone)]
pub enum File<'r> {
  DataFile(RcInode),
  Directory(RcDirContent<'r>),
  EmptyFile
}

#[derive(Clone)]
pub struct FileHandle<'r> {
  file: File<'r>,
  seek: Cell<usize>
}

#[derive(Clone)]
pub struct DirectoryContent<'r> {
  pub entries: HashMap<&'r str, File<'r>>
}

pub enum Whence {
  SeekSet,
  SeekCur,
  SeekEnd
}

impl<'r> File<'r> {
  pub fn new_dir(_parent: Option<File<'r>>) -> File<'r> {
    let content = Box::new(DirectoryContent { entries: HashMap::new() });
    let rc = Rc::new(RefCell::new(content));
    let dir = Directory(rc);

    // Note that dir is RCd, so this is cheap
    // Used to borrow dir and mut_dir at "same time"
    // RefCell makes sure we're not doing anything wrong
    // let mut mut_dir = dir.clone();

    // // Setting up "." and ".."
    // mut_dir.insert(".", dir.clone());
    // match parent {
    //   None => mut_dir.insert("..", dir.clone()),
    //   Some(f) => mut_dir.insert("..", f)
    // }

    dir
  }

  pub fn new_data_file(inode: RcInode) -> File<'r> {
    DataFile(inode)
  }

  pub fn get_dir_rc<'a>(&'a self) -> &'a RcDirContent<'r> {
    match self {
      &Directory(ref rc) => rc,
      _ => panic!("not a directory")
    }
  }

  pub fn get_inode_rc<'a>(&'a self) -> &'a RcInode {
    match self {
      &DataFile(ref rc) => rc,
      _ => panic!("not a directory")
    }
  }
}

impl<'r> FileHandle<'r> {
  // Probably not the right type.
  pub fn new(file: File<'r>) -> FileHandle<'r> {
    FileHandle {
      file: file,
      seek: Cell::new(0)
    }
  }

  pub fn read(&self, dst: &mut [u8]) -> usize {
    let offset = self.seek.get();
    let inode_rc = self.file.get_inode_rc();
    let changed = inode_rc.borrow().read(offset, dst);
    self.seek.set(offset + changed);
    changed
  }

  pub fn write(&mut self, src: &[u8]) -> usize {
    let offset = self.seek.get();
    let inode_rc = self.file.get_inode_rc();
    let changed = inode_rc.borrow_mut().write(offset, src);
    self.seek.set(offset + changed);
    changed
  }

  pub fn seek(&mut self, offset: isize, whence: Whence) -> usize {
    let inode_rc = self.file.get_inode_rc();

    let seek = self.seek.get();
    let new_seek = match whence {
      Whence::SeekSet => offset as usize,
      Whence::SeekCur => (seek as isize + offset) as usize,
      Whence::SeekEnd => (inode_rc.borrow().size() as isize + offset) as usize
    };

    self.seek.set(new_seek);
    new_seek
  }
}
