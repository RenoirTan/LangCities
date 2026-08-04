fn main() {
    println!("Loading parser...");
    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_lcdcdsl::LANGUAGE.into())
        .expect("Failed to load");
    println!("Parser loaded!");

    let source = "$mt.sc.ot_mt_m(/ˈlɛzwʊnɔɥɑ/) + $mt.sc.ot_mt_n(/ˈlɪŋ/)";
    let tree: tree_sitter::Tree = parser.parse(source, None).unwrap();
    println!("{:?}", tree);
}
