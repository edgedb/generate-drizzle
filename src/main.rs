mod generate;
mod introspection;

use std::collections::HashMap;

#[derive(Debug)]
struct Module {
    path: Vec<String>,
    tables: Vec<Table>,
    submodules: HashMap<String, Module>,
}

#[derive(Debug)]
struct Table {
    name: String,
    columns: Vec<introspection::Pointer>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut args = std::env::args();
    args.next();
    let out_dir = args.next().unwrap();
    let out_dir = std::path::Path::new(&out_dir);

    let module = introspection::query_schema().await?;

    generate::write_files(out_dir, &module, "schema")?;

    Ok(())
}

#[allow(dead_code)]
fn path_lookup(mut path: &str, index: usize) -> Option<(&str, bool)> {
    for _ in 0..index {
        let position = path.find("::")?;
        path = &path[position + 2..];
    }

    let (position, is_last) = path
        .find("::")
        .map(|p| (p, false))
        .unwrap_or_else(|| (path.len(), true));
    Some((&path[..position], is_last))
}

fn path_pop_front(path: String) -> Result<(String, String), String> {
    if let Some(position) = path.find("::") {
        let mut module = path;
        let remaining = module.split_off(position + 2);
        module.truncate(position);
        Ok((module, remaining))
    } else {
        Err(path)
    }
}

fn path_last(path: &str) -> &str {
    let position = path.rfind("::").map(|p| p + 2).unwrap_or(0);
    &path[position..]
}
