"""Emit path, line number and comment records for the xtask format check."""

import os
from pathlib import Path
import re
import shlex
import sys
import tokenize


SHELLS = {"sh", "bash", "dash", "ash", "ksh", "zsh"}
COMMAND_WORD = re.compile(r"(?:case|esac|then|do|else|elif|if|while|until|!|\{)(?=[ \t\n;&|()<>]|$)")
HERE_DOCUMENT = re.compile(r"""<<(-?)[ \t]*((?:'[^']*'|"[^"]*"|\\[^\n]|[^\s;&|()<>'"\\])+)""")


def language(path):
    if path.suffix.removeprefix(".") in SHELLS:
        return "shell"
    if path.suffix == ".py":
        return "python"
    if path.suffix.lower() in {".md", ".markdown"}:
        return None

    with path.open("rb") as stream:
        first_line = stream.readline()
    if not first_line.startswith(b"#!"):
        return None
    for word in first_line[2:].decode("utf-8", errors="replace").split():
        interpreter = word.rsplit("/", 1)[-1]
        if interpreter in SHELLS:
            return "shell"
        if re.fullmatch(r"python(?:[23](?:\.\d+)?)?", interpreter):
            return "python"
    return None


def python_comments(path):
    with path.open("rb") as stream:
        for token in tokenize.tokenize(stream.readline):
            if token.type == tokenize.COMMENT:
                yield token.start[0], token.string


def shell_comments(source):
    """Recognize comment starts at shell word boundaries without running the script."""
    frames = []
    at_word_start = True
    at_command_start = True
    cases = []
    pending_documents = []
    offset = 0
    line = 1
    while offset < len(source):
        character = source[offset]
        frame = frames[-1] if frames else None
        if frame == "'":
            if character == "'":
                frames.pop()
        elif source.startswith("\\\n", offset):
            offset += 2
            line += 1
            continue
        elif character == "\\" and offset + 1 < len(source):
            offset += 2
            at_word_start = at_command_start = False
            continue
        elif character == frame or (
            frame == "$(" and character == ")" and len(frames) not in cases
        ):
            frames.pop()
            at_word_start = at_command_start = frame == ")"
        elif source.startswith("$((", offset):
            frames.extend(("arithmetic", "arithmetic"))
            offset += 3
            continue
        elif source.startswith("$(", offset):
            frames.append("$(")
            at_word_start = at_command_start = True
            offset += 2
            continue
        elif source.startswith("${", offset):
            frames.append("}")
            offset += 2
            continue
        elif character == "`":
            frames.append("`")
            at_word_start = at_command_start = True
        elif frame == "arithmetic":
            if character == "(":
                frames.append("arithmetic")
            elif character == ")":
                frames.pop()
        elif frame != '"' and character in "\"'":
            frames.append(character)
            at_word_start = at_command_start = False
        elif frame not in {'"', "}"}:
            word = COMMAND_WORD.match(source, offset) if at_command_start else None
            if word:
                depth = len(frames)
                if word[0] == "case":
                    cases.append(depth)
                elif word[0] == "esac" and cases and cases[-1] == depth:
                    cases.pop()
                at_command_start = word[0] not in {"case", "esac"}
                at_word_start = False
                offset = word.end()
                continue
            if character == "#" and at_word_start:
                end = source.find("\n", offset)
                if end == -1:
                    end = len(source)
                yield line, source[offset:end]
                offset = end
                continue
            if source.startswith("<<<", offset):
                offset += 3
                at_word_start = True
                continue
            document = HERE_DOCUMENT.match(source, offset)
            if document:
                strip_tabs, word = document.groups()
                delimiter, = shlex.split(word)
                pending_documents.append((strip_tabs, delimiter))
                offset = document.end()
                at_word_start = False
                continue
            if character == "\n":
                for strip_tabs, delimiter in pending_documents:
                    while offset < len(source):
                        start = offset + 1
                        end = source.find("\n", start)
                        offset = len(source) if end == -1 else end
                        line += 1
                        content = source[start:offset]
                        if (content.lstrip("\t") if strip_tabs else content) == delimiter:
                            break
                pending_documents.clear()
            if character == "(":
                frames.append(")")
            at_word_start = character in " \t\n;&|()<>"
            if character not in " \t":
                at_command_start = character in "\n;&|()"
        offset += 1
        line += character == "\n"


def main():
    for directory in ("scripts", "strength"):
        # Walk raises on missing directories and other I/O failures instead of skipping them.
        for location, _, names in os.walk(directory, onerror=raise_error):
            for name in names:
                path = Path(location, name)
                if path.is_symlink() or not path.is_file():
                    continue
                kind = language(path)
                if kind == "python":
                    comments = python_comments(path)
                elif kind == "shell":
                    comments = shell_comments(path.read_text(encoding="utf-8"))
                else:
                    continue
                for line, comment in comments:
                    record = f"{path.as_posix()}\0{line}\0{comment}\0"
                    sys.stdout.buffer.write(record.encode("utf-8"))


def raise_error(error):
    raise error


main()
