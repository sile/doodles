use orfail::OrFail;
use std::path::PathBuf;

fn main() -> orfail::Result<()> {
    std::fs::create_dir_all("_site/").or_fail()?;
    std::fs::write("_site/index.html", "<h1>Hello, world!</h1>").or_fail()?;

    let png_files = PngFiles::collect().or_fail()?;
    eprintln!("PNG files: {}", png_files.files.len());

    for src_path in &png_files.files {
        let dst_path =
            PathBuf::from("_site/images/").join(src_path.strip_prefix("src/").or_fail()?);
        std::fs::create_dir_all(dst_path.parent().or_fail()?).or_fail()?;
        std::fs::copy(src_path, dst_path).or_fail()?;
    }

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
}
