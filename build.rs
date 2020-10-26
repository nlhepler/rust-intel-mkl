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

use bzip2::read::BzDecoder;
use curl::easy::Easy;
use tar::Archive;

use std::env::var;
use std::fs::{self, File};
use std::io::*;
use std::path::*;

// Use `conda search --json --platform 'win-64' mkl-static`
// to query the metadata of conda package (includes MD5 sum).

#[cfg(target_os = "linux")]
mod mkl {
    pub const LIB_PATH: &'static str = "lib";

    pub const DLS: &[(&'static str, &'static str, &'static str)] = &[(
        "mkl-static-2020.4-intel_304.tar.bz2",
        "https://conda.anaconda.org/intel/linux-64/mkl-static-2020.4-intel_304.tar.bz2",
        "9f589a1508fb083c3e73427db459ca4c",
    )];
}

#[cfg(target_os = "macos")]
mod mkl {
    pub const LIB_PATH: &'static str = "lib";

    pub const DLS: &[(&'static str, &'static str, &'static str)] = &[(
        "mkl-static-2020.4-intel_301.tar.bz2",
        "https://conda.anaconda.org/intel/osx-64/mkl-static-2020.4-intel_301.tar.bz2",
        "2f9e1b8b6d6b0903e81a573084e4494f",
    )];
}

#[cfg(target_os = "windows")]
mod mkl {
    pub const LIB_PATH: &'static str = "Library\\lib";

    pub const DLS: &[(&'static str, &'static str, &'static str)] = &[(
        "mkl-static-2020.4-intel_311.tar.bz2",
        "https://conda.anaconda.org/intel/win-64/mkl-static-2020.4-intel_311.tar.bz2",
        "5ae780c06edd0be62966c6d8ab47d5fb",
    )];
}

fn download(uri: &str, filename: &str, out_dir: &Path) {
    let out = PathBuf::from(out_dir.join(filename));

    // Download the tarball.
    let f = File::create(&out).unwrap();
    let mut writer = BufWriter::new(f);
    let mut easy = Easy::new();
    easy.follow_location(true).unwrap();
    easy.autoreferer(true).unwrap();
    easy.url(&uri).unwrap();
    easy.write_function(move |data| Ok(writer.write(data).unwrap()))
        .unwrap();
    easy.perform().unwrap();

    let response_code = easy.response_code().unwrap();
    if response_code != 200 {
        panic!("Unexpected response code {} for {}", response_code, uri);
    }
}

fn calc_md5(path: &Path) -> String {
    let mut f = BufReader::new(fs::File::open(path).unwrap());
    let mut buf = Vec::new();
    f.read_to_end(&mut buf).unwrap();

    let digest = md5::compute(&buf);
    format!("{:x}", digest)
}

fn extract<P: AsRef<Path>, P2: AsRef<Path>>(archive_path: P, extract_to: P2) {
    let file = File::open(archive_path).unwrap();
    let unzipped = BzDecoder::new(file);
    let mut a = Archive::new(unzipped);
    a.unpack(extract_to).unwrap();
}

fn main() {
    let out_dir = PathBuf::from(var("OUT_DIR").unwrap());

    for (archive, uri, md5) in mkl::DLS {
        let archive_path = out_dir.join(archive);
        if archive_path.exists() && calc_md5(&archive_path) == *md5 {
            println!("Use existings archive");
        } else {
            println!("Download archive");
            download(uri, archive, &out_dir);
            extract(&archive_path, &out_dir);

            let sum = calc_md5(&archive_path);
            if sum != *md5 {
                panic!(
                    "check sum of downloaded archive is incorrect: md5sum={}",
                    sum
                );
            }
        }
    }

    println!(
        "cargo:rustc-link-search={}",
        out_dir.join(mkl::LIB_PATH).display()
    );

    // mkl_intel_ilp64 links to a version w/ 64-bit ints,
    // mkl_intel_lp64 links to a version w/ 32-bit ints.
    // existing binding need lp64
    println!("cargo:rustc-link-lib=static=mkl_intel_lp64");
    println!("cargo:rustc-link-lib=static=mkl_sequential");
    println!("cargo:rustc-link-lib=static=mkl_core");
}
