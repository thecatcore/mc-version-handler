mod structs;

use chrono::{Duration, Local};
use fs_extra::dir::{copy as copy_dir, CopyOptions};
use lazy_static::lazy_static;
use reqwest::blocking::get as get_url;
use serbo::Manager;
use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

const MC_VERSIONS_URL: &'static str =
    "https://gist.github.com/arthurbambou/9dfda822df99359ab9dbd8a1970ff66b/raw";

lazy_static! {
    static ref ROOT_PATH: &'static Path = Path::new(".");
}

fn main() {
    let mut previous_version = String::from("");
    let mut server_updated = update_server(&previous_version);
    match server_updated {
        None => {}
        Some(ver) => {
            previous_version = ver;
            server_updated = launch_server(&previous_version);
            while server_updated.is_some() {
                previous_version = server_updated.unwrap();
                server_updated = if let Some(ver) = backup_server(&previous_version) {
                    if let Some(ver) = update_server(&ver) {
                        previous_version = ver;
                        launch_server(&previous_version)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }
}

fn launch_server(previous_version: &String) -> Option<String> {
    let mut manager = Manager::new();

    manager.start(25565).ok()?;
    println!("Starting server in: {}", previous_version);

    let start_date = Local::now();

    let stop_date = start_date + /*Duration::days(10);*/ Duration::minutes(5);

    let warn_date = stop_date - Duration::minutes(5);

    let mut warned = false;

    loop {
        if manager.is_online() {
            let instance = manager.get().unwrap();

            if !warned
                && Local::now() >= warn_date
                && instance.is_valid().is_ok()
                && instance.is_valid().unwrap()
            {
                instance.send(format!(
                    "say The server will stop in 5 minutes to upgrade to new version!"
                )).ok()?;
                instance.send(format!(
                    "/say The server will stop in 5 minutes to upgrade to new version!"
                )).ok()?;
                warned = true;
            } else if Local::now() >= stop_date
                && instance.is_valid().is_ok()
                && instance.is_valid().unwrap()
            {
                instance.send(format!("stop")).ok()?;
                instance.started();
                break;
            } else if instance.is_valid().is_ok()
                && !instance.is_valid().unwrap() {
                break;
            }
        }
    };

    match manager.stop() {
        Ok(_) => {}
        Err(err) => {
            println!("Error while stopping server: {}", err)
        }
    };

    println!("Server stopped");

    Some(previous_version.clone())
}

fn backup_server(previous_version: &String) -> Option<String> {
    let copy_opt = CopyOptions::new();

    fs::create_dir(format!("./backup-{}", &previous_version));

    copy_dir(
        "./server",
        format!("./backup-{}", &previous_version),
        &copy_opt,
    );

    println!("Server backuped");

    Some(previous_version.clone())
}

fn update_server(previous_version: &String) -> Option<String> {
    println!("hmm");
    let file = get_url(MC_VERSIONS_URL).ok()?;

    println!("Got file");

    let version_vec = structs::parse_version_manifest(file.text().ok()?.as_str())
        .ok()?
        .versions;

    println!("{}", version_vec.len());

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
    }
    .clone();

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

    println!("Server updated from version {} to version {}", previous_version, version.name.clone());

    Some(version.name)
}
