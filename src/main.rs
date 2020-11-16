mod config;
mod utils;

use std::{
    borrow::Cow,
    env,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    result::Result
};

use clap::*;
use git2::{build::CheckoutBuilder, Direction, Repository};

use config::*;

fn main() {
    let matches = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("config")
            .short("c")
            .long("config")
            .value_name("FILE")
            .help("Sets a custom config file")
            .takes_value(true))
        .get_matches();

    let config = get_config(matches.value_of("config"));

    let projects = match &config.projects {
        Some(p) => p,
        _ => return,
    };

    for project in projects {
        let mut path = PathBuf::new();
        path.push(Path::new(
            project.path.as_ref().unwrap_or(&config.default_path),
        ));

        match &project.name {
            Some(name) => path.push(name),
            None => path.push(utils::get_repo_name(&project.url))
        };

        if path.exists() {
            if let Err(e) = update(&path) {
                eprintln!("Error updating repository {:?}; {}", &path, e);
            }

            continue;
        }

        if let Err(e) = init(&path, &project) {
            eprintln!("Error initializing repository {:?}; {}", &path, e);
        }
    }
}

fn get_config(config: Option<&str>) -> Config {
    let config_path = match config {
        Some(c) => Cow::Borrowed(c),
        None => match env::var("GIT_MIRROR_CONFIG") {
            Ok(c) => Cow::Owned(c),
            _ => panic!("GIT_MIRROR_CONFIG isn't set."),
        }
    };

    let s: &str = &config_path;
    let mut config_file = match File::open(Path::new(s)) {
        Ok(c) => c,
        _ => panic!("Error opening config file."),
    };

    let mut buf = String::new();
    if config_file.read_to_string(&mut buf).is_err() {
        panic!("Error reading config file.")
    }

    match toml::from_str(&buf) {
        Ok(c) => c,
        Err(e) => panic!("Error parsing config file. {}", e),
    }
}

fn init(path: &Path, project: &ProjectConfig)  -> Result<(), git2::Error> {
    let _repo = Repository::clone(&project.url, path)?;
    if let Some(d) = &project.description {
        utils::set_description(path, d).unwrap();
    }

    Ok(())
}

fn update(path: &Path) -> Result<(), git2::Error> {
    let repo = Repository::open(&path)?;

    // Fetch remote
    let mut remote = repo.find_remote("origin")?;
    remote.connect(Direction::Fetch)?;
    let branch_buf = remote.default_branch()?;
    let branch = branch_buf.as_str().unwrap();
    remote.fetch(&[branch], None, None)?;

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let com = repo.reference_to_annotated_commit(&fetch_head)?;
    let mut branch_ref = repo.find_reference(&branch)?;
    branch_ref.set_target(com.id(), "IDK")?;
    repo.set_head(&branch)?;
    repo.checkout_head(Some(CheckoutBuilder::default().force()))?;
    Ok(())
}
