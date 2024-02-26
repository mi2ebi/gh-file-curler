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
pub fn fetch_dir(
    user: &str,
    repo: &str,
    paths: Vec<&str>,
    recurse: bool,
    token: &str,
) -> Result<Files, String> {
    _fetch_dir(user, repo, None, paths, recurse, token)
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
    _fetch_dir(user, repo, Some(out), paths, recurse, token)
}

fn _fetch_dir(
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
                    if file["download_url"].as_str().is_some() {
                        // println!("{path}/{name}");
                        let f = fetch(user, repo, vec![&format!("{path}/{name}")])
                            .unwrap()
                            .0[0]
                            .clone();
                        out.0.push(f.clone());
                        if let Some(s) = speedrun {
                            f.write_to(s);
                        }
                    }
                }
            } else if Some("dir") == file["type"].as_str() && recurse {
                if let Some(name) = file["name"].as_str() {
                    for x in _fetch_dir(
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

pub fn fetch(user: &str, repo: &str, files: Vec<&str>) -> Result<Files, String> {
    let client = Client::builder()
        .user_agent("gh-file-curler")
        .build()
        .unwrap();
    let mut out = Files(vec![]);
    for file in files {
        let url = format!("https://raw.githubusercontent.com/{user}/{repo}/main/{file}");
        let mut content = client.get(&url).send().unwrap().bytes();
        let mut i = 0;
        while content.is_err() && i < 3 {
            content = client.get(&url).send().unwrap().bytes();
            i += 1;
        }
        if content.is_err() {
            return Err(format!(
                "multiple requests to {url} failed (e.g. timed out)"
            ));
        }
        let content = content.unwrap();
        let f = GhfcFile {
            name: file.to_string(),
            content: content.to_vec(),
        };
        out.0.push(f);
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
/// Most useful on a `fetch()` call for one file
pub fn wrapped_first(f: Result<Files, String>) -> Option<Vec<u8>> {
    if let Ok(f) = f {
        let x = f.0[0].clone().content;
        if x != b"404: Not Found" {
            Some(x)
        } else {
            None
        }
    } else {
        None
    }
}
impl GhfcFile {
    pub fn write_to(self, path: &str) {
        let p = format!("{path}/{}", self.name);
        fs::create_dir_all(Path::new(&p).parent().unwrap()).unwrap();
        fs::write(p, self.content).unwrap();
    }
}
