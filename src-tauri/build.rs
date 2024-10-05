use std::{
    env,
    fs::{self, File},
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use curl::easy::Easy;
use flate2::read::GzDecoder;
use tar::Archive;

const VERSION: &str = "6666";

macro_rules! get(($name:expr) => (ok!(env::var($name))));
macro_rules! ok(($expression:expr) => ($expression.unwrap()));
macro_rules! log {
    ($fmt:expr) => (println!(concat!("src-tauri/build.rs:{}: ", $fmt), line!()));
    ($fmt:expr, $($arg:tt)*) => (println!(concat!("src-tauri/build.rs:{}: ", $fmt),
    line!(), $($arg)*));
}
macro_rules! log_var(($var:ident) => (log!(concat!(stringify!($var), " = {:?}"), $var)));

fn main() {
    install_prebuilt();
    tauri_build::build();
}

fn target_os() -> String {
    match get!("CARGO_CFG_TARGET_OS").as_str() {
        "windows" => "win".to_string(),
        "macos" => "mac".to_string(),
        _ => "unsupported".to_string(),
    }
}

fn target_arch() -> String {
    match get!("CARGO_CFG_TARGET_ARCH").as_str() {
        "x86" => "x86".to_string(),
        "x86_64" => "x64".to_string(),
        _ => "unsupported".to_string(),
    }
}

fn remove_suffix(value: &mut String, suffix: &str) {
    if value.ends_with(suffix) {
        let n = value.len();
        value.truncate(n - suffix.len());
    }
}

fn extract_tar_gz<P: AsRef<Path>, P2: AsRef<Path>>(archive_path: P, extract_to: P2) {
    let file = File::open(archive_path).unwrap();
    let unzipped = GzDecoder::new(file);
    let mut a = Archive::new(unzipped);
    a.unpack(extract_to).unwrap();
}

fn dll_suffix() -> &'static str {
    match &target_os() as &str {
        "windows" => ".dll",
        "macos" => ".dylib",
        _ => ".so",
    }
}

fn install_prebuilt() {
    // create the url
    let binary_url = format!(
        "https://github.com/bblanchon/pdfium-binaries/releases/download/chromium%2F{}/pdfium-{}-{}.tgz",
        VERSION,
        target_os(),
        target_arch()
    );
    log_var!(binary_url);
    let short_file_name = binary_url.split('/').last().unwrap();
    let mut base_name = short_file_name.to_string();
    remove_suffix(&mut base_name, "tgz");
    log_var!(base_name);
    let download_dir = match env::var("TF_RUST_DOWNLOAD_DIR") {
        Ok(s) => PathBuf::from(s),
        Err(_) => PathBuf::from(&get!("OUT_DIR")),
    };
    if !download_dir.exists() {
        fs::create_dir(&download_dir).unwrap();
    }
    let file_name = download_dir.join(short_file_name);
    log_var!(file_name);

    // Download the tgz.
    if !file_name.exists() {
        let f = File::create(&file_name).unwrap();
        let mut writer = BufWriter::new(f);
        let mut easy = Easy::new();
        easy.follow_location(true).unwrap();
        easy.url(&binary_url).unwrap();
        easy.write_function(move |data| Ok(writer.write(data).unwrap()))
            .unwrap();
        easy.perform().unwrap();

        let response_code = easy.response_code().unwrap();
        if response_code != 200 {
            panic!(
                "Unexpected response code {} for {}",
                response_code, binary_url
            );
        }
    }
    // Extract the tgz file
    let unpacked_dir = download_dir.join(base_name);
    let lib_dir = unpacked_dir.join("bin");
    // let library_file = format!("{}{}", "pdfium", dll_suffix());

    //let library_full_path = lib_dir.join(&library_file);

    extract_tar_gz(file_name, &unpacked_dir);

    // NOTE: The following shouldn't strictly be necessary. See note above `extract`.
    let framework_files = std::fs::read_dir(lib_dir).unwrap();
    for library_entry in framework_files.filter_map(Result::ok) {
        let library_full_path = library_entry.path();
        let new_library_full_path = get_output_path().join(library_full_path.file_name().unwrap());
        if new_library_full_path.exists() {
            log!(
                "{} already exists. Removing",
                new_library_full_path.display()
            );
            fs::remove_file(&new_library_full_path).unwrap();
        }
        log!(
            "Copying {} to {}...",
            library_full_path.display(),
            new_library_full_path.display()
        );
        fs::copy(&library_full_path, &new_library_full_path).unwrap();
    }
}

fn get_output_path() -> PathBuf {
    //<root or manifest path>/target/<profile>/
    let manifest_dir_string = env::var("CARGO_MANIFEST_DIR").unwrap();
    let build_type = env::var("PROFILE").unwrap();
    let path = Path::new(&manifest_dir_string)
        .join("target")
        .join(build_type);
    return PathBuf::from(path);
}
