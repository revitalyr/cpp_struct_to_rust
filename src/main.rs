use std::env;
use clang::{Clang, Index, EntityKind};

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

    for child in entity.get_children() {
        match child.get_kind()  {
            EntityKind::StructDecl => {
                println!("  StructDecl {}: ", child.get_display_name().unwrap());
                //print!(" has_attributes {}", child.has_attributes());
                for field in child.get_children() {
                    let fld_type = field.get_type().unwrap().get_display_name();
                    println!("    {}: {}", field.get_display_name().unwrap(), fld_type);
                }
            },
            _ => {
                //println!("  {child:?}");
            },
        }
    }
}
