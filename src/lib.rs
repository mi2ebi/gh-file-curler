use reqwest::{blocking::Client, header::AUTHORIZATION};
use std::{error::Error, fs, path::Path};

#[derive(Debug, Clone)]
pub struct GhfcFile {
    pub name: String,
    pub content: Vec<u8>,
}
#[derive(Clone)]
pub struct Files(pub Vec<GhfcFile>);

pub fn fetch(
    user: &str,
    repo: &str,
    paths: Vec<&str>,
    recurse: bool,
    token: &str,
) -> Result<Files, Box<dyn Error>> {
    let client = Client::builder().user_agent("gh-file-curler").build()?;
    let mut out = Files(vec![]);
    for path in paths {
        let url = format!("https://api.github.com/repos/{user}/{repo}/contents/{path}");
        let json = client
            .get(url)
            .header(AUTHORIZATION, format!("Bearer {token}"))
            .send()?
            .json::<serde_json::Value>()?;
        let json = json.as_array().unwrap();
        for file in json {
            if Some("file") == file["type"].as_str() {
                if let Some(name) = file["name"].as_str() {
                    if let Some(file_url) = file["download_url"].as_str() {
                        // println!("{path}/{name}");
                        let content = client.get(file_url).send()?.bytes()?;
                        out.0.push(GhfcFile {
                            name: format!("{path}/{name}"),
                            content: content.to_vec(),
                        });
                    }
                }
            } else if Some("dir") == file["type"].as_str() {
                if recurse {
                    if let Some(name) = file["name"].as_str() {
                        for x in fetch(user, repo, vec![&format!("{path}/{name}")], true, token)
                            .unwrap()
                            .0
                        {
                            out.0.push(x);
                        }
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
            let p = format!("{path}/{}", f.name);
            fs::create_dir_all(Path::new(&p).parent().unwrap()).unwrap();
            fs::write(p, f.content).unwrap();
        }
    }
}
