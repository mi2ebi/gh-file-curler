use reqwest::blocking::Client;
use std::error::Error;

#[derive(Debug)]
pub struct File {
    pub name: String,
    pub content: Vec<u8>,
}
pub struct Files(pub Vec<File>);

pub fn fetch(user: &str, repo: &str, paths: Vec<&str>) -> Result<Files, Box<dyn Error>> {
    let client = Client::builder().user_agent("gh-file-curler").build()?;
    let mut out = Files(vec![]);
    for path in paths {
        let url = format!("https://api.github.com/repos/{user}/{repo}/contents/{path}");
        let json = client.get(url).send()?.json::<serde_json::Value>()?;
        println!("{}", json);
        let json = json.as_array().unwrap();
        for file in json {
            if Some("file") == file["type"].as_str() {
                if let Some(name) = file["name"].as_str() {
                    println!(": {name}");
                    let file_url = file["download_url"].as_str().unwrap();
                    let content = client.get(file_url).send()?.bytes()?;
                    out.0.push(File {
                        name: format!("{path}/{name}"),
                        content: content.to_vec(),
                    });
                }
            } else if Some("dir") == file["type"].as_str() {
                if let Some(name) = file["name"].as_str() {
                    println!("> {name}");
                    for x in fetch(user, repo, vec![&format!("{path}/{name}")])
                        .unwrap()
                        .0
                    {
                        out.0.push(x);
                    }
                    println!("< (up)");
                }
            }
        }
    }
    Ok(out)
}