use std::env;
use std::fs;
use std::path::Path;

use syn::{Expr, File, ImplItem, Item, Lit, Stmt};

fn main() {
    let mcp_path = source_path("crates/mcp/src/adapter.rs");
    let cli_path = source_path("crates/cli/src/commands.rs");
    let config_path = source_path("crates/common/src/config.rs");

    println!("cargo::rerun-if-changed={}", mcp_path);
    println!("cargo::rerun-if-changed={}", cli_path);
    println!("cargo::rerun-if-changed={}", config_path);

    let mcp_methods = extract_mcp_methods(&mcp_path);
    let cli_commands = extract_cli_commands(&cli_path);
    let config_fields = extract_config_fields(&config_path);

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest = Path::new(&out_dir).join("code_inventory.rs");

    let mut out = String::new();
    out.push_str(&format!(
        "pub const MCP_METHODS: &[&str] = &{:?};\n",
        mcp_methods
    ));
    out.push_str(&format!(
        "pub const CLI_COMMANDS: &[&str] = &{:?};\n",
        cli_commands
    ));
    out.push_str(&format!(
        "pub const CONFIG_FIELDS: &[&str] = &{:?};\n",
        config_fields
    ));
    fs::write(&dest, out).unwrap();
}

fn source_path(relative: &str) -> String {
    let repo_root = env::var("CARGO_MANIFEST_DIR").unwrap();
    Path::new(&repo_root)
        .join("..")
        .join("..")
        .join(relative)
        .canonicalize()
        .unwrap_or_else(|_| {
            Path::new(&repo_root)
                .join("..")
                .join("..")
                .join(relative)
        })
        .to_string_lossy()
        .to_string()
}

fn parse_file(path: &str) -> File {
    let content = fs::read_to_string(path).unwrap();
    syn::parse_file(&content).unwrap()
}

fn extract_mcp_methods(path: &str) -> Vec<String> {
    let file = parse_file(path);
    let mut methods = Vec::new();

    for item in &file.items {
        if let Item::Fn(item_fn) = item {
            if item_fn.sig.ident == "handle_request" {
                collect_from_block(&item_fn.block.stmts, &mut methods);
            }
        } else if let Item::Impl(impl_block) = item {
            for impl_item in &impl_block.items {
                if let ImplItem::Fn(method) = impl_item {
                    if method.sig.ident == "handle_request" {
                        collect_from_block(&method.block.stmts, &mut methods);
                    }
                }
            }
        }
    }

    methods.sort();
    methods.dedup();
    methods
}

fn collect_from_block(stmts: &[Stmt], out: &mut Vec<String>) {
    for stmt in stmts {
        if let Stmt::Expr(expr, _) = stmt {
            collect_expr_recursive(expr, out);
        } else if let Stmt::Local(local) = stmt {
            if let Some(init) = &local.init {
                collect_expr_recursive(&init.expr, out);
            }
        }
    }
}

fn collect_expr_recursive(expr: &Expr, out: &mut Vec<String>) {
    match expr {
        Expr::Match(expr_match) => {
            for arm in &expr_match.arms {
                if let syn::Pat::Lit(pat_lit) = &arm.pat {
                    if let Lit::Str(s) = &pat_lit.lit {
                        out.push(s.value());
                    }
                }
            }
        }
        Expr::Block(block) => {
            collect_from_block(&block.block.stmts, out);
        }
        Expr::Call(call) => {
            for arg in &call.args {
                collect_expr_recursive(arg, out);
            }
        }
        Expr::MethodCall(call) => {
            collect_expr_recursive(&call.receiver, out);
            for arg in &call.args {
                collect_expr_recursive(arg, out);
            }
        }
        Expr::If(if_expr) => {
            collect_from_block(&if_expr.then_branch.stmts, out);
            if let Some((_, else_branch)) = &if_expr.else_branch {
                collect_expr_recursive(else_branch, out);
            }
        }
        _ => {}
    }
}

fn extract_cli_commands(path: &str) -> Vec<String> {
    let file = parse_file(path);
    let mut commands = Vec::new();

    for item in &file.items {
        if let Item::Enum(enum_item) = item {
            if enum_item.ident == "Commands" {
                for variant in &enum_item.variants {
                    commands.push(variant.ident.to_string().to_lowercase());
                }
            }
        }
    }

    commands.sort();
    commands
}

fn extract_config_fields(path: &str) -> Vec<String> {
    let file = parse_file(path);
    let mut fields = Vec::new();

    for item in &file.items {
        if let Item::Struct(struct_item) = item {
            if struct_item.ident == "SamgrahaConfig" {
                for field in &struct_item.fields {
                    if let Some(name) = &field.ident {
                        fields.push(name.to_string());
                    }
                }
            }
        }
    }

    fields.sort();
    fields
}
