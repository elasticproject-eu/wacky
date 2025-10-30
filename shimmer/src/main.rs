use clap::Parser;
use log::info;
use pathdiff::diff_paths;
use shimmer::parser_to_encoder;
use shimmer::{ShimConfig, ShimFn, functions_scaffold, generate_scaffold, names_transformer};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use toml_edit::{DocumentMut, InlineTable, Item, Table};
use wit_encoder;
use wit_encoder::PackageName;
use wit_parser::Resolve;

#[derive(Parser)]
#[command(about = "✨✨Shimmer creates shim scaffold ✨✨")]
struct Cli {
    // component to shim
    #[arg(short, long)]
    component_path: PathBuf,

    // interface to shim
    #[arg(short, long)]
    interface: String,
}
fn main() {
    let args = Cli::parse();
    if !args.component_path.exists() {
        panic!(" The component to shim path was not supplied");
    }
    let mut resolved = Resolve::default();
    let component_path = args.component_path.clone();
    let componentpackage_id = resolved.push_file(component_path).unwrap();
    let componentpackage = &resolved.packages[componentpackage_id];

    let component_nspace = &componentpackage.name.namespace;
    info!("Checking package name :{}", component_nspace);

    let component_name = &componentpackage.name.name;
    let shim_name = component_name.clone() + "shim";
    // running cargo component new add --lib
    // this is the path where the shim component will be made
    let dir_current = env::current_dir().expect("Error getting current directory");
    let root_dir = dir_current.parent().expect("No directory level above");
    let shim_projectname = shim_name.clone();
    let shim_path: PathBuf = root_dir.join(shim_projectname);
    info!("Shim Scaffhold will be created at {:?}", shim_path);

    let command_status = Command::new("cargo")
        .args(&["component", "new"])
        .arg(&shim_path)
        .arg("--lib")
        .status()
        .expect("Error in running cargo component new");

    assert!(command_status.success(), "Creating component failed");

    info!("Checking package name :{}", component_name);

    let mut package_enc =
        wit_encoder::Package::new(PackageName::new("shim", shim_name.to_string(), None));
    let mut world_enc = wit_encoder::World::new(shim_name.to_string());
    let mut shim_interface = None;
    for (interface_name, interface_id) in componentpackage.interfaces.clone() {
        info!("interface name is {}", interface_name);
        if interface_name == args.interface {
            shim_interface = Some((interface_name.clone(), interface_id.clone()));
            break;
        } else {
            panic!("Interface not found!");
        }
    }

    let (_, interface_id) = shim_interface.expect("Interface to shim not found");
    let resolved_clone = resolved.clone();
    let shim_id = &resolved_clone.interfaces[interface_id];
    let mut shiminterface_enc = wit_encoder::Interface::new(args.interface.clone());

    let mut str_param_name: String;
    let mut str_param_type: String;
    let fn_name = String::new();
    let mut final_fn_form: Vec<String> = Vec::new();
    let str_param_return: Option<String> = None;

    let mut shim_configurations = ShimConfig {
        shim_name,
        shim_interface: args.interface.clone(),
        component_to_shim_pkg: format!("{}::{}", component_nspace, component_name),
        fn_name_before_shim: Vec::new(),
    };

    let mut shim_fnconf = ShimFn {
        fn_name,
        param_namntype: Vec::new(),
        str_param_return,
    };
    for (name, function) in shim_id.functions.clone() {
        let mut shim_function = wit_encoder::StandaloneFunc::new(name.clone(), false);
        let mut parameters_shim = wit_encoder::Params::empty();
        shim_configurations.fn_name_before_shim.push(name.clone());
        shim_fnconf.fn_name = name;
        for (param_name, param_type) in &function.params {
            info!("{} {:#?}", param_name, param_type);

            str_param_name = param_name.clone();

            str_param_type = names_transformer(param_type, &resolved);
            shim_fnconf
                .param_namntype
                .push((str_param_name, str_param_type));

            let pars_to_enc = parser_to_encoder(param_type, &resolved);
            parameters_shim.push(param_name.clone(), pars_to_enc);
            shim_function.set_params(parameters_shim.clone());
        }
        if !function.result.is_none() {
            let returns = function.result.as_ref().unwrap();

            let str_param_return_str = names_transformer(returns, &resolved);
            shim_fnconf.str_param_return = Some(str_param_return_str);
            let result_types = parser_to_encoder(returns, &resolved);
            shim_function.set_result(Some(result_types));
        }

        shiminterface_enc.function(shim_function);
        final_fn_form.push(functions_scaffold(&shim_fnconf));
        shim_fnconf.param_namntype.clear();
    }
    package_enc.interface(shiminterface_enc);
    let import_shim_name = format!(
        "{}:{}/{}",
        component_nspace,
        component_name,
        args.interface.clone()
    );
    world_enc.named_interface_import(import_shim_name);
    world_enc.named_interface_export(args.interface);
    package_enc.world(world_enc);
    let final_text = format!("{}", package_enc);
    let shim_wit_path = shim_path.join("wit").join("world.wit");
    fs::write(shim_wit_path, final_text).expect("Error writing back to world.wit");

    let lib_shim_path = shim_path.join("src").join("lib.rs");
    info!("lib path check:{}", lib_shim_path.display());

    let libnfn_content = generate_scaffold(&shim_configurations, final_fn_form);
    fs::write(lib_shim_path, libnfn_content).expect("Writing to Shim lib.rs has failed");

    //Cargo Editing part
    let cargotl_path = shim_path.join("Cargo.toml");
    let cargotl_contents =
        fs::read_to_string(&cargotl_path).expect("Error while reading Cargo.toml contents");

    let mut cargo_doc = cargotl_contents
        .parse::<DocumentMut>()
        .expect("Error parsing the Cargo.toml");

    let import_dep_name = format!("{}:{}", component_nspace, component_name);

    let pkg = cargo_doc["package"].or_insert(Item::Table(Table::new()));
    let pkg = pkg.as_table_mut().unwrap();
    let mta = pkg.entry("metadata").or_insert(Item::Table(Table::new()));
    let mta = mta.as_table_mut().unwrap();
    let cmp = mta.entry("component").or_insert(Item::Table(Table::new()));
    let cmp = cmp.as_table_mut().unwrap();
    let trgt = cmp.entry("target").or_insert(Item::Table(Table::new()));
    let trgt = trgt.as_table_mut().unwrap();
    let dep = trgt
        .entry("dependencies")
        .or_insert(Item::Table(Table::new()));
    let dep = dep.as_table_mut().unwrap();

    let absolute_cmpath = args.component_path.canonicalize().unwrap();
    // computing the path eneded to add in cargo.toml to find where to satisfy the import
    let compc_path_dep = diff_paths(absolute_cmpath, &cargotl_path).unwrap();
    let compc_path_dep = format!("{}", compc_path_dep.display());
    let mut target_dependencies = InlineTable::new();
    target_dependencies.insert("path", compc_path_dep.into());
    dep.insert(&import_dep_name, Item::Value(target_dependencies.into()));
    fs::write(cargotl_path, cargo_doc.to_string())
        .expect("Failed during writing back to Cargo.toml");
}
