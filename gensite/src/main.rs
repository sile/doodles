use orfail::OrFail;
use std::path::PathBuf;

pub const IMAGE_HTML_TEMPLATE: &str = include_str!("image.html");

fn main() -> orfail::Result<()> {
    std::fs::create_dir_all("_site/").or_fail()?;

    let png_files = PngFiles::collect().or_fail()?;
    eprintln!("PNG files: {}", png_files.files.len());

    for src_path in &png_files.files {
        let dst_path = PathBuf::from("_site/").join(src_path.strip_prefix("src/").or_fail()?);
        std::fs::create_dir_all(dst_path.parent().or_fail()?).or_fail()?;
        std::fs::copy(src_path, dst_path).or_fail()?;

        generate_thumbnail(&src_path).or_fail()?;
        generate_image_html(&src_path).or_fail()?;
    }

    Ok(())
}

fn generate_thumbnail(src_path: &PathBuf) -> orfail::Result<()> {
    let dst_path = PathBuf::from("_site/")
        .join(src_path.strip_prefix("src/").or_fail()?)
        .with_extension("thumb.png");

    // TODO: resize if need
    std::fs::copy(src_path, dst_path).or_fail()?;

    Ok(())
}

fn generate_image_html(src_path: &PathBuf) -> orfail::Result<()> {
    let name = src_path
        .strip_prefix("src/")
        .or_fail()?
        .with_extension("")
        .to_str()
        .or_fail()?
        .to_owned();
    let html = IMAGE_HTML_TEMPLATE.replace("__NAME__", &name);

    let dst_path = PathBuf::from("_site/")
        .join(src_path.strip_prefix("src/").or_fail()?)
        .with_extension("html");
    std::fs::write(dst_path, html).or_fail()?;

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
