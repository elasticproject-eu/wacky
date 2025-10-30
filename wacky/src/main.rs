use anyhow::{Context, Result};
use clap::Parser;
use log::info;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;
use wacky::{
    ShimParametersConfig, ShimParametersExplicit, ShimParametersImplicit, wac_read, wac_write,
    wasi_support,
};
#[derive(Parser)]
struct Cli {
    #[arg(long = "cp")]
    config_path: PathBuf,

    #[arg(long = "wp")]
    wac_path: PathBuf,
}

fn main() -> Result<()> {
    env_logger::init();
    info!("Wacky is starting...");
    let args = Cli::parse();
    let read_toml = fs::read_to_string(&args.config_path)
        .context("Failed while reading untrusted_components configuration toml")?;

    // Populate HashMap with the untrusted components and shimming parameters
    info!("Reading HashMap from configuration toml provided");
    // here each untrusted component in header gets its equivalent parameters the body
    let untrusted_comps: HashMap<String, ShimParametersConfig> = toml::from_str(&read_toml)
        .context("Failed while loading untrusted_components into HashMap")?;

    let mut untrusted_map = HashMap::new();
    let mut untrusted_map_implicit = HashMap::new();

    for (key, param) in untrusted_comps {
        let new_key = key.trim().to_ascii_lowercase();
        if let Some(component) = &param.component_to_shim {
            if let Some(interface) = &param.interface_to_shim {
                let param = ShimParametersExplicit {
                    component_to_shim: component.trim().to_string(),
                    interface_to_shim: interface.trim().to_string(),
                    package_shim: param.package_shim.trim().to_string(),
                };
                untrusted_map.insert(new_key, param);
            }
        }
        if param.component_to_shim.is_none() && param.interface_to_shim.is_none() {
            let new_key = key.trim().to_ascii_lowercase();
            let param = ShimParametersImplicit {
                package_shim: param.package_shim.trim().to_string(),
            };

            untrusted_map_implicit.insert(new_key, param);
        }
    }

    //Populate HashSet with present components in the wac script
    info!("Reading Components taking part in composition from wac composition script...");
    let mut shimmed = false;
    let mut explicit = false;
    let components_found: HashSet<String>;
    components_found = wac_read::composed_components(&args.wac_path);
    info!("Checking for the presence of any untrusted components in wac script..");
    for (untrusted_component, parameters) in &untrusted_map {
        if components_found.contains(untrusted_component) {
            info!("Found the untrusted component{:?}", untrusted_component);
            info!(
                "Calling shimmer to inject the {:?}..",
                parameters.package_shim
            );
            wac_write::wac_shimmer(&args.wac_path, untrusted_component, parameters);
            explicit = true;
            shimmed = true;
        }
    }

    for (untrusted_component, parameters) in &untrusted_map_implicit {
        if explicit {
            // if explicit is present pass the new path NOT args
            if components_found.contains(untrusted_component) {
                info!("Found the untrusted component{:?}", untrusted_component);
                info!(
                    "Calling shimmer to inject the {:?}..",
                    parameters.package_shim
                );
                let new_shim_path = PathBuf::from("shimmed_script.wac");
                wasi_support::wac_shimmer_implicit(&new_shim_path, untrusted_component, parameters);
                shimmed = true;
            }
        } else {
            if components_found.contains(untrusted_component) {
                info!("Found the untrusted component{:?}", untrusted_component);
                info!(
                    "Calling shimmer to inject the {:?}..",
                    parameters.package_shim
                );
                wasi_support::wac_shimmer_implicit(&args.wac_path, untrusted_component, parameters);
                shimmed = true;
            }
        }
    }

    if !shimmed {
        info!("No untrusted components found");
    }

    Ok(())
}
