use reqwest::{blocking::Client, header::AUTHORIZATION};
use std::{fs, path::Path};

#[derive(Debug, Clone)]
pub struct GhfcFile {
    pub name: String,
    pub content: Vec<u8>,
}
#[derive(Clone)]
pub struct Files(pub Vec<GhfcFile>);

/// If you want a `paths` entry to be the root, use `""` - might not work properly if you use "/"
///
/// `token`s can be generated in your Github settings
pub fn fetch(
    user: &str,
    repo: &str,
    paths: Vec<&str>,
    recurse: bool,
    token: &str,
) -> Result<Files, String> {
    _fetch(user, repo, None, paths, recurse, token)
}

/// Identical to `fetch()`, except writes files immediately
pub fn speedrun(
    user: &str,
    repo: &str,
    out: &str,
    paths: Vec<&str>,
    recurse: bool,
    token: &str,
) -> Result<Files, String> {
    _fetch(user, repo, Some(out), paths, recurse, token)
}

fn _fetch(
    user: &str,
    repo: &str,
    speedrun: Option<&str>,
    paths: Vec<&str>,
    recurse: bool,
    token: &str,
) -> Result<Files, String> {
    let client = Client::builder()
        .user_agent("gh-file-curler")
        .build()
        .unwrap();
    let mut out = Files(vec![]);
    for path in paths {
        let url = format!("https://api.github.com/repos/{user}/{repo}/contents{path}");
        let json = client
            .get(&url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .send()
            .unwrap()
            .json::<serde_json::Value>()
            .unwrap();
        if json.as_array().is_none() {
            return Err(format!("{json}"));
        }
        let json = json.as_array().unwrap();
        for file in json {
            if Some("file") == file["type"].as_str() {
                if let Some(name) = file["name"].as_str() {
                    if let Some(file_url) = file["download_url"].as_str() {
                        // println!("{path}/{name}");
                        let content = client.get(file_url).send().unwrap().bytes().unwrap();
                        let f = GhfcFile {
                            name: format!("{path}/{name}"),
                            content: content.to_vec(),
                        };
                        out.0.push(f.clone());
                        if let Some(s) = speedrun {
                            f.write_to(s);
                        }                    
                    }
                }
            } else if Some("dir") == file["type"].as_str() && recurse {
                if let Some(name) = file["name"].as_str() {
                    for x in _fetch(
                        user,
                        repo,
                        speedrun,
                        vec![&format!("{path}/{name}")],
                        true,
                        token,
                    )
                    .unwrap()
                    .0
                    {
                        out.0.push(x);
                    }
                }
            }
        }
    }
    Ok(out)
}

impl Files {
    pub fn write_to(self, path: &str) {
        for f in self.0 {
            f.write_to(path);
        }
    }
}
impl GhfcFile {
    fn write_to(self, path: &str) {
        let p = format!("{path}/{}", self.name);
        fs::create_dir_all(Path::new(&p).parent().unwrap()).unwrap();
        fs::write(p, self.content).unwrap();
    }
}
