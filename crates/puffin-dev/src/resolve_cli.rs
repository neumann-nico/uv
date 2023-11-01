use std::fs;
use std::path::{Path, PathBuf};

use clap::Parser;
use directories::ProjectDirs;
use itertools::Itertools;

use pep508_rs::Requirement;
use platform_host::Platform;
use puffin_client::RegistryClientBuilder;
use puffin_dispatch::BuildDispatch;
use puffin_interpreter::Virtualenv;
use puffin_traits::BuildContext;

#[derive(Parser)]
pub(crate) struct ResolveCliArgs {
    requirements: Vec<Requirement>,
    #[clap(long)]
    limit: Option<usize>,
    /// Path to the cache directory.
    #[arg(global = true, long, env = "PUFFIN_CACHE_DIR")]
    cache_dir: Option<PathBuf>,
}

pub(crate) async fn resolve_cli(args: ResolveCliArgs) -> anyhow::Result<()> {
    let project_dirs = ProjectDirs::from("", "", "puffin");
    let cache = project_dirs.as_ref().map(ProjectDirs::cache_dir);

    let platform = Platform::current()?;
    let venv = Virtualenv::from_env(platform, cache)?;
    let build_dispatch = BuildDispatch::new(
        RegistryClientBuilder::default().cache(cache).build(),
        cache.map(Path::to_path_buf),
        venv.interpreter_info().clone(),
        fs::canonicalize(venv.python_executable())?,
    );

    let mut resolution = build_dispatch.resolve(&args.requirements).await?;
    resolution.sort_unstable_by(|a, b| a.name.cmp(&b.name));
    // Concise format for dev
    println!("{}", resolution.iter().map(ToString::to_string).join(" "));

    Ok(())
}