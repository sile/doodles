use orfail::OrFail;

fn main() -> orfail::Result<()> {
    std::fs::create_dir_all("_site/").or_fail()?;
    std::fs::write("_site/index.html", "<h1>Hello, world!</h1>").or_fail()?;
    Ok(())
}
