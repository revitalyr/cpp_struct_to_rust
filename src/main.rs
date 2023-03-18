#![feature(try_blocks)]
use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::Write;
//use std::process::exit;
use clang::{Clang, Index, EntityKind, Entity};
use clang::source::Location;
//use clib;
use regex::Regex;

type Dictionary = HashSet<String>;

struct KnownTypes {
    structs: Dictionary,
}

impl KnownTypes {
    fn new() -> Self {
        KnownTypes {
            structs: Dictionary::new(),
        }
    }

    fn add_struct(&mut self, name: &str) {
        self.structs.insert(name.to_string());
    }
}

struct Field {
    name: String,
    type_ : String,
}

struct StructGen {
    name: String,
    members: Vec<Field>,
}

impl StructGen {
    fn new(name: &str) -> Self {
        StructGen {
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

    fn get_rust_code(&self) -> String {
        let mut code = format!("#[derive(Debug, Copy, Clone)]\n#[repr(C)]\nstruct {} {{\n", self.name);
        self.members
            .iter()
            .for_each(|fld| {
                code += format!("  {}: {},\n", fld.name, fld.type_).as_str();
            } );
        code += "}\n\n";
        code
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

    fn try_c_to_rust_type(&mut self, c_type: &str) -> Option<String> {
        lazy_static::lazy_static! {
        static ref RE_ARRAY: Regex = Regex::new(r"([^\[]+)\[([^\]]*)\]").unwrap();
    }

        if let Some(cap) = RE_ARRAY.captures(c_type) {
            let arr_type = cap.get(1).unwrap().as_str().trim();
            return Some(format!("[{}; {}]", self.c_to_rust_type(arr_type), cap.get(2).unwrap().as_str()));
        }

        Some(match c_type {
            "int" => "c_int",
            "bool" => "bool",
            "char" => "c_char",
            "const char *" => "CStr",
            "unsigned int" => "c_uint",
            _ => return None,
        }.to_string())
    }

    fn c_to_rust_type(&mut self, c_type: &str) -> String {
        if let Some(s) = self.try_c_to_rust_type(c_type) {
            return s;
        }
        eprintln!("\x1B[31mmissed '{c_type}' line: {}, column: {}\x1b[0m", self.location.line, self.location.column);
        format!("!!!{c_type}!!!")
    }

    fn add_struct(&mut self, name: &str) {
        self.known_types.add_struct(name);
    }
}

fn get_name(ent: &Entity) -> String {
    ent.get_display_name().unwrap().clone()
}

fn get_type(ent: &Entity) -> String {
    ent.get_type().unwrap().get_display_name().clone()
}

fn main() {
    let cpp_file = env::args().skip(1).next().unwrap().as_str().to_string();
    println!("cpp_file: '{cpp_file}'");
    let clang = Clang::new().unwrap();
    let index = Index::new(&clang, false, false);
    let arguments = vec!["-I", "dummy"];
    let mut parser = index.parser(cpp_file);
    let parser = parser.arguments(&arguments);
    let tu = match parser.parse() {
        Ok(tu) => tu,
        Err(e) => panic!("Parse error: {e}"),
    };

    let entity = tu.get_entity();
    println!("{entity:?}");

    let mut rust_code =
r"
#![allow(dead_code, non_snake_case)]
use std::ffi::{CStr, c_char, c_int};

fn main () {
}

".to_string();

    let mut output = File::create("./misc/src/lib.rs").unwrap();
    let mut converter = Converter::new();

    for child in entity.get_children() {
        match child.get_kind()  {
            EntityKind::StructDecl => {
                converter.set_location(&child);
                let name = get_name(&child);
                let mut  str_def = StructGen::new(&name);
                //println!("  StructDecl {}: ", name);

                for field in child.get_children() {
                    converter.set_location(&field);
                    let fld_name = get_name(&field);
                    let fld_type = get_type(&field);

                    str_def.add_field(&fld_name, &converter.c_to_rust_type(&fld_type));
                    //println!("    {}: {}", fld_name, fld_type);
                }
                let def = str_def.get_rust_code();
                //println!("{}", def);
                rust_code += &def;
                converter.add_struct(&name);
                //println!("{rust_code}");
                // exit(0);
            },
            _ => {
                println!("  {child:?}");
            },
        }
    }

    write!(output, "{rust_code}").unwrap();
}
