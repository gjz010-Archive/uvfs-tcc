use crate::{Fd, FileOperations, reserve_fd, free_fd};
use std::io::prelude::*;
use std::fs::File;
use tar::Archive;
use std::vec::*;
use std::collections::BTreeMap;
use std::sync::Arc;
use libc::{SEEK_CUR, SEEK_END, SEEK_SET};
use std::io::Read;
use spin::Mutex;
use alloc::boxed::*;
use std::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};
use std::slice::{from_raw_parts_mut, from_raw_parts};

#[link(name="tccinc")]
extern "C"{
    fn binary_tccinc_tar_end()->!;
    fn binary_tccinc_tar_size()->!;
    fn binary_tccinc_tar_start()->!;
}

pub static TAR_FS: FileOperations = FileOperations{
    open: open_tarfile,
    read: read_tarfile,
    write: write_tarfile,
    lseek: lseek_tarfile,
    close: close_tarfile
};
pub struct TarFileFS {
    tree: BTreeMap<String, Arc<Vec<u8>>>
}
pub struct TarFile{
    file: Arc<Vec<u8>>,
    cursor: usize
}
impl TarFileFS{
    pub fn new(arr: &[u8])->TarFileFS {
        let mut archive = Archive::new(arr);
        let mut tree = BTreeMap::new();
        for file in archive.entries().unwrap() {
            let mut file = file.unwrap();
            //println!("{:?}", file.path());
            let mut v = Vec::new();
            file.read_to_end(&mut v).unwrap();
            tree.insert(String::from(file.path().unwrap().to_str().unwrap()), Arc::new(v));

        }
        //println!("Done");
        TarFileFS { tree }
    }
    pub fn open(&self, pathname: &str, flags: i32, modes: i32)->Option<TarFile>{
        let v=self.tree.get(pathname)?;
        Some(TarFile{
            file: Arc::clone(v),
            cursor: 0
        })
    }

}
impl TarFile{
    pub fn read(&mut self, f: Fd, buf: &mut [u8]) -> isize{
        let mut slice=&self.file.as_slice()[self.cursor..];
        let sz=slice.read(buf).unwrap();
        self.cursor+=sz;
        sz as isize
    }
    pub fn write(&mut self, f: Fd, buf: &[u8]) -> isize{
        -1 // No writing.
    }
    pub fn lseek(&mut self, f: Fd, offset: i32, whence: i32) -> i32{
        if whence==SEEK_CUR{
            let mut t=self.cursor as isize;
            t = t + offset as isize;
            if t<0 {
                t=0;
            }else if t>self.file.len() as isize{
                t= self.file.len() as isize;
            }
            self.cursor= t as usize;
        }else if whence == SEEK_SET {
            assert!(offset>=0);
            self.cursor= offset as usize;
            if self.cursor>self.file.len(){
                self.cursor=self.file.len();
            }
        }else if whence == SEEK_END{
            self.cursor=self.file.len();
        }
        return self.cursor as i32;
    }
    pub fn close(&mut self, f: Fd) -> i32{
        0
    }
}
fn unref_mut<T: 'static>(data: usize)->&'static mut T{
    unsafe {return &mut *(data as *mut T);}
}
fn unref<T: 'static>(data: usize)->&'static T{
    unsafe {return &*(data as *const T);}
}

pub static mut TARFILE_FS: Option<Mutex<TarFileFS>> = None;
pub static TARFILE_FS_OPS  : FileOperations= FileOperations{
    open: open_tarfile,
    read: read_tarfile,
    write: write_tarfile,
    lseek: lseek_tarfile,
    close: close_tarfile
};
pub fn initialize_tarfs(){
    unsafe {
        TARFILE_FS.replace(Mutex::new(TarFileFS::new(std::slice::from_raw_parts(binary_tccinc_tar_start as *const u8, binary_tccinc_tar_size as usize))));
    }
}
pub extern "C" fn open_tarfile(data: *mut usize, pathname: *const u8, flags: i32, modes: i32) -> Fd{
    //println!("Open Tar File {}", unsafe {crate::from_cstr(pathname)});
    let fs=unsafe {TARFILE_FS.as_ref().unwrap().lock()};
    //println!("Open Tar File");
    let file=fs.open(unsafe {crate::from_cstr(pathname)}, flags, modes);
    //println!("Open Tar File");
    let file = match file{
        None=>return -libc::ENOENT,
        Some(x)=>x
    };
    //println!("Open Tar File");
    let file=Box::new(file);
    //println!("Open Tar File");
    unsafe {*data = Box::leak(file) as *mut _ as usize;}
    //println!("Open Tar File Done");
    let fd=reserve_fd();
    //println!("Open Tar File Done {}", fd);
    fd

}
pub extern "C" fn read_tarfile(data: usize,f: Fd, buf: *mut u8, count: usize) -> isize{
    //println!("Read Tar File");
    let tarfile=unref_mut::<TarFile>(data);
    tarfile.read(f, unsafe {from_raw_parts_mut(buf, count)})

}
pub extern "C" fn write_tarfile(data: usize,f: Fd, buf: *const u8, count: usize) -> isize{
    //println!("Write Tar File");
    let tarfile=unref_mut::<TarFile>(data);
    tarfile.write(f, unsafe {from_raw_parts(buf, count)})
}
pub extern "C" fn lseek_tarfile(data: usize,f: Fd, offset: i32, whence: i32) -> i32{
    //println!("Lseek Tar File");
    let tarfile=unref_mut::<TarFile>(data);
    tarfile.lseek(f, offset, whence)
}
pub extern "C" fn close_tarfile(data: usize,f: Fd) -> i32{
    //println!("Close Tar File");
    let tarfile=unref_mut::<TarFile>(data);
    let mut tarfile=unsafe {Box::from_raw(tarfile as *mut TarFile)};
    free_fd(f);
    tarfile.close(f)

}
