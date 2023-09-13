use std::env;
use std::path::{Path, PathBuf};

use fs_extra::dir::CopyOptions;

/*

fn generate_tesseract_bindings() {
    let path1 = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/third_party/tesseract/include/"
    ));
    let path1 = path1.canonicalize().unwrap();
    let clang_extra_include = vec![
        path1.display().to_string(),
        "/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/include/c++/v1/".to_string(),
    ];

    let mut capi_bindings = bindgen::Builder::default()
        .header(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/tesseract_sys/wrapper_capi.h"
        ))
        .detect_include_paths(true)
        .allowlist_function("^Tess.*")
        .blocklist_type("Boxa")
        .blocklist_type("Pix")
        .blocklist_type("Pixa")
        .blocklist_type("_IO_FILE")
        .blocklist_type("_IO_codecvt")
        .blocklist_type("_IO_marker")
        .blocklist_type("_IO_wide_data");

    for inc in clang_extra_include.iter() {
        capi_bindings = capi_bindings.clang_arg(format!("-I{}", inc));
    }

    let bindings = capi_bindings // if this fails on Mac, run "xcode-select --install" to install standard C headers
        .generate()
        .expect("Unable to generate capi bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    bindings
        .write_to_file(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/tesseract_sys/capi_bindings.rs"
        ))
        .expect("Couldn't write capi bindings!");

    fs::write(
        concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/tesseract_sys/public_types_bindings.rs"
        ),
        public_types_bindings(&clang_extra_include),
    )
    .expect("Couldn't write public types bindings!");
}

fn generate_leptonica_bindings() {
    let include_path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/third_party/leptonica/src/"
    ))
    .canonicalize()
    .unwrap();
    let bindings = bindgen::Builder::default()
        .header(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/third_party/leptonica/src/allheaders.h"
        ))
        .detect_include_paths(true)
        .clang_arg(format!("-I{}", include_path.display().to_string()));

    let bindings = bindings
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .generate()
        .expect("Unable to generate bindings");

    let out_path = Path::new(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/leptonica_sys/bindings.rs"
    ));
    bindings
        .write_to_file(out_path)
        .expect("Couldn't write bindings!");
}

#[cfg(not(target_os = "macos"))]
fn public_types_bindings(clang_extra_include: &[String]) -> String {
    let mut public_types_bindings = bindgen::Builder::default()
        .header(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/src/tesseract_sys/wrapper_public_types.hpp"
        ))
        .detect_include_paths(true)
        .allowlist_var("^k.*")
        .allowlist_var("^tesseract::k.*")
        .blocklist_item("^kPolyBlockNames")
        .blocklist_item("^tesseract::kPolyBlockNames");

    for inc in clang_extra_include {
        public_types_bindings = public_types_bindings.clang_arg(format!("-I{}", *inc));
    }

    public_types_bindings
        .generate()
        .expect("Unable to generate public types bindings")
        .to_string()
        .replace("tesseract_k", "k")
}

// MacOS clang is incompatible with Bindgen and constexpr
// https://github.com/rust-lang/rust-bindgen/issues/1948
// Hardcode the constants rather than reading them dynamically
#[cfg(target_os = "macos")]
fn public_types_bindings(_clang_extra_include: &[String]) -> &'static str {
    include_str!("src/tesseract_sys/public_types_bindings_mac.rs")
}

*/

fn download_leptonica() -> PathBuf {
    let source = "https://github.com/DanBloomberg/leptonica/archive/refs/tags/1.83.1.tar.gz";
    let target = Path::new(&env::var("OUT_DIR").unwrap()).join("leptonica");
    std::fs::create_dir_all(&target).unwrap();
    download_and_unpack(source, &target)
}

