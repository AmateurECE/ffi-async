//! This build script copies the `memory.x` file from the crate root into
//! a directory where the linker can always find it at build time.
//! For many projects this is optional, as the linker always searches the
//! project root directory -- wherever `Cargo.toml` is. However, if you
//! are using a workspace or have a more complicated build setup, this
//! build script becomes required. Additionally, by requesting that
//! Cargo re-run the build script whenever `memory.x` is changed,
//! updating `memory.x` ensures a rebuild of the application with the
//! new memory settings.

use std::env;
use std::fs::File;
use std::io::Write;
use std::os::unix::fs;
use std::path::PathBuf;

fn main() {
    // Put `memory.x` in our output directory and ensure it's
    // on the linker search path.
    let out = &PathBuf::from(env::var_os("OUT_DIR").unwrap());
    File::create(out.join("memory.x"))
        .unwrap()
        .write_all(include_bytes!("memory.x"))
        .unwrap();
    println!("cargo:rustc-link-search={}", out.display());

    // By default, Cargo will re-run a build script whenever
    // any file in the project changes. By specifying `memory.x`
    // here, we ensure the build script is only re-run when
    // `memory.x` is changed.
    println!("cargo:rerun-if-changed=memory.x");

    println!("cargo:rustc-link-arg-bins=--nmagic");
    println!("cargo:rustc-link-arg-bins=-Tlink.x");
    println!("cargo:rustc-link-arg-bins=-Tdefmt.x");

    // Compile libapp
    println!("cargo:rerun-if-changed=app.c");
    println!("cargo:rerun-if-changed=await.c");
    println!("cargo:rerun-if-changed=syscalls.c");
    println!("cargo:rerun-if-changed=sysmem.c");
    let destination = cmake::Config::new(".")
        .define("CMAKE_TRY_COMPILE_TARGET_TYPE", "STATIC_LIBRARY")
        .define("CMAKE_EXPORT_COMPILE_COMMANDS", "ON")
        .build();
    let app_dir = PathBuf::from(destination.display().to_string()).join("build");
    println!(
        "cargo:rustc-link-search=native={}",
        app_dir.as_path().to_str().unwrap()
    );
    println!("cargo:rustc-link-lib=static=app");
    println!("cargo:rustc-link-lib=c");
    println!(
        "cargo:rustc-link-search=arm-gnu-toolchain-15.2.rel1-aarch64-arm-none-eabi/arm-none-eabi/lib/thumb/v7e-m+fp/hard/"
    );

    let compile_commands = PathBuf::from("compile_commands.json");
    if std::fs::exists(&compile_commands).unwrap() {
        std::fs::remove_file(&compile_commands).unwrap();
    }
    fs::symlink(
        app_dir
            .join("compile_commands.json")
            .as_path()
            .to_str()
            .unwrap(),
        &compile_commands,
    )
    .unwrap();

    // Generate bindings for libapp
    let bindings = bindgen::Builder::default()
        // The input header we would like to generate
        // bindings for.
        .header("src/wrapper.h")
        // Use core instead of std
        .use_core()
        .clang_arg("-Iarm-gnu-toolchain-15.2.rel1-aarch64-arm-none-eabi/arm-none-eabi/include")
        // Tell cargo to invalidate the built crate whenever any of the
        // included header files changed.
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        // Finish the builder and generate the bindings.
        .generate()
        // Unwrap the Result and panic on failure.
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
