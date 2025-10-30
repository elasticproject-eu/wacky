use log::info;
use std::collections::HashSet;
use std::fs;
use std::path::Path;
use wac_parser::{Document, PrimaryExpr, Statement};

pub fn composed_components(path: &Path) -> HashSet<String> {
    let wac_script = fs::read_to_string(path).expect("Error reading WAC file");
    let doc = Document::parse(&wac_script).expect("Error parsing WAC into Document");
    let mut components_found: HashSet<String> = HashSet::new();
    info!("Starting the fetch components process...");
    for statement in &doc.statements {
        if let Statement::Let(let_stmt) = statement {
            if let PrimaryExpr::New(new_expr) = &let_stmt.expr.primary {
                components_found.insert(new_expr.package.string.to_string());
            }
        }
    }

    info!("Components Detected:");
    for name in &components_found {
        info!("• {}", name);
    }
    return components_found;
}
