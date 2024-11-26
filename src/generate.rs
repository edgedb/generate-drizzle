use std::borrow::Cow;
use std::collections::{HashMap, HashSet};
use std::io::Write;
use std::path::Path;

use anyhow::Context as _;
use itertools::Itertools;

use crate::Module;

#[derive(Debug, Default)]
pub(crate) struct Context {
    indent: usize,

    imports: HashMap<String, HashSet<String>>,
}

impl Context {
    pub fn indent(&mut self) {
        self.indent += 1;
    }

    pub fn dedent(&mut self) {
        self.indent -= 1;
    }

    pub fn new_line(&self) -> String {
        "\n".to_string() + &"  ".repeat(self.indent)
    }

    pub fn import<'a, S1: ToString, S2: ToString + Into<Cow<'a, str>>>(
        &mut self,
        module: S1,
        name: S2,
    ) -> Cow<'a, str> {
        let import = self.imports.entry(module.to_string()).or_default();
        import.insert(name.to_string());
        name.into()
    }

    pub fn imports(&self) -> impl Iterator<Item = (&String, &HashSet<String>)> {
        self.imports.iter()
    }
}

pub fn write_files(out_dir: &Path, module: &Module, module_name: &str) -> anyhow::Result<()> {
    let generate_dir = !module.submodules.is_empty();

    let self_file_path = if generate_dir {
        std::fs::create_dir_all(out_dir.join(module_name))
            .context("cannot create schema directory")?;
        out_dir.join(module_name).join("index.ts")
    } else {
        out_dir.join(format!("{module_name}.ts"))
    };

    if !module.tables.is_empty() {
        let generated = generate_file(module);

        let file = std::fs::File::create(&self_file_path).context("cannot write file")?;
        let mut writer = std::io::BufWriter::new(file);
        writer.write_all(generated.as_bytes())?;

        println!("Generated {:?}", self_file_path);
    }

    let sub_dir = out_dir.join(module_name);
    for (sub_name, sub) in &module.submodules {
        write_files(&sub_dir, sub, &sub_name)?;
    }

    Ok(())
}

fn generate_file(module: &Module) -> String {
    let mut ctx = Context::default();

    let pg_schema = module_path_to_pg_schema(&module.path);
    let pg_schema = Some(pg_schema).filter(|x| x != "public");

    let mut body = String::new();
    for table in &module.tables {
        /*
        export const movieTable = pgTable("Movie", {
            id: uuid().primaryKey().generatedAlwaysAs(sql.raw("NULL")),
            title: varchar({ length: 255 }).notNull(),
            release_year: integer(),
        });
        */

        body += "\nexport const ";
        body += &generate_table_var_name(&table.name);
        body += " = ";

        if let Some(pg_schema) = &pg_schema {
            body += &ctx.import("drizzle-orm/pg-core", "pgSchema");
            body += "(\"";
            body += pg_schema;
            body += "\").table(\"";
        } else {
            body += &ctx.import("drizzle-orm/pg-core", "pgTable");
            body += "(\"";
        }

        body += &table.name;
        body += "\", {";

        ctx.indent();

        for column in &table.columns {
            body += &ctx.new_line();

            if column.is_link {
                body += &column.name;
                body += "_id: ";
                body += &ctx.import("drizzle-orm/pg-core", "uuid");
                body += "()";
            } else {
                body += &column.name;
                body += ": ";
                body += &generate_type_ref(&mut ctx, &column.target_name)
                    .unwrap_or_else(|| panic!("unknown type: {}", column.target_name));
            }

            if column.required {
                body += ".notNull()";
            }
            if let Some(def) = &column.default {
                body += ".default(";
                body += &ctx.import("drizzle-orm", "sql");
                body += ".raw(\"";
                body += def;
                body += "\"))";
            }

            if column.is_link {
                let target_name = super::path_last(&column.target_name);

                body += ".references(() => ";

                // TODO: handle references to other modules
                body += &generate_table_var_name(target_name);
                body += ".id)";
            }

            body += ",";
        }

        ctx.dedent();
        body += &ctx.new_line();
        body += "});\n";

        let links: Vec<_> = table.columns.iter().filter(|c| c.is_link).collect();
        if !links.is_empty() {
            body += "\nexport const ";
            body += &camel_case(&table.name);
            body += "Relations = ";

            body += &ctx.import("drizzle-orm", "relations");
            body += "(";
            body += &generate_table_var_name(&table.name);
            body += ", ({ one }) => ({";

            ctx.indent();
            for link in links {
                body += &ctx.new_line();
                body += &link.name;
                body += ": one(";

                let target_name = super::path_last(&link.target_name);
                body += &generate_table_var_name(target_name);
                body += ", {";
                ctx.indent();

                body += &ctx.new_line();
                body += "fields: [";
                body += &generate_table_var_name(&table.name);
                body += ".";
                body += &link.name;
                body += "_id],";

                body += &ctx.new_line();
                body += "references: [";
                body += &generate_table_var_name(target_name);
                body += ".id],";

                ctx.dedent();
                body += &ctx.new_line();

                body += "}),";
            }

            ctx.dedent();
            body += &ctx.new_line();
            body += "}));\n";
        }
    }

    let mut res = String::new();
    for (import, values) in ctx.imports().sorted_by_key(|x| x.0.as_str()) {
        res += "import { ";
        for (index, value) in values.iter().sorted().enumerate() {
            if index > 0 {
                res += ", ";
            }
            res += &value;
        }
        res += " } from '";
        res += import;
        res += "';\n"
    }
    res += "\n";
    res += &body;
    res
}

