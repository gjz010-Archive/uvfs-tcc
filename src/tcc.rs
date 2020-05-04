use libc::*;
use libc;
type ptr = usize;
#[doc(hidden)]
#[macro_export]
macro_rules! export_c_symbol {
    ($name2:ident, $ret:ty : $name:ident($( $type:ty : $arg:ident  ),*) ) => {
        #[link(name="tcc")] extern "C" {#[no_mangle] pub fn $name($( $arg : $type),*) -> $ret;}
        #[no_mangle]
        pub unsafe extern "C" fn $name2 ($( $arg : $type),*) -> $ret {
            $name($( $arg ),*)
        }
    };
    (void $name:ident($( $arg:ident : $type:ty ),*)) => {
        export_c_symbol!(fn $name($( $arg : $type),*) -> ());
    }
}
fn bar(){

}
type void=();
export_c_symbol!(r_tcc_new, ptr:tcc_new());
export_c_symbol!(r_tcc_delete, void:tcc_delete(ptr:s));
export_c_symbol!(r_tcc_set_lib_path,void:tcc_set_lib_path(ptr:s,ptr:path));
export_c_symbol!(r_tcc_set_error_func,void:tcc_set_error_func(ptr:s,ptr:error_opaque,ptr:errfunc));
export_c_symbol!(r_tcc_set_options,void:tcc_set_options(ptr:s,ptr:str));
export_c_symbol!(r_tcc_add_include_path,libc::c_int:tcc_add_include_path(ptr:s,ptr:pathname));
export_c_symbol!(r_tcc_add_sysinclude_path,libc::c_int:tcc_add_sysinclude_path(ptr:s,ptr:pathname));
export_c_symbol!(r_tcc_define_symbol,void:tcc_define_symbol(ptr:s,ptr:sym,ptr:value));
export_c_symbol!(r_tcc_undefine_symbol,void:tcc_undefine_symbol(ptr:s,ptr:sym));
export_c_symbol!(r_tcc_add_file,libc::c_int:tcc_add_file(ptr:s,ptr:filename));
export_c_symbol!(r_tcc_compile_string,libc::c_int:tcc_compile_string(ptr:s,ptr:buf));
export_c_symbol!(r_tcc_set_output_type,libc::c_int:tcc_set_output_type(ptr:s,libc::c_int:output_type));
export_c_symbol!(r_tcc_add_library_path,libc::c_int:tcc_add_library_path(ptr:s,ptr:pathname));
export_c_symbol!(r_tcc_add_library,libc::c_int:tcc_add_library(ptr:s,ptr:libraryname));
export_c_symbol!(r_tcc_add_symbol,libc::c_int:tcc_add_symbol(ptr:s,ptr:name,ptr:val));
export_c_symbol!(r_tcc_output_file,libc::c_int:tcc_output_file(ptr:s,ptr:filename));
export_c_symbol!(r_tcc_run,libc::c_int:tcc_run(ptr:s,libc::c_int:argc,ptr:argv));
export_c_symbol!(r_tcc_relocate,libc::c_int:tcc_relocate(ptr:s1,ptr:ptr));
export_c_symbol!(r_tcc_get_symbol,ptr:tcc_get_symbol(ptr:s,ptr:name));
