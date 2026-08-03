import tree_sitter_lcdcdsl
from tree_sitter import Language, Parser


source_str = """
$mt.sc.ot_mt_m(/ˈlɛzwʊnɔɥɑ/) + $mt.sc.ot_mt_n(/ˈlɪŋ/)
"""


def main() -> None:
    language = Language(tree_sitter_lcdcdsl.language())
    parser = Parser(language)
    source = bytes(source_str, "utf-8")
    tree = parser.parse(source)
    tree.print_dot_graph(1)  # print to stdout
