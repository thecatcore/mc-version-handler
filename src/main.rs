mod structs;

use reqwest::blocking::get as get_url;
use std::fs;
use std::path::{Path};
use lazy_static::lazy_static;
use std::fs::File;
use std::io::{Write, Read};
use fs_extra::dir::{copy as copy_dir, CopyOptions};

const MC_VERSIONS_URL: &'static str = "https://gist.github.com/arthurbambou/9dfda822df99359ab9dbd8a1970ff66b/raw";

lazy_static! {
    static ref ROOT_PATH: &'static Path = Path::new(".");
}

fn main() {
    let mut previous_version = String::from("");
    let mut server_updated = update_server(&previous_version);
    while server_updated.is_some() {
        previous_version = server_updated.unwrap();
        server_updated = if backup_server(&previous_version).is_some() {
            update_server(&previous_version)
        } else { None }
    }
}

fn backup_server(previous_version: &String) -> Option<String> {
    let mut copy_opt = CopyOptions::new();

    fs::create_dir(format!("./backup-{}", &previous_version)).ok()?;

    copy_dir(
        "./server",
        format!("./backup-{}", previous_version),
        &copy_opt
    ).ok()?;

    Some(String::from(""))
}

fn update_server(previous_version: &String) -> Option<String> {
    let file = get_url(MC_VERSIONS_URL).ok()?;

    let version_vec = structs::parse_version_manifest(file.text().ok()?.as_str()).ok()?.versions;

    let version = if previous_version.is_empty() {
        version_vec.first()?
    } else {
        let mut int = 0;
        for versions in &version_vec {
            if &versions.name == previous_version {
                int += 1;
                break;
            }
            int += 1;
        }

        version_vec.get(int)?
    }.clone();

    let server_dir = ROOT_PATH.clone().join("server");

    fs::create_dir_all(&server_dir).ok()?;

    let server_jar = server_dir.join("server.jar");

    if server_jar.exists() {
        fs::remove_file(&server_jar).ok()?;
    }

    let mut body: Vec<u8> = Vec::new();

    let mut data = get_url(version.url).ok()?;

    data.read_to_end(&mut body).ok()?;

    let mut server_jar_file = File::create(server_jar).ok()?;

    server_jar_file.write(&body).ok()?;

    Some(version.name)
}
