// MIT License
//
// Copyright (c) 2017 Toshiki Teramura
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

use std::env::var;
use std::path::*;
use std::process::Command;
use std::fs;
use std::io::*;

#[cfg(target_os = "linux")]
const MKL_ARCHIVE: &'static str = "mkl_linux.tar.xz";
#[cfg(target_os = "macos")]
const MKL_ARCHIVE: &'static str = "mkl_osx.tar.xz";

fn download(uri: &str, filename: &str, out_dir: &Path) {
    let out = out_dir.join(filename);
    let mut f = BufWriter::new(fs::File::create(out).unwrap());
    let p = Command::new("curl").arg(uri).output().expect(
        "Failed to start download",
    );
    f.write(&p.stdout).unwrap();
}

fn check_sum(check_sum_path: &Path, dir: &Path) -> bool {
    Command::new("md5sum")
        .args(&["-c", check_sum_path.to_str().unwrap()])
        .current_dir(dir)
        .status()
        .expect("Failed to check md5 sum")
        .success()
}

fn expand(archive: &Path, out_dir: &Path) {
    let st = Command::new("tar")
        .args(&["xvf", archive.to_str().unwrap()])
        .current_dir(&out_dir)
        .status()
        .expect("Failed to start expanding archive");
    if !st.success() {
        panic!("Failed to expand archive");
    }
}

fn main() {
    let crate_root = PathBuf::from(var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(var("OUT_DIR").unwrap());

    let oid = "4e492799a3ca2366653a59d946c0fee89b489fce";
    let uri = format!(
        "https://raw.githubusercontent.com/termoshtt/rust-intel-mkl/{}/mkl_lib/{}",
        oid,
        MKL_ARCHIVE
    );
    let archive_path = out_dir.join(MKL_ARCHIVE);
    let md5sum_path = crate_root.join(format!("mkl_lib/{}.md5", MKL_ARCHIVE));

    if !archive_path.exists() {
        download(&uri, MKL_ARCHIVE, &out_dir);
    }
    if !check_sum(&md5sum_path, &out_dir) {
        panic!("check sum of archive is incorrect");
    }
    expand(&archive_path, &out_dir);

    println!("cargo:rustc-link-search={}", out_dir.display());
    println!("cargo:rustc-link-lib=dylib=mkl_intel_lp64");
    println!("cargo:rustc-link-lib=dylib=mkl_gnu_thread");
    println!("cargo:rustc-link-lib=dylib=mkl_core");
    println!("cargo:rustc-link-lib=dylib=gomp");
}
