#![feature(try_blocks)]

use std::{
    cell::RefCell,
    collections::HashMap,
    env,
    fs::{read_dir, File},
    io::{Read, Write},
    path::Path,
};

use anyhow::{anyhow, Result};
use prost_build::compile_protos;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=../proto");
    println!("cargo:rerun-if-changed=../template");
    println!("cargo:rerun-if-changed=../asset");

    build_proto()?;
    load_template()?;
    load_asset()?;

    Ok(())
}

fn build_proto() -> Result<()> {
    compile_protos(&["../proto/search.proto"], &["../proto/"])?;
    Ok(())
}

fn load_template() -> Result<()> {
    let out_path = env::var("OUT_DIR")?;
    let out_path = Path::new(&out_path).join("template.rs");

    let mut file = File::create(out_path)?;
    file.write_all(b"pub const TEMPLATES: &'static [(&'static str, &'static [u8])] = &[\n")?;

    let source = Path::new("../template");

    let paths = read_dir(source)?
        .map(|e| try {
            let e = e?;
            let path = e.path();
            let path = path.to_str().ok_or(anyhow!("path is not utf8"))?;
            path.to_string()
        })
        .collect::<Result<Vec<_>>>()?;

    for path in paths {
        let mut template = File::open(&path)?;
        let mut buffer = vec![];
        template.read_to_end(&mut buffer)?;

        let name = &path["../template/".len()..];

        let template = format!("    (\"{name}\", &{buffer:?}),\n");
        file.write_all(template.as_bytes())?;
    }

    file.write_all(b"];\n")?;

    Ok(())
}

fn load_asset() -> Result<()> {
    let mime_types = read_mime_types()?;

    let assets = RefCell::new(vec![]);
    find_asset("../asset", &assets)?;

    let assets = assets.into_inner();
    let out_path = env::var("OUT_DIR")?;
    let out_path = Path::new(&out_path).join("asset.rs");

    let mut file = File::create(out_path)?;
    file.write_all(
        b"pub const ASSETS: &'static [(&'static str, &'static [u8], &'static str)] = &[\n",
    )?;

    for path in assets {
        let mut asset = File::open(&path)?;
        let mut buffer = vec![];
        asset.read_to_end(&mut buffer)?;

        let path = &path["../asset/".len()..];

        let mime_type = {
            let ext = {
                let parts: Vec<_> = path.split(".").collect();
                parts[parts.len() - 1]
            };

            &mime_types[ext]
        };

        let asset = format!("    (\"{path}\", &{buffer:?}, \"{mime_type}\"),\n");
        file.write_all(asset.as_bytes())?;
    }

    file.write_all(b"];\n")?;

    Ok(())
}

fn find_asset(path: &str, assets: &RefCell<Vec<String>>) -> Result<()> {
    let path = Path::new(path);

    if path.is_dir() {
        let entries = read_dir(path)?;
        for entry in entries {
            let entry = entry?;
            let entry_path = entry.path();
            find_asset(
                entry_path.to_str().ok_or(anyhow!("path is not utf8"))?,
                assets,
            )?;
        }
    } else {
        let path = path.to_str().ok_or(anyhow!("path is not utf8"))?;
        assets.borrow_mut().push(path.to_string());
    }

    Ok(())
}

fn read_mime_types() -> Result<HashMap<String, String>> {
    let mut mime_types = File::open("../mime.types")?;
    let mut buffer = vec![];
    mime_types.read_to_end(&mut buffer)?;

    let mime_types: HashMap<_, _> = String::from_utf8(buffer)?
        .split("\n")
        .into_iter()
        .map(|s| s.trim())
        .filter(|s| s.len() > 0)
        .filter(|s| !s.starts_with('#'))
        .flat_map(|s| {
            let line: Vec<_> = s.split_whitespace().collect();
            let mime = line[0];
            line.into_iter().skip(1).map(move |ext| {
                let mime = mime;
                (ext, mime)
            })
        })
        .map(|(ext, mime)| (ext.to_owned(), mime.to_owned()))
        .collect();

    Ok(mime_types)
}
