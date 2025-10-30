use wit_encoder;
use wit_parser::{self, Resolve};

#[derive(Debug)]
pub struct ShimFn {
    pub fn_name: String,
    pub param_namntype: Vec<(String, String)>,
    pub str_param_return: Option<String>,
}

#[derive(Debug)]
pub struct ShimConfig {
    pub shim_name: String,
    pub shim_interface: String,
    pub component_to_shim_pkg: String,
    pub fn_name_before_shim: Vec<String>,
}

const LIB_TEMPLATE: &str = r#"#[allow(warnings)]
mod bindings;

use crate::bindings::exports::shim::{{writershim}}::{{writer}}::Guest;
use bindings::{{docs::writetwo}}::{{writer_interfacetoshim}}::{{{fn_name as fn_name_before_shim}}};

struct Component;

impl Guest for Component {
{{SHIM_LOGIC}}
}

bindings::export!(Component with_types_in bindings);"#;

const SHIM_LOGIC_N: &str = r#"
    fn {{fn_name}}({{parameter}}) {
        {{fn_new_name}}({{str_param_name}})
    }
"#;

const SHIM_LOGIC_RETURN: &str = r#"
    fn {{fn_name}}({{parameter}}) -> {{str_param_return}} {
        {{fn_new_name}}({{str_param_name}})
    }
"#;

pub fn parser_to_encoder(before: &wit_parser::Type, resolver: &Resolve) -> wit_encoder::Type {
    match before {
        wit_parser::Type::U8 => wit_encoder::Type::U8,
        wit_parser::Type::U16 => wit_encoder::Type::U16,
        wit_parser::Type::U32 => wit_encoder::Type::U32,
        wit_parser::Type::U64 => wit_encoder::Type::U64,
        wit_parser::Type::F64 => wit_encoder::Type::F64,
        wit_parser::Type::F32 => wit_encoder::Type::F32,
        wit_parser::Type::S8 => wit_encoder::Type::S8,
        wit_parser::Type::S16 => wit_encoder::Type::S16,
        wit_parser::Type::S32 => wit_encoder::Type::S32,
        wit_parser::Type::S64 => wit_encoder::Type::S64,
        wit_parser::Type::Char => wit_encoder::Type::Char,
        wit_parser::Type::String => wit_encoder::Type::String,
        wit_parser::Type::Bool => wit_encoder::Type::Bool,
        wit_parser::Type::ErrorContext => wit_encoder::Type::ErrorContext,
        wit_parser::Type::Id(id_type) => {
            let def_type = &resolver.types[*id_type];
            let type_kind = &def_type.kind;
            typedef_converter(resolver, type_kind)
        }
    }
}

pub fn typedef_converter(
    resolver: &Resolve,
    type_kind: &wit_parser::TypeDefKind,
) -> wit_encoder::Type {
    match type_kind {
        wit_parser::TypeDefKind::Tuple(tu) => {
            let mut inner: Vec<wit_encoder::Type> = Vec::new();
            for x in tu.types.iter() {
                inner.push(parser_to_encoder(x, resolver));
            }
            wit_encoder::Type::tuple(inner)
        }

        wit_parser::TypeDefKind::List(ty_in) => {
            let inner = parser_to_encoder(ty_in, resolver);
            wit_encoder::Type::List(Box::new(inner))
        }

        wit_parser::TypeDefKind::FixedSizeList(ty_in, size) => {
            let inner = parser_to_encoder(ty_in, resolver);
            wit_encoder::Type::FixedSizeList(Box::new(inner), *size)
        }

        wit_parser::TypeDefKind::Option(ty_in) => {
            let inner = parser_to_encoder(ty_in, resolver);
            wit_encoder::Type::Option(Box::new(inner))
        }

        wit_parser::TypeDefKind::Result(result_ty) => {
            let mut ok_result_typ: Option<wit_encoder::Type> = None;
            if let Some(ok_in) = &result_ty.ok {
                let ok_enc = parser_to_encoder(ok_in, resolver);
                ok_result_typ = Some(ok_enc);
            }

            let mut err_result_typ: Option<wit_encoder::Type> = None;
            if let Some(err_in) = &result_ty.err {
                let err_enc = parser_to_encoder(err_in, resolver);
                err_result_typ = Some(err_enc);
            }
            let result_type;
            if let Some(ok_result_typ) = ok_result_typ {
                if let Some(err_result_typ) = err_result_typ {
                    result_type = wit_encoder::Result_::both(ok_result_typ, err_result_typ);
                } else {
                    result_type = wit_encoder::Result_::ok(ok_result_typ);
                }
            } else if let Some(err_result_typ) = err_result_typ {
                result_type = wit_encoder::Result_::err(err_result_typ);
            } else {
                result_type = wit_encoder::Result_::empty();
            }

            wit_encoder::Type::Result(Box::new(result_type))
        }
        user_defined => {
            eprintln!(
                "Custom Implementation is needed for User Defined type :{:?}, defaulting to string for now.",
                user_defined
            );
            wit_encoder::Type::String
        }
    }
}

