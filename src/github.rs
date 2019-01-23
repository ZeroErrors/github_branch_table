use std::collections::HashMap;
use std::error::Error;
use std::sync::mpsc;
use std::thread;

use serde_json::Value;

use crate::Settings;

#[derive(Debug, Serialize, Deserialize)]
pub struct Branch {
    pub last_updated_by: String,
    pub last_updated: String,
    // TODO: Use DateTime<Utc> here?
    pub pr: Option<u64>,
}

fn request(url: &str, settings: &Settings) -> reqwest::Result<reqwest::Response> {
    let builder = reqwest::Client::builder()
        .build()?
        .get(url)
        .header("Accept", "application/vnd.github.v3+json");
    if let Some(token) = &settings.github_token {
        builder.header("Authorization", format!("token {}", token))
            .send()
    } else {
        builder.send()
    }
}

fn parse_link_header(links: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for link in links.split(",") {
        let parts: Vec<&str> = link.split(";").collect();
        let url = parts[0].trim_matches(&[' ', '<', '>'] as &[char]);
        let rel_parts: Vec<&str> = parts[1].split("=").collect();
        let rel_value = rel_parts[1].trim_matches(&[' ', '"'] as &[char]);
        map.insert(rel_value.to_owned(), url.to_owned());
    }
    map
}

fn get_branch_pr(repo: &str, branch_name: &str, settings: &Settings) -> Result<Option<u64>, Box<Error>> {
    let repo_split: Vec<&str> = repo.split("/").collect();
    let owner = repo_split[0];
    let url = format!("https://api.github.com/repos/{}/pulls?head={}:{}", repo, owner, branch_name);
    let value: Value = request(&url, settings)?.json()?;
    let array: &Vec<Value> = value.as_array().ok_or("Expected Array!")?;
    match array.get(0) {
        Some(first_pr) => {
            let value: &Value = &first_pr["number"];
            Ok(Some(value.as_u64().ok_or("Expected u64!")?))
        }
        None => Ok(None)
    }
}

fn get_branch_info(repo: &str, branch_name: &str, settings: &Settings) -> Result<Branch, Box<Error>> {
    let url = format!("https://api.github.com/repos/{}/branches/{}", repo, branch_name);
    let value: Value = request(&url, settings)?.json()?;
    let commit = value["commit"].as_object().ok_or("Expected Object!")?;
    let last_updated_by = match &commit["author"] {
        Value::Object(obj) => obj["login"].as_str().ok_or("Expected author login to be String!")?,
        _ => commit["commit"]["author"]["name"].as_str().ok_or("Expected author name to be String!")?
    };
    let last_updated = commit["commit"]["author"]["date"].as_str().ok_or("Expected date to be String!")?;

    let pr = get_branch_pr(repo, branch_name, settings)?;

    Ok(Branch {
        last_updated_by: last_updated_by.to_owned(),
        last_updated: last_updated.to_owned(),
        pr,
    })
}

pub fn list_branches(repo: &str, page: u16, branches: &mut HashMap<String, Branch>, settings: &Settings) -> Result<(), Box<Error>> {
    println!("Get branches in {} (page {})", repo, page);
    let url = format!("https://api.github.com/repos/{}/branches?page={}", repo, page);
    let mut response = request(&url, settings)?;

    println!("Rate Limit: {}, Remaining: {}, Reset: {}",
             response.headers().get("X-RateLimit-Limit").unwrap().to_str().unwrap(),
             response.headers().get("X-RateLimit-Remaining").unwrap().to_str().unwrap(),
             response.headers().get("X-RateLimit-Reset").unwrap().to_str().unwrap());

    let next = match response.headers().get("Link") {
        Some(link) => parse_link_header(link.to_str().unwrap()).contains_key("next"),
        None => false,
    };
    let value: Value = response.json()?;

    let status = response.status().as_u16();
    if status != 200 {
        let message = value["message"].as_str().ok_or("Expected 'message' to be String!")?;
        return Err(format!("API Error ({}): {}, {}", repo, status, message).into());
    }

    let mut thread_results: Vec<(String, mpsc::Receiver<Result<Branch, ()>>)> = vec![];

    for b in value.as_array().ok_or("Expected Array!")? {
        let obj = b.as_object().ok_or("Expected Object!")?;
        let branch_name = obj["name"].as_str().ok_or("Expected 'name' to be a String")?;

        let (sender, receiver) = mpsc::channel();
        thread_results.push((branch_name.to_owned(), receiver));

        let settings = settings.clone();
        let repo = repo.to_owned();
        let branch_name = branch_name.to_owned();
        thread::spawn(move || {
            let branch_info = get_branch_info(&repo, &branch_name, &settings);
            if let Err(err) = &branch_info {
                println!("Failed to get branch info for {} in {}: {:?} ", branch_name, repo, err);
            }
            sender.send(branch_info.map_err(|_| ())).unwrap();
        });
    }

    while !thread_results.is_empty() {
        thread_results.retain(|pair| {
            match pair.1.try_recv() {
                Ok(result) => { // Thread finished
                    if let Ok(branch) = result {
                        branches.insert(pair.0.to_owned(), branch);
                    }
                    false
                }
                Err(_) => true // Waiting for thread
            }
        });
    }

    if next {
        list_branches(repo, page + 1, branches, settings)?;
    }

    Ok(())
}
