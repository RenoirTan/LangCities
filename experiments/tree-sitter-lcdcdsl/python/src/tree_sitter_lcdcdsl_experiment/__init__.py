from pprint import pprint
import tree_sitter_lcdcdsl
from tree_sitter import Language, Parser, TreeCursor


source_str = """
$mt.sc.ot_mt_m(/ˈlɛzwʊnɔɥɑ/) + $mt.sc.ot_mt_n(/ˈlɪŋ/)
"""


def tree_cursor_to_python(cursor: TreeCursor, source: bytes = bytes()) -> dict:
    data = {
        "name": cursor.node.type,
        "start": (cursor.node.start_point.row, cursor.node.start_point.column),
        "end": (cursor.node.end_point.row, cursor.node.end_point.column),
    }
    n_children = len(cursor.node.children)
    # print(f"in: {cursor.node.type} {n_children}")
    if n_children <= 0:
        if source:
            data["text"] = str(
                source[cursor.node.start_byte : cursor.node.end_byte],
                encoding="utf-8",
            )
    else:
        data["descendants"] = []
        child_cursor = cursor.node.walk()  # cursor.copy() crashes
        child_cursor.goto_first_child()
        for i in range(n_children):
            d = tree_cursor_to_python(child_cursor, source)
            data["descendants"].append(d)
            child_cursor.goto_next_sibling()
    # print(f"out: {cursor.node.type}")
    return data


def main() -> None:
    language = Language(tree_sitter_lcdcdsl.language())
    parser = Parser(language)
    source = bytes(source_str, "utf-8")
    tree = parser.parse(source)
    # tree.print_dot_graph(1)  # print to stdout
    pythonic = tree_cursor_to_python(tree.walk(), source)
    pprint(pythonic)
