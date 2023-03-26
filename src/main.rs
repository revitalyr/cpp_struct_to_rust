#![feature(try_blocks)]

use std::{  collections::{HashMap, HashSet},
            path,
            fmt::{Debug, Formatter},
            fs::{self, File},
            io::Write,
            path::PathBuf};

//use std::process::exit;
use clang::{Clang, Entity, EntityKind, Index, TypeKind};
use clang::source::Location;

use dunce;
use regex::Regex;
use clap::Parser;

#[derive(Parser)]
#[clap(ignore_errors=true)]
struct Cli {
    /// Path to source
    cpp_path: String,
    /// Path to result
    #[arg(short, long, value_name = "RESULT RUST FILE")]
    rs_path: Option<String>,
    /// Clang args
    #[arg(last = true, value_name = "CLANG's ARGUMENTS")]
    clang_args: Vec<String>,
}

#[derive(Debug)]
struct Field {
    name: String,
    type_: String,
}

#[derive(Debug)]
struct StructDef {
    name: String,
    members: Vec<Field>,
}

impl StructDef {
    fn new(name: &str) -> Self {
        StructDef {
            name: name.to_string(),
            members: vec![],
        }
    }

    fn add_field(&mut self, fld_name: &str, fld_type: &str) {
        self.members.push(Field {
            name: fld_name.to_string(),
            type_: fld_type.to_string(),
        })
    }

    fn get_used_types(&self) -> NamesSet {
        let mut result = NamesSet::with_capacity(self.members.len());
        for fld in &self.members {
            result.insert(fld.type_.clone());
        }

        result
    }

    fn get_rust_code(&self) -> String {
        let mut code = format!("\n#[derive(Debug, Clone)]\n#[repr(C)]\npub struct {} {{\n", self.name);
        self.members
            .iter()
            .for_each(|fld| {
                code += format!("  pub {}: {},\n", fld.name, fld.type_).as_str();
            });
        code += "}\n";
        code
    }
}

#[derive(Debug)]
struct TypeDef {
    name: String,
    _kind: TypeKind,
    def: String,
}

type NamesSet = HashSet<String>;
type Dictionary<T> = HashMap<String, T>;
type StructDefDict = Dictionary<StructDef>;
type TypeDefDict = Dictionary<TypeDef>;
type EnumDefDict = Dictionary<Vec<String>>;

#[derive(Debug, PartialEq)]
enum KnownType {
    None,
    Struct,
    TypeDef,
    Enum,
}

struct KnownTypes {
    structs: StructDefDict,
    typedefs: TypeDefDict,
    enums: EnumDefDict,
}

impl KnownTypes {
    fn new() -> Self {
        KnownTypes {
            structs: StructDefDict::new(),
            typedefs: TypeDefDict::new(),
            enums: EnumDefDict::new(),
        }
    }

    fn is_know_type(&self, name: &str) -> KnownType {
        if self.structs.contains_key(name) {
            return KnownType::Struct;
        }
        if self.typedefs.contains_key(name) {
            return KnownType::TypeDef;
        }
        if self.enums.contains_key(name) {
            return KnownType::Enum;
        }

        KnownType::None
    }

    fn add_struct(&mut self, str_def: StructDef) {
        self.structs.insert(str_def.name.clone(), str_def);
    }

    fn add_typedef(&mut self, td: TypeDef) {
        self.typedefs.insert(td.name.clone(), td);
    }

    fn add_enum(&mut self, name: &str, values: Vec<String>) {
        self.enums.insert(name.to_string(), values);
    }
}

impl Debug for KnownTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "structs: {:?}\ntypedefs: {:?}\nenums: {:?}", self.structs, self.typedefs, self.enums)
    }
}

struct Converter<'tu> {
    known_types: KnownTypes,
    location: Location<'tu>,
}

impl<'tu> Converter<'tu> {
    fn new() -> Self {
        Converter {
            known_types: KnownTypes::new(),
            location: Location {
                file: None,
                line: 0,
                column: 0,
                offset: 0,
            },
        }
    }

