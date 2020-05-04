#![feature(alloc)]
#![feature(lang_items)]
#![feature(alloc_error_handler)]
#![feature(link_args)]
#![crate_type = "cdylib"]
#![crate_id = "r"]
extern crate alloc;
use alloc::sync::Arc;
use alloc::string::String;
use libc;
use alloc::collections::BTreeMap;
use spin::Mutex;
use core::convert::TryInto;
use core::alloc::{GlobalAlloc, Layout};
use core::panic::PanicInfo;
use crate::tarfs::{initialize_tarfs, TARFILE_FS_OPS};

mod tcc;
mod tarfs;
mod rustfs;
#[link(name = "hello")]

pub unsafe fn from_cstr(s: *const u8) -> &'static str {
    use core::{slice, str};
    let len = (0usize..).find(|&i| *s.add(i) == 0).unwrap();
    str::from_utf8(slice::from_raw_parts(s, len)).unwrap()
}

static mut UV_FS: Option<Arc<Mutex<UVFS>>> = None;
// We disable std so that it can be put anywhere.
pub type Fd = i32;
#[no_mangle]
pub extern "C" fn vopen(pathname: *const u8, flags: i32, modes: i32) -> Fd{
    let mut fs=unsafe {UV_FS.as_ref().unwrap().lock()};
    let path=unsafe{from_cstr(pathname)};
    let pair=fs.choose_provider(path);
    let ops=pair.0;
    let name=pair.1;
    let mut data:usize=0;
    let fd=(ops.open)(&mut data, name.as_ptr(), flags, modes);
    fs.add_file(File {
        data,
        fd,
        ops: Arc::clone(&ops)
    });
    fd
}
#[no_mangle]
pub extern "C" fn vread(f: Fd, buf: *mut u8, count: usize) -> isize{
    let mut fs=unsafe{UV_FS.as_ref().unwrap().lock()};
    let file=fs.get_file(f);
    (file.ops.read)(file.data, f, buf, count)
}
#[no_mangle]
pub extern "C" fn vwrite(f: Fd, buf: *const u8, count: usize) -> isize {
    let mut fs=unsafe{UV_FS.as_ref().unwrap().lock()};
    let file=fs.get_file(f);
    (file.ops.write)(file.data, f, buf, count)
}
#[no_mangle]
pub extern "C" fn vlseek(f: Fd, offset: i32, whence: i32) -> i32 {
    let mut fs=unsafe{UV_FS.as_ref().unwrap().lock()};
    let file=fs.get_file(f);
    (file.ops.lseek)(file.data, f, offset, whence)
}
#[no_mangle]
pub extern "C" fn vclose(f: Fd) -> i32 {
    let mut fs=unsafe{UV_FS.as_ref().unwrap().lock()};
    let file=fs.remove_file(f);
    match file{
        Some(file)=>(file.ops.close)(file.data, f),
        None=>0
    }
}
#[no_mangle]
pub extern "C" fn vvdup(oldfd: i32)->i32{
    //println!("DUP not supported!");
    libc::ENOSYS
}
struct File {
    data: usize,
    fd: Fd,
    ops: Arc<FileOperations>
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct FileOperations{
    open: extern "C" fn(data: *mut usize, pathname: *const u8, flags: i32, modes: i32) -> Fd,
    read: extern "C" fn(data: usize,f: Fd, buf: *mut u8, count: usize) -> isize,
    write: extern "C" fn(data: usize,f: Fd, buf: *const u8, count: usize) -> isize,
    lseek: extern "C" fn(data: usize,f: Fd, offset: i32, whence: i32) -> i32,
    close: extern "C" fn(data: usize,f: Fd) -> i32
}

extern "C" fn default_open(data: *mut usize, pathname: *const u8, flags: i32, modes: i32) -> Fd{
    unsafe {*data=0};
    if (flags & libc::O_CREAT)!=0 {
        unsafe { libc::open(pathname as *const i8, flags, modes) }
    }else{
        unsafe { libc::open(pathname as *const i8, flags) }
    }
}
extern "C" fn default_read(data: usize, f: Fd, buf: *mut u8, count: usize) -> isize{
    unsafe {libc::read(f, buf as *mut core::ffi::c_void, count.try_into().unwrap()).try_into().unwrap()}
}
extern "C" fn default_write(data: usize, f: Fd, buf: *const u8, count: usize) -> isize {
    unsafe {libc::write(f, buf as *mut core::ffi::c_void, count.try_into().unwrap()).try_into().unwrap()}
}
extern "C" fn default_lseek(data: usize, f: Fd, offset: i32, whence: i32) -> i32 {
    unsafe {libc::lseek(f, offset, whence)}
}
extern "C" fn default_close(data: usize, f: Fd) -> i32 {
    unsafe {libc::close(f)}
}

#[no_mangle]
pub extern "C" fn reserve_fd()->Fd{

    let fd=unsafe {
        let mut v=[0;2];
        libc::pipe(v.as_mut_ptr(), 0, libc::O_BINARY);
        libc::close(v[1]);
        v[0]
    };
    assert!(fd!=-1);
    fd
}
#[no_mangle]
pub extern "C" fn free_fd(fd: Fd){
    unsafe {libc::close(fd);}
}

#[no_mangle]
pub extern "C" fn register_operation(name: *const u8, ops: *const FileOperations){
    unsafe {
        let name = from_cstr(name);
        let mut fs=unsafe {UV_FS.as_ref().unwrap().lock()};
        fs.register_operation(name, *ops);
    }
}
struct UVFS {
    registeredOps: BTreeMap<String, Arc<FileOperations>>,
    openedFiles: BTreeMap<i32, Arc<File>>,
    defaultOps : Arc<FileOperations>
}
#[no_mangle]
pub extern "C" fn initialize_uvfs(){
    //println!("UVFS initializing.");
    let mut uvfs = UVFS {
        registeredOps: BTreeMap::new(),
        openedFiles: BTreeMap::new(),
        defaultOps: Arc::new(FileOperations{
            open: default_open,
            read: default_read,
            write: default_write,
            lseek: default_lseek,
            close: default_close
        })
    };
    initialize_tarfs();
    uvfs.register_operation("tcc", TARFILE_FS_OPS);
    unsafe {UV_FS.replace(Arc::new(Mutex::new(uvfs)))};
}
impl UVFS {
    pub fn choose_provider<'a>(&self, s: &'a str)->(Arc<FileOperations>, &'a str){
        let mut splitter = s.splitn(2, "@uvfs://");
        let first=splitter.next();
        let snd=splitter.next();
        //println!("{:?} {:?}", first, snd);
        match snd{
            Some(file)=>{
                (self.get_registered_operations(first.unwrap()), file)
            }
            None=>(Arc::clone(&self.defaultOps), s)
        }
    }
    pub fn add_file(&mut self, f: File)->Arc<File>{
        let af=Arc::new(f);
        self.openedFiles.insert(af.fd, Arc::clone(&af));
        af
    }
    pub fn remove_file(&mut self, fd: Fd)->Option<Arc<File>>{
        self.openedFiles.remove(&fd)
    }
    pub fn register_operation(&mut self, s: &str, ops: FileOperations) {
        self.registeredOps.insert(String::from(s), Arc::new(ops));
    }
    pub fn get_registered_operations(&self, s: &str)->Arc<FileOperations>{
        Arc::clone(self.registeredOps.get(s).unwrap_or(&self.defaultOps))
    }
    pub fn get_file(&mut self, fd: Fd)->Arc<File>{
        match self.openedFiles.get(&fd){
            Some(f)=>Arc::clone(f),
            None=>{ // Add system fd on the fly.
                let f=self.add_file(File{
                    data: 0,
                    fd,
                    ops: Arc::clone(&self.defaultOps)
                });
                f
            }
        }
    }
}
/*
extern "C"{
#[no_mangle]
fn _aligned_malloc(size : usize, alignment: usize) -> *mut u8;
#[no_mangle]
fn _aligned_free(item : *mut u8);
}
struct LibCAllocator;
unsafe impl GlobalAlloc for LibCAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        _aligned_malloc(layout.size(), layout.align())
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        _aligned_free(ptr);
    }
}
#[global_allocator]
static Alloc: LibCAllocator  = LibCAllocator;

#[cfg(not(test))]#[lang = "eh_personality"] fn eh_personality() {}
#[cfg(not(test))]#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    loop {}
}
#[cfg(not(test))]#[alloc_error_handler]
fn oom(_layout: Layout)  -> ! {
    loop {}
}
*/

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

pub mod hello{
    #[no_mangle]
    pub extern fn foo(a: i32) -> i32 {
        1
    }
}