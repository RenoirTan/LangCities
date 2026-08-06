fn main() {
    println!("Loading parser...");
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_lcdcdsl::LANGUAGE.into())
        .expect("Failed to load");
    println!("Parser loaded!");

    let source = "$mt.sc.ot_mt_m(/ˈlɛzwʊnɔɥɑ/) + $mt.sc.ot_mt_n(/ˈlɪŋ/)";
    let tree = parser.parse(source, None).unwrap();
    let root_node = tree.root_node();
    println!("{:?}", root_node);
    let mut cursor = root_node.walk();
    println!("{}", root_node.child_count());
    for child_node in root_node.children(&mut cursor) {
        println!("{:?}", child_node);
    }
}