    fn set_location(&mut self, entity: &Entity<'tu>) {
        self.location = entity.get_location().unwrap().get_spelling_location();
    }

    fn try_c_to_rust_type(&self, c_type: &str) -> Option<String> {
        let text = match c_type {
                                        "int" => "c_int",
                                        "bool" => "bool",
                                        "char" => "c_char",
                                        "const char *" => "*const c_char",
                                        "unsigned short" => "c_ushort",
                                        "unsigned int" => "c_uint",
                                        "size_t" => "usize",
                                        "int *" => "Vec<c_int>",
                                        "char *" => "*const c_char",
                                        "char **" => "*const *const c_char",
                                        "void *" => "*const c_void",
                                        "uintptr_t" => "*const c_void",
                                        _ => "",
                                    };
        if !text.is_empty() {
            return Some(text.to_string());
        }

        match self.known_types.is_know_type(c_type) {
            KnownType::Struct =>
                return Some(c_type.to_string()),
            KnownType::TypeDef =>
                return Some(c_type.to_string()),
            KnownType::Enum =>
                return Some(c_type.to_string()),
            KnownType::None =>
                return None,
        }
    }

    fn c_to_rust_type(&mut self, c_type: &str) -> String {
        if let Some(s) = self.try_c_to_rust_type(c_type) {
            return s;
        }

        lazy_static::lazy_static! {
            static ref RE_STRUCT_REF: Regex = Regex::new(r"struct ([^ ]+) *").unwrap();
        }

        if let Some(cap) = RE_STRUCT_REF.captures(c_type) {
            let struct_name = cap.get(1).unwrap().as_str().trim();
            return format!("*const {struct_name}");
        }

        lazy_static::lazy_static! {
            static ref RE_ARRAY: Regex = Regex::new(r"([^\[]+)\[([^\]]*)\]").unwrap();
        }

        if let Some(cap) = RE_ARRAY.captures(c_type) {
            let arr_type = cap.get(1).unwrap().as_str().trim();
            return format!("[{}; {}]", self.c_to_rust_type(arr_type), cap.get(2).unwrap().as_str());
        }

        eprintln!("\x1B[31mmissed '{c_type}' line: {}, column: {} in {}\x1b[0m",
                  self.location.line, self.location.column, self.location.file.unwrap().get_path().display());
        format!("!!!{c_type}!!!")
    }

    fn add_struct(&mut self, str_def: StructDef) {
        self.known_types.add_struct(str_def);
    }

    fn add_typedef(&mut self, td: TypeDef) {
        self.known_types.add_typedef(td);
    }

    fn add_enum(&mut self, name: &str, values: Vec<String>) {
        self.known_types.add_enum(name, values);
    }
}

fn get_name(ent: &Entity) -> String {
    if let Some(ref name) = ent.get_display_name() {
        name.clone()
    } else {
        "NONE".to_string()
    }
}

fn get_type(ent: &Entity) -> String {
    if let Some(ref type_) = ent.get_type() {
        type_.get_display_name().clone()
    } else {
        "NONE".to_string()
    }
}

