use std::env;

fn main() {
    if let Err(err) = shadow_rs::new() {
        eprintln!("Error using shadow_rs to retrieve build metadata: {:?}", err);
    }

    let target = env::var("TARGET").unwrap_or_else(|e| panic!("{}", e));
    if target.contains("android")  {
        // HACK: Somehow dependencies are not linking these libraries for android builds?
        println!("cargo:rustc-link-lib=EGL");
    }
}