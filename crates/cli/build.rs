use vergen_git2::{BuildBuilder, CargoBuilder, Emitter, Git2Builder, RustcBuilder};

fn main() -> anyhow::Result<()> {
    let build = BuildBuilder::all_build()?;
    let cargo = CargoBuilder::default().debug(true).features(true).build()?;
    let git2 = Git2Builder::default().sha(false).build()?;
    let rustc = RustcBuilder::default().semver(true).build()?;

    Emitter::default()
        .add_instructions(&build)?
        .add_instructions(&cargo)?
        .add_instructions(&git2)?
        .add_instructions(&rustc)?
        .emit()?;

    Ok(())
}
