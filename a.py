from sys import stderr
from typing import Tuple, TextIO, Iterable

def indent(file: TextIO,
           text: str,
           prefix: int = None,
           suffix: int = None) -> None:
    if prefix is None:
        prefix = 2
    if suffix is None:
        suffix = 0
    file.write(' '*prefix + text + ' '*suffix + '\n')

def throw(pos: Tuple[str, int, int],
          kind: str, msg: str,
          note: str = None) -> None:
    stderr.write("pile: error at "
                 f"{pos[0]}:{pos[1]}:{pos[2]}:\n")
    indent(stderr, f"| {kind}:")
    for line in break_line_at(50, msg):
        indent(stderr, f"|    {line}")
    if note is not None:
        for line in break_line_at(50, note):
            indent(stderr, f"+ {line}")
    exit(1)


def break_line_at(char_pos: int, value: str) -> Iterable[str]:
    words = value.split()
    current_line = ''
    for word in words:
        if len(current_line) + len(word) + 1 <= char_pos:
            current_line += f'{word} '
        else:
            yield current_line.strip()
            current_line = f'{word} '
    if current_line:
        yield current_line.strip()


throw(("a", 1, 1), "test", "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed sagittis ex vel lorem porttitor, et ultrices nisi condimentum. Proin ultrices massa eu mauris tincidunt, ut congue")