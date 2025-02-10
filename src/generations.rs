use std::fs;
use std::str;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

#[derive(Ord, Eq, Debug)]
pub struct Generation {
    number: usize,
    path: PathBuf,
    profile_path: PathBuf,
    age: u64,
}

impl Generation {
    fn new_from_direntry(name: &str, dirent: &fs::DirEntry) -> Result<Self, String> {
        let file_name = dirent.file_name();
        let file_name = file_name.to_string_lossy();
        let tokens: Vec<_> = file_name.split('-').collect();
        if tokens.len() != 3 || tokens[0] != name|| tokens[2] != "link" {
            return Err(format!("Cannot create generation representation"))
        }

        let profile_path = dirent.path().parent().unwrap()
            .join(name);

        let number = str::parse::<usize>(tokens[1])
            .map_err(|_| format!("Cannot parse \"{}\" as generation number", tokens[1]))?;

        let last_modified = fs::symlink_metadata(&dirent.path())
            .map_err(|e| format!("Unable to get metadata for path {} ({})", dirent.path().to_string_lossy(), e))?
            .modified()
            .map_err(|e| format!("Unable to get metadata for path {} ({})", dirent.path().to_string_lossy(), e))?;
        let now = SystemTime::now();
        let age = now.duration_since(last_modified)
            .map_err(|e| format!("Unable to calculate generation age ({})", e))?
            .as_secs() / 60 / 60 / 24;

        Ok(Generation {
            number, age,
            path: dirent.path(),
            profile_path,
        })
    }

    pub fn number(&self) -> usize {
        self.number
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn profile_path(&self) -> &Path {
        &self.profile_path
    }

    pub fn age(&self) -> u64 {
        self.age
    }
}

impl PartialOrd for Generation {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.number.partial_cmp(&other.number)
    }
}

impl PartialEq for Generation {
    fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }
}

pub fn user_generations(user: &str) -> Result<Vec<Generation>, String> {
    let mut generations: Vec<_> = fs::read_dir(format!("/nix/var/nix/profiles/per-user/{}", user))
        .map_err(|e| format!("Unable to read directory ({})", e))?
        .flatten()
        .filter(|e| e.file_name() != "profile")
        .map(|e| Generation::new_from_direntry("profile", &e))
        .flatten()
        .collect();
    generations.sort();
    Ok(generations)
}