fn compile_leptonica(source_dir: &Path) -> (PathBuf, Vec<PathBuf>) {
    let out_dir = std::env::var("OUT_DIR").expect("no out dir");
    let base_dir = Path::new(&out_dir).join("leptonica");

    let _ = std::fs::create_dir_all(&base_dir);

    let _ = fs_extra::dir::copy(&source_dir, &base_dir, &CopyOptions::default());

    let base_dir = base_dir.join("leptonica").join("leptonica-1.83.1");

    println!("{}", base_dir.display());

    // Disable all image I/O except bmp and pnm files
    let environ_h_path = base_dir.join("src").join("environ.h");
    let environ_h = std::fs::read_to_string(&environ_h_path)
        .unwrap()
        .replace(
            "#define  HAVE_LIBJPEG       1",
            "#define  HAVE_LIBJPEG       0",
        )
        .replace(
            "#define  HAVE_LIBTIFF       1",
            "#define  HAVE_LIBTIFF       0",
        )
        .replace(
            "#define  HAVE_LIBPNG        1",
            "#define  HAVE_LIBPNG        0",
        )
        .replace(
            "#define  HAVE_LIBZ          1",
            "#define  HAVE_LIBZ          0",
        )
        .replace("#define  HAVE_LIBJPEG       0", "#undef HAVE_LIBJPEG")
        .replace("#define  HAVE_LIBTIFF       0", "#undef HAVE_LIBTIFF")
        .replace("#define  HAVE_LIBPNG        0", "#undef HAVE_LIBPNG")
        .replace("#define  HAVE_LIBZ          0", "#undef HAVE_LIBZ")
        .replace(
            "#ifdef  NO_CONSOLE_IO",
            "#define NO_CONSOLE_IO\n#ifdef  NO_CONSOLE_IO",
        );
    std::fs::write(environ_h_path, environ_h).unwrap();

    // configure cmake/Configure.cmake
    let configure_cmake_path = base_dir.join("cmake").join("Configure.cmake");
    let configure_cmake = std::fs::read_to_string(&configure_cmake_path)
        .unwrap()
        .replace("HAVE_LIBGIF 1", "HAVE_LIBGIF 0")
        .replace("HAVE_LIBJPEG 1", "HAVE_LIBJPEG 0")
        .replace("HAVE_LIBJP2K 1", "HAVE_LIBJP2K 0")
        .replace("HAVE_LIBPNG 1", "HAVE_LIBPNG 0")
        .replace("HAVE_LIBTIFF 1", "HAVE_LIBTIFF 0")
        .replace("HAVE_LIBWEBP 1", "HAVE_LIBWEBP 0")
        .replace("HAVE_LIBWEBP_ANIM 1", "HAVE_LIBWEBP_ANIM 0")
        .replace("HAVE_LIBZ 1", "HAVE_LIBZ 0");
    std::fs::write(configure_cmake_path, configure_cmake).unwrap();

    // Remove png, jpen, etc. from makefile.static
    let makefile_static_path = base_dir.join("prog").join("makefile.static");
    let makefile_static = std::fs::read_to_string(&makefile_static_path)
        .unwrap()
        .replace(
            "ALL_LIBS =	$(LEPTLIB) -ltiff -ljpeg -lpng -lz -lm",
            "ALL_LIBS =	$(LEPTLIB) -lm",
        );
    std::fs::write(makefile_static_path, makefile_static).unwrap();

    // Edit endianness.h to set endian
    #[cfg(target_endian = "big")]
    let target_endian = "#define L_BIG_ENDIAN\n";
    #[cfg(target_endian = "little")]
    let target_endian = "#define L_LITTLE_ENDIAN\n";
    std::fs::write(base_dir.join("src").join("endianness.h"), target_endian).unwrap();

    let dst = cmake::Config::new(&base_dir).always_configure(true).build();

    let library_path = dst
        .join("lib")
        .join("libleptonica.a")
        .canonicalize()
        .unwrap();

    (library_path, vec![dst.join("include").join("leptonica")])
}

fn download_tesseract() -> PathBuf {
    let source = "https://github.com/tesseract-ocr/tesseract/archive/refs/tags/5.3.0.tar.gz";
    let target = Path::new(&env::var("OUT_DIR").unwrap()).join("tesseract");
    std::fs::create_dir_all(&target).unwrap();
    download_and_unpack(source, &target)
}

fn download_and_unpack(url: &str, target: &PathBuf) -> PathBuf {
    use flate2::read::GzDecoder;
    use std::fs::File;
    use tar::Archive;

    let body = reqwest::blocking::get(url).unwrap().bytes().unwrap();
    std::fs::write(target.join("out.tar.gz"), body.as_ref()).unwrap();
    let tar_gz = File::open(&target.join("out.tar.gz")).unwrap();
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);
    archive.unpack(target).unwrap();
    target.clone()
}

fn compile_tesseract(source_dir: &Path) -> (PathBuf, Vec<PathBuf>) {
    let out_dir = std::env::var("OUT_DIR").expect("no out dir");
    let base_dir = Path::new(&out_dir).join("tesseract");

    let _ = std::fs::create_dir_all(&base_dir);

    let _ = fs_extra::dir::copy(&source_dir, &base_dir, &CopyOptions::default());

    let base_dir = base_dir.join("tesseract").join("tesseract-5.3.0");

    let dst = cmake::Config::new(&base_dir)
        .always_configure(true)
        .configure_arg("-DHAVE_LIBARCHIVE=OFF")
        .configure_arg("-DHAVELIBCURL=OFF")
        .configure_arg("-DHAVE_TIFFIO_H=OFF")
        .configure_arg("-DGRAPHICS_DISABLED=ON")
        .configure_arg("-DBUILD_TRAINING_TOOLS=OFF")
        .configure_arg("-DBUILD_TESTS=OFF")
        .configure_arg("-DUSE_SYSTEM_ICU=ON")
        .build();

    let library_path = dst
        .join("lib")
        .join("libtesseract.a")
        .canonicalize()
        .unwrap();

    (library_path, vec![dst.join("include").join("tesseract")])
}

fn main() {
    let (leptonica_lib, _leptonica_includes) = compile_leptonica(&download_leptonica());

    // generate_leptonica_bindings(&leptonica_includes);

    let (tesseract_lib, _tesseract_includes) = compile_tesseract(&download_tesseract());

    // generate_tesseract_bindings(&tesseract_includes);

    println!(
        "cargo:rustc-link-search={}",
        leptonica_lib.parent().unwrap().display()
    );
    println!(
        "cargo:rustc-link-search={}",
        tesseract_lib.parent().unwrap().display()
    );
    println!("cargo:rustc-link-lib=static=tesseract");
    println!("cargo:rustc-link-lib=static=leptonica");
    println!("cargo:rustc-link-lib=static:-bundle=c++"); // link libstdc++ for tesseract
}
