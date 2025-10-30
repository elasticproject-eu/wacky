use crate::ShimParametersImplicit;
use crate::wac_write::wac_printer;
use log::info;
use miette::SourceSpan;
use std::fs;
use std::path::PathBuf;
use std::string::String;
use wac_parser::{
    Document, Expr, Ident, InstantiationArgument, LetStatement, NewExpr, PackageName, PrimaryExpr,
    Statement,
};

pub fn wac_shimmer_implicit(
    path: &PathBuf,
    untrusted_component: &String,
    parameters: &ShimParametersImplicit,
) {
    let wac_script = fs::read_to_string(path).expect("Error reading WAC file");
    let mut doc = Document::parse(&wac_script).expect("Error parsing wac into document");
    let mut index: usize = 0;
    let mut name = String::new();
    // finding the position of the component to shim and its name
    for (idx, statement) in &mut doc.statements.iter().enumerate() {
        if let Statement::Let(let_stmt) = statement {
            if let PrimaryExpr::New(new_expr) = &let_stmt.expr.primary {
                // change from component to shim to untrusted component when implicit
                if new_expr.package.string == untrusted_component {
                    name = (*(let_stmt.id.string)).to_string();
                    index = idx;
                    break;
                }
            }
        }
    }

    let mut shim_name;

    // Since its primaryExpr hence its new,its this part --->  new app:service
    // Declaration LHS its newexpr, we only need primary
    info!("Preparing the shim to be inserted");
    let new_pkg_shim = NewExpr {
        span: (3..7).into(),
        package: PackageName {
            string: &parameters.package_shim,
            name: &parameters.package_shim,
            version: None,
            span: (3..7).into(),
        },
        arguments: vec![InstantiationArgument::Fill(SourceSpan::new(0.into(), 3))],
    };
    // Now Plug postfix into the Expression
    let shim_expr = Expr {
        span: (0..42).into(),
        primary: PrimaryExpr::New(new_pkg_shim),
        postfix: vec![],
    };

    // This is this part ---> let service_shim
    // Identifier name we give, RHS
    let shim_prefix = String::from("implicitshim");
    let mut counter = 1;
    shim_name = name + &shim_prefix;
    shim_name.push_str(&counter.to_string());
    let shim_instantiation = Statement::Let(LetStatement {
        docs: vec![],
        id: Ident {
            string: &shim_name,
            span: (0..42).into(),
        },
        expr: shim_expr,
    });
    //insert shim before untrusted component
    let shim_position = index.saturating_sub(1);
    info!("Inserting Shim..");
    doc.statements.insert(shim_position, shim_instantiation);

    info!("Done! The Instantiation of shim is completed..");

    // Here we add the implicit {...implicit_shim}
    info!("Changing the provider of the untrusted component to import from the shim..");

    let shim_implicit = shim_name.clone();
    for statement in &mut doc.statements {
        if let Statement::Let(let_stmt) = statement {
            if let PrimaryExpr::New(new_expr) = &mut let_stmt.expr.primary {
                if new_expr.package.string == untrusted_component {
                    for arg in &mut new_expr.arguments {
                        if let InstantiationArgument::Fill(_) = *arg {
                            *arg = InstantiationArgument::Spread(Ident {
                                string: &shim_implicit,
                                span: (3..7).into(),
                            });
                            counter += 1;
                        }
                    }
                }
            }
        }
    }

    wac_printer(&doc, &wac_script, "shimmed_script.wac");
    info!("Shim successfully inserted.");
}
