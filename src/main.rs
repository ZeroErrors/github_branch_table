extern crate chrono;
extern crate dotenv;
extern crate prettytable;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;

use chrono::DateTime;
use chrono::Utc;
use clap::{App, Arg};

mod github;
mod output;

fn try_load(path: &Path) -> Result<HashMap<String, github::Branch>, Box<Error>> {
    let file = File::open(&path)?;
    Ok(serde_json::from_reader(file)?)
}

fn get_or_load(repo: &str, settings: &Settings) -> Result<HashMap<String, github::Branch>, Box<Error>> {
    if settings.cache_enabled {
        let file_string = format!("{}.json", repo.replace("/", "_"));
        let path = Path::new(file_string.as_str());
        match try_load(&path) {
            Ok(result) => return Ok(result),
            Err(err) => println!("Failed to load file {}: {}", file_string, err),
        }
    }

    let mut server_branches = HashMap::new();
    github::list_branches(repo, 1, &mut server_branches, settings)?;

    if settings.cache_enabled {
        let file_string = format!("{}.json", repo.replace("/", "_"));
        let path = Path::new(file_string.as_str());
        let mut file = File::create(&path)?;
        let json_string = serde_json::to_string_pretty(&server_branches)?;
        file.write_all(json_string.as_bytes())?;
    }

    Ok(server_branches)
}

fn get_latest_modified_date(map: &HashMap<String, github::Branch>) -> Result<DateTime<Utc>, Box<Error>> {
    let mut vec: Vec<DateTime<Utc>> = vec![];
    for branch in map.values() {
        vec.push(branch.last_updated.parse::<DateTime<Utc>>()?)
    }
    vec.sort();
    Ok(vec[0])
}

fn store_branches(branches: &mut HashMap<String, HashMap<String, github::Branch>>, repo: &str, settings: &Settings) -> Result<(), Box<Error>> {
    for (key, branch) in get_or_load(repo, settings)? {
        match branches.get_mut(&key) {
            Some(map) => {
                map.insert(repo.to_owned(), branch);
            }
            None => {
                let mut map = HashMap::new();
                map.insert(repo.to_owned(), branch);
                branches.insert(key.to_owned(), map);
            }
        }
    }
    Ok(())
}

#[derive(Clone, Debug)]
pub struct Settings {
    cache_enabled: bool,
    github_token: Option<String>,
}

fn main() -> Result<(), Box<Error>> {
    dotenv::dotenv().ok();

    let matches = App::new("GitHub Branch Table")
        .version("1.0")
        .author("Zero <https://github.com/ZeroErrors>")
        .about("Generates a table output of branches for multiple repositories allowing for easy comparison.")
        .arg(Arg::with_name("cache")
            .short("c")
            .long("cache")
            .help("Enable the repo cache"))
        .arg(Arg::with_name("token")
            .short("t")
            .long("token")
            .takes_value(true)
            .help("GitHub token used when making API requests"))
        .arg(Arg::with_name("REPOS")
            .help("GitHub repo's to include in the table")
            .required(true)
            .multiple(true)
            .index(1))
        .get_matches();

    let cache_enabled = matches.occurrences_of("cache") > 0;
    let github_token = matches.value_of("token").map(|s| s.to_owned())
        .or(std::env::var("GITHUB_TOKEN").ok());

    let settings = Settings {
        cache_enabled,
        github_token,
    };

    let repos: Vec<&str> = matches.values_of("REPOS").ok_or("No repos specified!")?.collect();

    // Fetch data from GitHub or cache
    let mut branches: HashMap<String, HashMap<String, github::Branch>> = HashMap::new();
    for repo in &repos {
        store_branches(&mut branches, repo, &settings)?;
    }

    // Sort branches
    let mut branches: Vec<_> = branches.iter().collect();
    branches.sort_by(|one, two| {
        let date = get_latest_modified_date(one.1).unwrap();
        let other_date = get_latest_modified_date(two.1).unwrap();
        date.cmp(&other_date)
    });
    branches.reverse();

    output::print(branches, repos);
    Ok(())
}
