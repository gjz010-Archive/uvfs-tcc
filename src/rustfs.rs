/*
use super::Fd;
use crate::FileOperations;
use std::sync::Arc;

pub trait RustFile{
    fn read(&mut self, f: Fd, buf: *mut u8, count: usize) -> isize;
    fn write(&mut self, f: Fd, buf: *const u8, count: usize) -> isize;
    fn lseek(&mut self, f: Fd, offset: i32, whence: i32) -> i32;
    fn close(&mut self, data: usize) -> i32;
}
pub trait RustFS<T: RustFile> {
    fn open(&mut self, pathname: &str, flags: i32, modes: i32) -> T;
}

pub struct RustFSWrapper<T: RustFile, F: RustFS<T>>{
    fs: RustFS<T>
}

extern "C" fn open_rustfile<T: RustFile, F: RustFS<T>>(data: *mut usize, pathname: *const u8, flags: i32, modes: i32) -> Fd{

}
extern "C" fn read_rustfile(data: usize,f: Fd, buf: *mut u8, count: usize) -> isize{

}
extern "C" fn write_rustfile(data: usize,f: Fd, buf: *const u8, count: usize) -> isize{

}
extern "C" fn lseek_rustfile(data: usize,f: Fd, offset: i32, whence: i32) -> i32{

}
extern "C" fn close_rustfile(data: usize,f: Fd) -> i32{

}
*/
