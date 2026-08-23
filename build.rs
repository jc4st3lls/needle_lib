use std::env;
use std::path::PathBuf;

fn main() {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();

    let os_dir = match target_os.as_str() {
        "macos" => "macos",
        "linux" => "linux",
        "windows" => "windows",
        _ => panic!("Sistema operativo no soportado por needle_lib: {}", target_os),
    };

    let arch_dir = match (target_os.as_str(), target_arch.as_str()) {
        ("macos", "aarch64") => "arm64",
        ("linux", "aarch64") => "aarch64",
        ("linux", "x86_64") => "x86_64",
        ("windows", "x86_64") => "x86_64",
        ("windows", "aarch64") => "arm64",
        _ => panic!("Arquitectura no soportada para {}: {}", target_os, target_arch),
    };

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let native_dir = manifest_dir.join("native").join(os_dir).join(arch_dir);

    if !native_dir.exists() {
        panic!(
            "La carpeta de librería estática nativa no existe: {}",
            native_dir.display()
        );
    }

    // Enlaza contra la librería estática libneedle.a
    println!("cargo:rustc-link-search=native={}", native_dir.display());
    println!("cargo:rustc-link-lib=static=needle");

    // Enlazar con la librería estándar de C++ según el sistema operativo
    match target_os.as_str() {
        "macos" => {
            println!("cargo:rustc-link-lib=c++");
        }
        "linux" => {
            println!("cargo:rustc-link-lib=stdc++");
        }
        "windows" => {
            // En Windows con MSVC, libcpmt/msvcprt suele enlazarse automáticamente.
        }
        _ => {}
    }

    println!("cargo:rerun-if-changed=wrapper.h");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("no se pudo generar los bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("no se pudo escribir bindings.rs");
}