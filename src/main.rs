mod config;
mod utils;

use std::{
    env,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use clap::*;
use git2::{build::CheckoutBuilder, Direction, Repository};

use config::*;

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .subcommand(SubCommand::with_name("init"))
        .subcommand(SubCommand::with_name("update"))
        .get_matches();

    if matches.subcommand_matches("init").is_some() {
        init();
    } else if matches.subcommand_matches("update").is_some() {
        update();
    }
}

fn get_config() -> Config {
    let config_path = match env::var("GIT_MIRROR_CONFIG") {
        Ok(c) => c,
        _ => panic!("GIT_MIRROR_CONFIG isn't set."),
    };

    let mut config_file = match File::open(Path::new(&config_path)) {
        Ok(c) => c,
        _ => panic!("Error opening config file."),
    };

    let mut buf = String::new();
    if config_file.read_to_string(&mut buf).is_err() {
        panic!("Error reading config file.")
    }

    let config: Config = match toml::from_str(&buf) {
        Ok(c) => c,
        Err(e) => panic!("Error parsing config file. {}", e),
    };

    config
}

fn init() {
    let config = get_config();

    let projects = match config.projects {
        Some(p) => p,
        _ => return,
    };

    for project in projects {
        let mut path = PathBuf::new();
        path.push(Path::new(
            project.path.as_ref().unwrap_or(&config.default_path),
        ));

        match project.name {
            Some(name) => path.push(name),
            None => path.push(utils::get_repo_name(&project.url))
        };

        if path.exists() {
            println!("{:#?} already exists, skipping.", path);
            continue;
        }

        let _repo = match Repository::clone(&project.url, path) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };
    }
}

fn update() {
    let config = get_config();

    let projects = match config.projects {
        Some(p) => p,
        _ => return,
    };

    for project in projects {
        let mut path = PathBuf::new();
        path.push(Path::new(
            project.path.as_ref().unwrap_or(&config.default_path),
        ));

        match project.name {
            Some(name) => path.push(name),
            None => path.push(utils::get_repo_name(&project.url))
        };

        if !path.exists() {
            println!("{:#?} doesn't exist, skipping.", &path);
            continue;
        }

        let repo = match Repository::open(&path) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("{}", e);
                continue;
            }
        };

        let mut remote = repo.find_remote("origin").unwrap();
        remote.connect(Direction::Fetch).unwrap();
        let branch_buf = remote.default_branch().unwrap();
        let branch = branch_buf.as_str().unwrap();
        if remote.fetch(&[branch], None, None).is_err() {
            eprintln!("Failed fetching branch {} for {:#?}", branch, &path);
            continue;
        }

        let fetch_head = repo.find_reference("FETCH_HEAD").unwrap();
        let com = repo.reference_to_annotated_commit(&fetch_head).unwrap();
        let mut branch_ref = repo.find_reference(&branch).unwrap();
        branch_ref.set_target(com.id(), "IDK").unwrap();
        repo.set_head(&branch).unwrap();
        repo.checkout_head(Some(CheckoutBuilder::default().force()))
            .unwrap();
    }
}
