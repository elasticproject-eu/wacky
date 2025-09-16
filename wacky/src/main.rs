use anyhow::{Context, Result};
use clap::Parser;
//use log::debug;
use log::info;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use wacky::{ShimParameters, wac_read, wac_write};
#[derive(Parser)]
struct Cli {
    #[arg(long = "cp", default_value = "untrusted_components.toml")]
    config_path: PathBuf,

    #[arg(long = "wp", default_value = "compose.wac")]
    wac_path: PathBuf,
}

fn main() -> Result<()> {
    env_logger::init();
    info!("Wacky is starting...");
    println!("Wacky is starting...");
    let args = Cli::parse();
    let read_toml = fs::read_to_string(&args.config_path)
        .context("Failed while reading untrusted_components configuration toml")?;

    // Populate HashMap with the untrusted components and shimming parameters
    info!("Reading HashMap from configuration toml provided");
    println!("Loading Access Control configurations provided");
    let untrusted_comps: HashMap<String, ShimParameters> = toml::from_str(&read_toml)
        .context("Failed while loading untrusted_components into HashMap")?;

    // Cleaning up key & value
    let cap = untrusted_comps.len();
    let mut untrusted_map: HashMap<String, ShimParameters> = HashMap::with_capacity(cap);
    for (key, mut param) in untrusted_comps {
        let new_key = key.trim().to_ascii_lowercase();

        param.component_to_shim = param.component_to_shim.trim().to_string();
        param.interface_to_shim = param.interface_to_shim.trim().to_string();
        param.package_shim = param.package_shim.trim().to_string();

        if param.component_to_shim.is_empty()
            || param.interface_to_shim.is_empty()
            || param.package_shim.is_empty()
        {
            anyhow::bail!("One of the parameters is missing!");
        }

        untrusted_map.insert(new_key, param);
    }

    //Populate HashSet with present components in the wac script
    info!("Reading Components taking part in composition from wac composition script...");
    println!("Extracting Components taking part in the composition");
    let mut shimmed = false;
    let components_found: HashSet<String>;
    components_found = wac_read::composed_components(&args.wac_path);
    info!("Checking for the presence of any untrusted components in wac script..");
    println!("Checking for the presence of any untrusted components");
    for (untrusted_component, parameters) in &untrusted_map {
        if components_found.contains(untrusted_component) {
            info!("Found the untrusted component{:?}", untrusted_component);
            println!("Found the untrusted component{:?}", untrusted_component);
            info!(
                "Calling shimmer to inject the {:?}..",
                parameters.package_shim
            );
            println!(
                "Calling shimmer to inject the {:?}..",
                parameters.package_shim
            );
            wac_write::wac_shimmer(&args.wac_path, untrusted_component, parameters);
            shimmed = true;
        }
    }
    if !shimmed {
        println!("No untrusted components found");
    }

    Ok(())
}
