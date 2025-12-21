use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();
    println!("cargo::rustc-env=TARGET={target}");

    let profile = env::var("PROFILE").unwrap();
    println!("cargo::rustc-env=PROFILE={profile}");

    println!("cargo::rustc-check-cfg=cfg(backend, values(\"winit\", \"android\"))");

    if cfg!(target_os = "android") {
        println!("cargo::rustc-cfg=backend=\"android\"");
    } else if cfg!(all(
        feature = "winit",
        any(
            target_family = "unix",
            target_os = "macos",
            target_os = "windows",
        )
    )) {
        println!("cargo::rustc-cfg=backend=\"winit\"");
    }
}
