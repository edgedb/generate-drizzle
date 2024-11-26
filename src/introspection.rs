use std::collections::HashMap;

use edgedb_tokio::Queryable;
use itertools::Itertools;

#[derive(Queryable, Debug)]
struct ObjectType {
    name: String,
    ptrs: Vec<Pointer>,
}

#[derive(Queryable, Debug)]
pub struct Pointer {
    pub name: String,
    pub target_name: String,
    pub is_link: bool,
    pub cardinality: Cardinality,
    pub required: bool,
    pub default: Option<String>,
}

#[derive(Queryable, Debug, PartialEq, Eq)]
pub enum Cardinality {
    One,
    Many,
}

pub async fn query_schema() -> anyhow::Result<super::Module> {
    let config = edgedb_tokio::Builder::new()
        .host("localhost")?
        .port(5656)?
        .client_security(edgedb_tokio::ClientSecurity::InsecureDevMode)
        .build_env()
        .await?;
    let client = edgedb_tokio::Client::new(&config);

    let object_types: Vec<ObjectType> = client
        .query(
            r#"
            with ot := schema::ObjectType
            select ot {
                name, ptrs := (
                    select .pointers {
                        name,
                        target_name := .target.name,
                        is_link := ot.pointers is schema::Link,
                        cardinality,
                        required,
                        default,
                    }
                    filter not exists .expr
                )
            }
            filter not .builtin and not .from_alias
            "#,
            &(),
        )
        .await?;

    Ok(partition_into_modules(object_types, vec![]))
}

fn partition_into_modules(object_types: Vec<ObjectType>, path: Vec<String>) -> super::Module {
    let mut submodules = HashMap::<String, Vec<ObjectType>>::new();
    let mut local: Vec<ObjectType> = Vec::new();

    // sift trough object types
    for mut t in object_types {
        match super::path_pop_front(t.name) {
            Ok((first, remaining)) => {
                t.name = remaining;
                let module = submodules.entry(first).or_default();
                module.push(t);
            }
            Err(only) => {
                t.name = only.to_string();
                local.push(t);
            }
        }
    }

    // recurse
    let submodules = submodules
        .into_iter()
        .map(|(name, vals)| {
            let mut path = path.clone();
            path.push(name.clone());
            (name, partition_into_modules(vals, path))
        })
        .collect();

    local.sort_by(|a, b| a.name.cmp(&b.name));
    let tables = local.into_iter().flat_map(object_type_to_table).collect();

    super::Module {
        path,
        tables,
        submodules,
    }
}

fn object_type_to_table(t: ObjectType) -> Vec<super::Table> {
    let mut r = Vec::new();

    let (ptr_table, ptr_column): (Vec<_>, Vec<_>) = t
        .ptrs
        .into_iter()
        .partition(|ptr| ptr.cardinality == Cardinality::Many);

    for ptr in ptr_table {
        r.push(super::Table {
            name: format!("{}.{}", t.name, ptr.name),
            columns: vec![
                Pointer {
                    name: "source".to_string(),
                    target_name: t.name.clone(),
                    is_link: true,
                    cardinality: Cardinality::One,
                    required: true,
                    default: None,
                },
                Pointer {
                    name: "target".to_string(),
                    target_name: ptr.target_name,
                    is_link: ptr.is_link,
                    cardinality: Cardinality::One,
                    required: true,
                    default: None,
                },
                // TODO: link pointers
            ],
        });
    }

    // TODO: link tables for single links with link properties

    r.insert(
        0,
        super::Table {
            name: t.name,
            columns: ptr_column
                .into_iter()
                .filter(|p| p.name != "__type__")
                .sorted_by(pointer_ordering)
                .collect(),
        },
    );

    r
}

fn pointer_ordering(a: &Pointer, b: &Pointer) -> std::cmp::Ordering {
    ptr_priority(a)
        .cmp(&ptr_priority(b))
        .then(a.name.cmp(&b.name))
}

fn ptr_priority(ptr: &Pointer) -> u8 {
    match ptr.name.as_str() {
        "id" => 0,
        "__type__" => 1,
        "source" => 2,
        "target" => 3,
        _ => 4,
    }
}
