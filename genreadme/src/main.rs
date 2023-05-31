use orfail::OrFail;
use pixcil::model::Models as PixcilModels;
use std::{collections::HashMap, path::PathBuf};

const README_HEADER: &str = r#"Pixcil Doodles
==============

My pixel art doodles drawn by using [Pixcil](https://github.com/sile/pixcil).

---
"#;

fn main() -> orfail::Result<()> {
    println!("{README_HEADER}");

    let png_files = PngFiles::collect().or_fail()?;
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
        let mut updated_times = HashMap::new();
        for path in &self.files {
            let data = std::fs::read(path).or_fail()?;
            let model = PixcilModels::from_png(&data).or_fail()?;
            updated_times.insert(path.clone(), model.attrs.updated_time.unwrap_or_default());
        }
        self.files.sort_by_key(|path| &updated_times[path]);
        self.files.reverse();
        Ok(())
    }
}
