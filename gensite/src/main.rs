use orfail::OrFail;
use pixcil::model::Models as PixcilModel;
use pixcil::pixel::PixelSize;
use std::path::PathBuf;
use std::time::Duration;

pub const IMAGE_HTML_TEMPLATE: &str = include_str!("image.html");
pub const FAVICON: &[u8] = include_bytes!("favicon.png");

fn main() -> orfail::Result<()> {
    std::fs::create_dir_all("_site/").or_fail()?;
    std::fs::write("_site/favicon.png", FAVICON).or_fail()?;

    let png_files = PngFiles::collect().or_fail()?;
    eprintln!("PNG files: {}", png_files.files.len());

    let repo = git2::Repository::open(".").or_fail()?;
    let finder = UpdateTimeFinder::new(&repo).or_fail()?;

    for src_path in &png_files.files {
        let dst_path = PathBuf::from("_site/").join(src_path.strip_prefix("src/").or_fail()?);
        std::fs::create_dir_all(dst_path.parent().or_fail()?).or_fail()?;
        std::fs::copy(src_path, dst_path).or_fail()?;

        let model = PixcilModel::from_png(&std::fs::read(src_path).or_fail()?).or_fail()?;
        generate_thumbnail(&src_path, &model).or_fail()?;
        generate_image_html(&src_path, &model, &finder).or_fail()?;
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
    let thumbnail = to_thumbnail_png(&model, scale).or_fail()?;
    std::fs::write(&dst_path, thumbnail).or_fail()?;

    Ok(())
}

fn generate_image_html(
    src_path: &PathBuf,
    model: &PixcilModel,
    finder: &UpdateTimeFinder,
) -> orfail::Result<()> {
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
    if let Some(bg_color) = model.config.background_color {
        colors.push(bg_color);
    }
    colors.sort_by_cached_key(|rgb| Hsv::from_rgb(rgb.r, rgb.g, rgb.b).to_sort_key());
    for rgb in colors {
        palette.push_str(&format!(
            r#"<div style="display:inline-block; width:32px; height:32px; background:rgb({},{},{},{})"></div>"#,
            rgb.r, rgb.g, rgb.b, rgb.a as f32 / 255.0
        ));
    }

    let update_date = time::Date::from_calendar_date(1970, time::Month::January, 1).or_fail()?
        + finder.find(src_path).or_fail()?;
    let update_date = update_date
        .format(&time::format_description::parse("[year]-[month]-[day]").or_fail()?)
        .or_fail()?;
    let padding_top = if size.width > size.height {
        format!("{:.2}%", size.height as f32 / size.width as f32 * 100.0)
    } else {
        "100%".to_owned()
    };

    let html = IMAGE_HTML_TEMPLATE
        .replace("__NAME__", &name)
        .replace("__SIZE__", &format!("{}x{}", size.width, size.height))
        .replace("__PALETTE__", &palette)
        .replace("__UPDATE_DATE__", &update_date)
        .replace("__PADDING_TOP__", &padding_top);

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

fn to_thumbnail_png(model: &PixcilModel, scale: usize) -> orfail::Result<Vec<u8>> {
    (scale > 0).or_fail()?;
    let bg_color = model.config.background_color;
    let frame_count = model.config.animation.enabled_frame_count();
    let size = model.frame_size();
    let image_size = PixelSize::from_wh(size.width * scale as u16, size.height * scale as u16);

    let mut png_data = Vec::new();
    {
        let mut encoder = png::Encoder::new(
            &mut png_data,
            u32::from(image_size.width),
            u32::from(image_size.height),
        );
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);

        if frame_count > 1 {
            encoder.set_animated(frame_count as u32, 0).or_fail()?;
            encoder
                .set_frame_delay(1, model.config.animation.fps() as u16)
                .or_fail()?;
        }

        let mut writer = encoder.write_header().or_fail()?;

        for frame in 0..frame_count {
            let mut image_data =
                vec![0; image_size.width as usize * image_size.height as usize * 4];
            for (i, position) in model
                .config
                .frame
                .get_preview_region(&model.config, frame as usize)
                .pixels()
                .enumerate()
            {
                let color = model
                    .pixel_canvas
                    .get_pixel(&model.config, position)
                    .or(bg_color);
                let Some(color) = color else {
                    continue;
                };

                let y_base = i / size.width as usize * scale;
                let x_base = i % size.width as usize * scale;
                for y_delta in 0..scale {
                    for x_delta in 0..scale {
                        let i = (y_base + y_delta) * image_size.width as usize * 4
                            + (x_base + x_delta) * 4;
                        image_data[i] = color.r;
                        image_data[i + 1] = color.g;
                        image_data[i + 2] = color.b;
                        image_data[i + 3] = color.a;
                    }
                }
            }
            writer.write_image_data(&image_data).or_fail()?;
        }
    }
    Ok(png_data)
}