pub fn names_transformer(before: &wit_parser::Type, resolver: &Resolve) -> String {
    match before {
        wit_parser::Type::U8 => "u8".to_string(),
        wit_parser::Type::U16 => "u16".to_string(),
        wit_parser::Type::U32 => "u32".to_string(),
        wit_parser::Type::U64 => "u64".to_string(),
        wit_parser::Type::F64 => "f64".to_string(),
        wit_parser::Type::F32 => "f32".to_string(),
        wit_parser::Type::S8 => "i8".to_string(),
        wit_parser::Type::S16 => "i16".to_string(),
        wit_parser::Type::S32 => "i32".to_string(),
        wit_parser::Type::S64 => "i64".to_string(),
        wit_parser::Type::Char => "char".to_string(),
        wit_parser::Type::String => "String".to_string(),
        wit_parser::Type::Bool => "bool".to_string(),
        wit_parser::Type::ErrorContext => "()".into(),

        wit_parser::Type::Id(id_type) => {
            let def_type = &resolver.types[*id_type];
            let type_kind = &def_type.kind;

            match type_kind {
                wit_parser::TypeDefKind::Tuple(tu) => {
                    let mut inner = Vec::new();
                    for x in tu.types.iter() {
                        inner.push(names_transformer(x, resolver));
                    }
                    format!("({})", inner.join(","))
                }

                wit_parser::TypeDefKind::List(ty_in) => {
                    let inner = names_transformer(ty_in, resolver);
                    format!("Vec<{}>", inner)
                }

                wit_parser::TypeDefKind::Option(ty_in) => {
                    let inner = names_transformer(ty_in, resolver);
                    format!("Option<{}>", inner)
                }

                wit_parser::TypeDefKind::Result(result_typ) => {
                    let ok_str = if let Some(ok) = &result_typ.ok {
                        names_transformer(ok, resolver)
                    } else {
                        "()".to_string()
                    };

                    let err_str = if let Some(err) = &result_typ.err {
                        names_transformer(err, resolver)
                    } else {
                        "()".to_string()
                    };

                    format!("Result<{}, {}>", ok_str, err_str)
                }
                user_defined => {
                    eprintln!(
                        "Custom Implementation is needed for User Defined type :{:?}, defaulting to string for now.",
                        user_defined
                    );
                    "User_Defined".to_string()
                }
            }
        }
    }
}

pub fn functions_scaffold(fnconfig: &ShimFn) -> String {
    let param_pair = fnconfig
        .param_namntype
        .iter()
        .map(|(par_name, par_type)| format!("{}: {}", par_name, par_type))
        .collect::<Vec<_>>()
        .join(",");

    let param_names = fnconfig
        .param_namntype
        .iter()
        .map(|(par_name, _)| format!("{}", par_name))
        .collect::<Vec<_>>()
        .join(",");
    let fn_new_name = fnconfig.fn_name.clone() + "_before_shim";
    let shim_fn_scaff = match &fnconfig.str_param_return {
        Some(str_return) => SHIM_LOGIC_RETURN
            .replace("{{fn_name}}", &fnconfig.fn_name)
            .replace("{{parameter}}", &param_pair)
            .replace("{{str_param_name}}", &param_names)
            .replace("{{str_param_return}}", &str_return)
            .replace("{{fn_new_name}}", &fn_new_name),
        None => SHIM_LOGIC_N
            .replace("{{fn_name}}", &fnconfig.fn_name)
            .replace("{{parameter}}", &param_pair)
            .replace("{{fn_new_name}}", &fn_new_name),
    };
    shim_fn_scaff
}

pub fn generate_scaffold(sconfig: &ShimConfig, shim_logic_sc: Vec<String>) -> String {
    let shim_aggregated_logic = shim_logic_sc.join("\n \n");
    let before_fn_name = sconfig
        .fn_name_before_shim
        .iter()
        .map(|old_name| format!("{} as {}_before_shim", old_name, old_name))
        .collect::<Vec<_>>()
        .join("  ,");

    let lib_content = LIB_TEMPLATE
        .replace("{{writershim}}", &sconfig.shim_name)
        .replace("{{writer}}", &sconfig.shim_interface)
        .replace("{{docs::writetwo}}", &sconfig.component_to_shim_pkg)
        .replace("{{writer_interfacetoshim}}", &sconfig.shim_interface)
        .replace("{{fn_name as fn_name_before_shim}}", &before_fn_name)
        .replace("{{SHIM_LOGIC}}", &shim_aggregated_logic);

    lib_content
}
