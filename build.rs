use std::env;
use std::fs;
use std::io::Write;
use std::path::PathBuf;

const HF_BASE_URL: &str = "https://huggingface.co/jc4st3lls/needle_lib/resolve/main";

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
        ("macos", "x86_64") => "x86_64",
        ("linux", "aarch64") => "aarch64",
        ("linux", "x86_64") => "x86_64",
        ("windows", "x86_64") => "amd64",
        ("windows", "aarch64") => "arm64",
        _ => panic!("Arquitectura no soportada para {}: {}", target_os, target_arch),
    };

    let lib_filename = match target_os.as_str() {
        "macos" => "libneedle.dylib",
        "linux" => "libneedle.so",
        "windows" => "libneedle.dll",
        _ => unreachable!(),
    };

    // ~/.needle/<os>/<arch>/
    let home_dir = user_home_dir();
    let needle_root = home_dir.join(".needle");
    let native_dir = needle_root.join(os_dir).join(arch_dir);

    if !native_dir.exists() {
        fs::create_dir_all(&native_dir)
            .unwrap_or_else(|e| panic!("No se pudo crear la carpeta {}: {}", native_dir.display(), e));
    }

    let lib_path = native_dir.join(lib_filename);
    if !lib_path.exists() {
        download_library(os_dir, arch_dir, lib_filename, &lib_path);
    }

    // Enlaza contra libneedle
    println!("cargo:rustc-link-search=native={}", native_dir.display());
    println!("cargo:rustc-link-lib=dylib=needle");

    // Para que el binario encuentre la lib en runtime sin exportar LD_LIBRARY_PATH/DYLD a mano.
    if target_os == "macos" || target_os == "linux" {
        println!("cargo:rustc-link-arg=-Wl,-rpath,{}", native_dir.display());
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

/// Devuelve la carpeta de usuario según el SO.
fn user_home_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        // En Windows, normalmente USERPROFILE ya viene definida.
        if let Ok(p) = env::var("USERPROFILE") {
            return PathBuf::from(p);
        }
        // Fallback por si USERPROFILE no estuviera disponible.
        let drive = env::var("HOMEDRIVE")
            .expect("No se pudo determinar el directorio de usuario (USERPROFILE/HOMEDRIVE+HOMEPATH)");
        let path = env::var("HOMEPATH")
            .expect("No se pudo determinar el directorio de usuario (USERPROFILE/HOMEDRIVE+HOMEPATH)");
        PathBuf::from(format!("{}{}", drive, path))
    } else {
        // macOS y Linux
        env::var("HOME")
            .map(PathBuf::from)
            .expect("No se pudo determinar el directorio de usuario ($HOME)")
    }
}

/// Descarga la librería nativa desde Hugging Face a `dest`.
fn download_library(os_dir: &str, arch_dir: &str, lib_filename: &str, dest: &PathBuf) {
    let url = format!("{}/{}/{}/{}", HF_BASE_URL, os_dir, arch_dir, lib_filename);

    println!(
        "cargo:warning=needle_lib: descargando {} (puede tardar unos segundos)...",
        url
    );

    let response = ureq::get(&url)
        .call()
        .unwrap_or_else(|e| panic!("Fallo al descargar {}: {}", url, e));

    // Descarga a un fichero temporal y luego lo renombra, para no dejar
    // un fichero corrupto/incompleto si la descarga falla a medias.
    let tmp_path = dest.with_extension("part");
    {
        let mut file = fs::File::create(&tmp_path)
            .unwrap_or_else(|e| panic!("No se pudo crear {}: {}", tmp_path.display(), e));
        let mut reader = response.into_body().into_reader();
        std::io::copy(&mut reader, &mut file)
            .unwrap_or_else(|e| panic!("Fallo al escribir {}: {}", tmp_path.display(), e));
        file.flush().ok();
    }

    fs::rename(&tmp_path, dest).unwrap_or_else(|e| {
        panic!(
            "No se pudo mover {} a {}: {}",
            tmp_path.display(),
            dest.display(),
            e
        )
    });

    println!("cargo:warning=needle_lib: descargada en {}", dest.display());
}