use orfail::OrFail;
use std::{collections::HashMap, path::PathBuf, time::Duration};

const README_HEADER: &str = r#"Pixcil Doodles
==============

My pixel art doodles drawn with [Pixcil](https://github.com/sile/pixcil).

---
"#;

fn main() -> orfail::Result<()> {
    println!("{README_HEADER}");

    let mut png_files = PngFiles::collect().or_fail()?;
    png_files.sort().or_fail()?;

    for path in png_files.files {
        let name = path
            .strip_prefix("src/")
            .or_fail()?
            .with_extension("")
            .to_str()
            .or_fail()?
            .to_owned();
        println!("[![{name}](https://sile.github.io/doodles/{name}.png)](https://sile.github.io/doodles/{name}.html)");
    }
    println!();

    Ok(())
}

#[derive(Debug)]
pub struct UpdateTimeFinder<'a> {
    head_commit: git2::Commit<'a>,
}

impl<'a> UpdateTimeFinder<'a> {
    fn new(repo: &'a git2::Repository) -> orfail::Result<Self> {
        let head = repo.head().or_fail()?;
        let oid = head.target().or_fail()?;
        let head_commit = repo.find_commit(oid).or_fail()?;
        Ok(Self { head_commit })
    }

    fn find(&self, path: &PathBuf) -> orfail::Result<Duration> {
        let mut commit = self.head_commit.clone();
        let oid = commit.tree().or_fail()?.get_path(path).or_fail()?.id();
        let mut unixtime = Duration::from_secs(commit.time().seconds() as u64);
        while let Some(parent) = commit.parents().next() {
            commit = parent;
            let Ok(entry) = commit.tree().or_fail()?.get_path(path) else {
                break;
            };
            if oid != entry.id() {
                break;
            }
            unixtime = Duration::from_secs(commit.time().seconds() as u64);
        }
        Ok(unixtime)
    }
}

#[derive(Debug)]
pub struct PngFiles {
    pub files: Vec<PathBuf>,
}

impl PngFiles {
    pub fn collect() -> orfail::Result<Self> {
        let mut files = Vec::new();
        let mut stack = vec!["src/".into()];
        while let Some(path) = stack.pop() {
            for entry in std::fs::read_dir(path).or_fail()? {
                let entry = entry.or_fail()?;
                let path = entry.path();

                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().unwrap_or_default() == "png" {
                    files.push(path);
                }
            }
        }
        Ok(Self { files })
    }

    pub fn sort(&mut self) -> orfail::Result<()> {
        let repo = git2::Repository::open(".").or_fail()?;
        let finder = UpdateTimeFinder::new(&repo).or_fail()?;

        let mut updated_times = HashMap::new();
        for path in &self.files {
            updated_times.insert(path.clone(), finder.find(path).or_fail()?);
        }
        self.files.sort_by_key(|path| &updated_times[path]);
        self.files.reverse();
        Ok(())
    }
}
