use orfail::OrFail;
use pixcil::model::Models as PixcilModel;
use std::path::PathBuf;

pub const IMAGE_HTML_TEMPLATE: &str = include_str!("image.html");
pub const FAVICON: &[u8] = include_bytes!("favicon.png");

fn main() -> orfail::Result<()> {
    std::fs::create_dir_all("_site/").or_fail()?;
    std::fs::write("_site/favicon.png", FAVICON).or_fail()?;

    let png_files = PngFiles::collect().or_fail()?;
    eprintln!("PNG files: {}", png_files.files.len());

    for src_path in &png_files.files {
        let dst_path = PathBuf::from("_site/").join(src_path.strip_prefix("src/").or_fail()?);
        std::fs::create_dir_all(dst_path.parent().or_fail()?).or_fail()?;
        std::fs::copy(src_path, dst_path).or_fail()?;

        let model = PixcilModel::from_png(&std::fs::read(src_path).or_fail()?).or_fail()?;
        generate_thumbnail(&src_path, &model).or_fail()?;
        generate_image_html(&src_path, &model).or_fail()?;
    }

    Ok(())
}

fn generate_thumbnail(src_path: &PathBuf, model: &PixcilModel) -> orfail::Result<()> {
    let dst_path = PathBuf::from("_site/")
        .join(src_path.strip_prefix("src/").or_fail()?)
        .with_extension("thumb.png");

    let size = model.frame_size();
    let min_size = size.width.min(size.height) as f32;
    let scale = (800.0 / min_size).ceil() as usize;
    let thumbnail = model.to_thumbnail_png(scale).or_fail()?;
    std::fs::write(&dst_path, thumbnail).or_fail()?;

    Ok(())
}

fn generate_image_html(src_path: &PathBuf, model: &PixcilModel) -> orfail::Result<()> {
    let name = src_path
        .strip_prefix("src/")
        .or_fail()?
        .with_extension("")
        .to_str()
        .or_fail()?
        .to_owned();
    let size = model.frame_size();

    let mut palette = String::new();
    let mut colors = model.palette().into_iter().collect::<Vec<_>>();
    colors.sort_by_cached_key(|rgb| Hsv::from_rgb(rgb.r, rgb.g, rgb.b).to_sort_key());
    for rgb in colors {
        palette.push_str(&format!(
            r#"<div style="display:inline-block; width:32px; height:32px; background:rgb({},{},{},{})"></div>"#,
            rgb.r, rgb.g, rgb.b, rgb.a as f32 / 255.0
        ));
    }
    let html = IMAGE_HTML_TEMPLATE
        .replace("__NAME__", &name)
        .replace("__SIZE__", &format!("{}x{}", size.width, size.height))
        .replace(
            "__PALETTE__",
            &format!("{} {}", model.palette().len().to_string(), palette),
        )
        .replace(
            "__UPDATED_TIME__",
            &model
                .config
                .attrs
                .updated_time
                .unwrap_or_default()
                .as_secs()
                .to_string(),
        );

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

#[derive(Debug, Clone, Copy, PartialEq)]
struct Hsv {
    h: f64,
    s: f64,
    v: f64,
}

impl Hsv {
    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        let r = r as f64 / 255.0;
        let g = g as f64 / 255.0;
        let b = b as f64 / 255.0;
        let max = r.max(g).max(b);
        let min = r.min(g).min(b);
        let n = max - min;

        let s = if max == 0.0 { 0.0 } else { n / max };
        let v = max;
        let h = if n == 0.0 {
            0.0
        } else if max == r {
            if g < b {
                6.0 + g / n - b / n
            } else {
                (g - b) / n
            }
        } else if max == g {
            2.0 + b / n - r / n
        } else {
            4.0 + r / n - g / n
        } / 6.0;

        Self { h, s, v }
    }

    fn to_sort_key(self) -> (u8, u8) {
        (
            (self.h * 6.0).round() as u8,
            ((self.s * 0.5 + self.v * 0.5) * 255.0).round() as u8,
        )
    }
}
