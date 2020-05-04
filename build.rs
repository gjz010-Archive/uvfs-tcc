fn main() {
    println!("cargo:rustc-link-search=c://msys64//home//gjz010//tcc-0.9.27");
    println!("cargo:rustc-cdylib-link-arg=-Wl,--retain-symbols-file=tcc.def");

}