fn main() {
    let cli = Cli::parse();

    let cpp_file = path::Path::new(&cli.cpp_path);
    println!("cpp_file: '{}'", cpp_file.display());
    let clang = Clang::new().unwrap();
    let index = Index::new(&clang, false, true);
    //let arguments = if let Some(args) = cli.clang_args{ args } else { Vec::new() };
    let arguments = cli.clang_args;
    let mut parser = index.parser(cpp_file);
    //let parser = parser.arguments(&[arguments.trim_matches(|c| c == '"' || c == '\'')]);
    let parser = parser.arguments(&arguments);
    let tu = match parser.parse() {
        Ok(tu) => tu,
        Err(e) => panic!("Parse error: {e}"),
    };

    let entity = tu.get_entity();
    //println!("{entity:?}");

    let mut converter = Converter::new();

    for child in entity.get_children() {
        match child.get_kind() {
            EntityKind::StructDecl => {
                if child.get_children().len() > 0 {
                    converter.set_location(&child);
                    let name = get_name(&child);
                    let mut str_def = StructDef::new(&name);

                    //println!("StructDecl: {name}");
                    for field in child.get_children() {
                        converter.set_location(&field);
                        let fld_name = get_name(&field);
                        let fld_type = get_type(&field);

                        //println!("{fld_type} {fld_name}");
                        str_def.add_field(&fld_name, &converter.c_to_rust_type(&fld_type));
                    }
                    converter.add_struct(str_def);
                }
            }
            EntityKind::TypedefDecl => {
                let name = get_name(&child);
                let td_type = &child.get_typedef_underlying_type().unwrap();
                if converter.known_types.is_know_type(&name) == KnownType::None {
                    let mut def_text = td_type.get_display_name().replace("struct ", "");

                    //println!("TypedefDecl: {name}");
                    if (name != def_text) && !def_text.starts_with("enum") {
                        if def_text.ends_with("*") {
                            def_text = format!("*const {}", def_text.trim_end_matches(|c| c=='*'));
                        }
                        converter.add_typedef(TypeDef{name: name, _kind: td_type.get_kind(), def: def_text });
                    }
                }
            }
            EntityKind::EnumDecl => {
                let name = get_name(&child);
                let mut values = Vec::<String>::with_capacity(child.get_children().len());

                //println!("EnumDecl: {name}");
                for val in child.get_children() {
                    values.push(get_name(&val));
                }
                converter.add_enum(&name, values);
            }
            _ => {
                //println!("  {child:?}");
            }
        }
    }

    let mut outfile = if let Some(rs_path) = cli.rs_path {
        PathBuf::from(&rs_path)
    } else {
        PathBuf::from(cpp_file.file_name().unwrap().to_str().unwrap())
    };
    if outfile.extension().unwrap().to_str() == Some("cpp") {
        outfile.set_extension("rs");
    };
    if outfile.exists() {
        let mut backup = PathBuf::from(&outfile);
        backup.set_extension("bak");
        fs::rename(&outfile, &backup).unwrap();
    }

    let mut output = File::create(&outfile).unwrap();
    println!("rust_file: '{}'", dunce::simplified(outfile.canonicalize().unwrap().as_path()).display());

    //println!("\nknown_types:{:?}", converter.known_types);
    let header =
        r"
        #![allow(dead_code, non_snake_case, non_camel_case_types)]
        use std::ffi::{c_char, c_int, c_uint, c_ushort, c_void};

        fn main () {
        }

        ";

    let mut rust_code = "".to_string();

    for line in header.lines() {
        let s = line.trim_start();
        rust_code += s;
        rust_code += "\n";
    }

    let mut used_types = NamesSet::new();

    for (_, str_def) in &converter.known_types.structs {
        let def = str_def.get_rust_code();
        rust_code += &def;
        used_types.extend(str_def.get_used_types());
    }

    for (name, values) in &converter.known_types.enums {
        if used_types.contains(name) {
            rust_code += &format!("#[derive(Debug, PartialEq, Clone)]\npub enum {name} {{\n    {}\n}}\n", values.join(",\n    "));
        }
    }

    let mut refered_types = NamesSet::new();
    for (name, def) in &converter.known_types.typedefs {
        if used_types.contains(name) {
            let def_text = if let Some(text) = def.def.strip_prefix("*const ") { text.to_string() } else { def.def.clone() };
            let fld_type = if let Some(rust_type) = converter.try_c_to_rust_type(&def_text) {
                rust_type
            } else {
                format!("!!!{def_text}!!!")
            };
            rust_code += &format!("type {} = {};\n", name, fld_type);
            refered_types.insert(def_text.to_string());
        }
    }

    write!(output, "{rust_code}").unwrap();
}
