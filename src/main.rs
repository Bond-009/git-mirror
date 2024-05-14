mod config;
mod utils;

use std::{
    borrow::Cow,
    env,
    fs,
    path::{Path, PathBuf},
    result::Result
};

use clap::*;
use git2::{build::CheckoutBuilder, Direction, FetchPrune, FetchOptions, Repository};

use config::*;

fn main() {
    let matches = command!()
        .name(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            arg!(
            -c --config <FILE> "Sets a custom config file"
            ).value_parser(value_parser!(PathBuf)),
        )
        .get_matches();

    let config = get_config(matches.get_one::<PathBuf>("config"));

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

fn get_config(config: Option<&PathBuf>) -> Config {
    let config_path: Cow<PathBuf> = match config {
        Some(c) => Cow::Borrowed(c),
        None => match env::var("GIT_MIRROR_CONFIG") {
            Ok(c) => Cow::Owned(PathBuf::from(c)),
            _ => panic!("GIT_MIRROR_CONFIG isn't set."),
        }
    };

    let config_file = match fs::read_to_string(config_path.as_path()) {
        Ok(c) => c,
        _ => panic!("Error reading config file."),
    };

    match toml::from_str(&config_file) {
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

    let mut fetch_opt = FetchOptions::default();
    fetch_opt.prune(FetchPrune::On);

    remote.fetch(&[branch], Some(&mut fetch_opt), None)?;

    let fetch_head = repo.find_reference("FETCH_HEAD")?;
    let com = repo.reference_to_annotated_commit(&fetch_head)?;
    let mut branch_ref = repo.find_reference(&branch)?;
    branch_ref.set_target(com.id(), "IDK")?;
    repo.set_head(&branch)?;
    repo.checkout_head(Some(CheckoutBuilder::default().force()))?;
    Ok(())
}