fn module_path_to_pg_schema(path: &[String]) -> String {
    path.iter()
        .map(|x| {
            if x == "default" {
                Cow::Owned("public".to_owned())
            } else {
                Cow::Borrowed(x)
            }
        })
        .join("::")
}

fn generate_type_ref(ctx: &mut Context, type_name: &str) -> Option<String> {
    let pg_name = match type_name {
        "std::str" => "text",
        "std::int64" => {
            let mut r = ctx.import("drizzle-orm/pg-core", "bigint").to_string();
            r += r#"({ mode: "number"})"#;
            return Some(r);
        }
        "std::int32" => "integer",
        "std::int16" => "smallint",
        "std::decimal" => "numeric",
        "std::bigint" => todo!(), // "edgedbt",
        "std::bool" => "bool",
        "std::float64" => "doublePrecision",
        "std::float32" => "real",
        "std::uuid" => "uuid",
        "std::datetime" => todo!(), // "edgedbt",
        "std::duration" => todo!(), // "edgedbt",
        "std::bytes" => "bytea",
        "std::json" => "jsonb",

        "std::cal::local_datetime" => todo!(), // ('edgedbt', 'timestamp_t'),
        "std::cal::local_date" => todo!(),     // ('edgedbt', 'date_t'),
        "std::cal::local_time" => "time",
        "std::cal::relative_duration" => todo!(), // ('edgedbt', 'relative_duration_t'),
        "std::cal::date_duration" => todo!(),     // ('edgedbt', 'date_duration_t'),

        "cfg::memory" => todo!(), // ('edgedbt', 'memory_t'),

        "std::pg::json" => "json",
        "std::pg::timestamptz" => "timestamptz",
        "std::pg::timestamp" => "timestamp",
        "std::pg::date" => "date",
        "std::pg::interval" => "interval",
        _ => return None,
    };
    Some(ctx.import("drizzle-orm/pg-core", pg_name).to_string() + "()")
}

fn generate_table_var_name(table_name: &str) -> String {
    camel_case(&table_name) + "Table"
}

fn camel_case(s: &str) -> String {
    let mut chunks = s.split(&['.', '_']);

    let first = chunks.next().map(lower_case_first);
    let other = chunks.map(upper_case_first);

    first.into_iter().chain(other).join("")
}

fn lower_case_first(s: &str) -> std::borrow::Cow<str> {
    let mut chars = s.chars();
    if let Some(first) = chars.next() {
        let mut r = first.to_lowercase().to_string();
        r += chars.as_str();
        r.into()
    } else {
        s.into()
    }
}

fn upper_case_first(s: &str) -> std::borrow::Cow<str> {
    let mut chars = s.chars();
    if let Some(first) = chars.next() {
        let mut r = first.to_uppercase().to_string();
        r += chars.as_str();
        r.into()
    } else {
        s.into()
    }
}
