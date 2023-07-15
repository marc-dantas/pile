from sys import stderr
from typing import TextIO, Tuple, Iterable


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
    for line in break_line_at(25, msg):
        indent(stderr, f"|    {line}")
    if note is not None:
        for line in break_line_at(25, note):
            indent(stderr, f"+ {line}")
    exit(1)


def break_line_at(char_pos: int, value: str) -> Iterable[str]:
    words = value.split()
    current_line = ''
    current_length = 0
    for word in words:
        word_length = len(word) + 1
        if current_length + word_length > char_pos:
            yield current_line
            current_line = ''
            current_length = 0
        current_line += f'{word} '
        current_length += word_length
    yield current_line

throw(("a.pl", 1, 0),
      "error kind",
      "error message to be displayed error message to be displayed error message to be displayed",
      "error message to be displayed error message to be displayed error message to be displayed")
