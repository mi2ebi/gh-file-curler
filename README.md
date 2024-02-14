## gh-file-curler

Grabs only the files from a Github repo, without the Git history

## Usage

```rs
use gh_file_curler::fetch;
use std::fs;

fn main() {
    fs::remove_dir_all("out").unwrap_or(());
    fs::create_dir("out").unwrap();
    let the = fetch("berrymot", "gh-file-curler", vec![""], true, "TOKEN")
        .unwrap();
    the.clone().write_to("out");
    println!("{} files", the.0.len());
}
```