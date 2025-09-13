use crate::ShimParameters;
use log::info;
use miette::SourceSpan;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::string::String;
use wac_parser::{
    AccessExpr, Document, DocumentPrinter, Expr, Ident, InstantiationArgument,
    InstantiationArgumentName, LetStatement, NamedInstantiationArgument, NewExpr, PackageName,
    PostfixExpr, PrimaryExpr, Statement, String as WacString,
};

pub fn wac_shimmer(path: &Path, untrusted_component: &String, parameters: &ShimParameters) {
    let wac_script = fs::read_to_string(path).expect("Error reading WAC file");
    let mut doc = Document::parse(&wac_script).expect("Error parsing wac into document");
    let mut index: usize = 0;
    let mut name = String::new();

    //Extracting name and position of component to be shimmed

    for (idx, statement) in &mut doc.statements.iter().enumerate() {
        if let Statement::Let(let_stmt) = statement {
            if let PrimaryExpr::New(new_expr) = &let_stmt.expr.primary {
                if new_expr.package.string == parameters.component_to_shim {
                    name = (*(let_stmt.id.string)).to_string();
                    index = idx;
                    break;
                }
            }
        }
    }

    let component_name = name.clone();
    let mut shim_name;

    // Preparing to add the shim instantiation
    info!("Preparing shim instantiation values");

    //Same interface name used by shimmer
    let key = InstantiationArgumentName::String(WacString {
        value: &parameters.interface_to_shim,
        span: (0..42).into(),
    });

    // The exporter name after the :
    let base = Ident {
        string: &component_name,
        span: (3..7).into(),
    };

    // This is again the interface after the component providing it so exporter.writer <---
    // Span values are chosen arbitrarily, range adjusted only if necesssary
    let accessor = AccessExpr {
        id: Ident {
            string: &parameters.interface_to_shim,
            span: (0..42).into(),
        },
        span: (0..42).into(),
    };

    // Now plugging the expression argument values with above variables
    info!("Now plugging the expression argument values with populated variables..");
    let value_expr = Expr {
        primary: PrimaryExpr::Ident(base),
        postfix: vec![PostfixExpr::Access(accessor)],
        span: (0..42).into(),
    };
    let named_arg = InstantiationArgument::Named(NamedInstantiationArgument {
        name: key,
        expr: value_expr,
    });

    // Since its primaryExpr but its new, this part --->  new app:service
    let new_pkg_shim = NewExpr {
        span: (3..7).into(),
        package: PackageName {
            string: &parameters.package_shim,
            name: &parameters.package_shim,
            version: None,
            span: (3..7).into(),
        },
        arguments: vec![
            named_arg,
            InstantiationArgument::Fill(SourceSpan::new(0.into(), 3)),
        ],
    };

    let shim_expr = Expr {
        span: (0..42).into(),
        primary: PrimaryExpr::New(new_pkg_shim),
        postfix: vec![],
    };

    // This is this part ---> let service_shim
    let shim_prefix = String::from("shim");
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
    let shim_position = index + 1;
    doc.statements.insert(shim_position, shim_instantiation);
    // Done! The Instantiation of shim is completed here
    info!("Done! The Instantiation of shim is completed..");

    // Changing the provider of the untrusted component to import from the shim
    info!("Changing the provider of the untrusted component to import from the shim..");
    for statement in &mut doc.statements {
        if let Statement::Let(let_stmt) = statement {
            if let PrimaryExpr::New(new_expr) = &mut let_stmt.expr.primary {
                if new_expr.package.string == untrusted_component {
                    for arg in &mut new_expr.arguments {
                        if let InstantiationArgument::Named(NamedInstantiationArgument {
                            name,
                            expr,
                        }) = arg
                        {
                            if let InstantiationArgumentName::Ident(id) = name {
                                if id.string != parameters.interface_to_shim {
                                    continue;
                                }
                                if let PrimaryExpr::Ident(shim_base) = &mut expr.primary {
                                    shim_base.string = &shim_name;
                                    counter += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    wac_printer(&doc, &wac_script, "shimed_script.wac");
}

fn wac_printer(doc: &Document, source: &str, path: &str) {
    let mut out = String::new();
    let mut wac_printer = DocumentPrinter::new(&mut out, source, None);
    wac_printer.document(doc).expect("Failed to write to file");

    let mut file = File::create(path).expect("Failed to write");
    file.write_all(out.as_bytes()).expect("Failed");
}
