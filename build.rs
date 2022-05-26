use std::env;

fn main() {
    let target = env::var("TARGET").unwrap_or_else(|e| panic!("{}", e));

    if target.contains("android")  {
        // HACK: Somehow dependencies are not linking these libraries for android builds?
        println!("cargo:rustc-link-lib=EGL");
        // println!("cargo:rustc-link-lib=GLESv2");
    }
}