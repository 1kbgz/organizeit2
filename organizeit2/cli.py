from typing import Annotated

from rich.console import Console
from rich.table import Table
from typer import Exit, Option, Typer

from organizeit2 import Directory

_list = list


def _unmatched_table(unmatch, size: bool = False, modified: bool = False):
    if unmatch:
        table = Table(title="Unmatched")
        table.add_column("Path", style="cyan")
        if size:
            table.add_column("Size", style="cyan")
        if modified:
            table.add_column("Modified", style="cyan")
        for _ in unmatch:
            row = [str(_)]
            if size:
                row.append(str(_.size(0)))
            if modified:
                row.append(_.modified().isoformat())
            table.add_row(*row)
        console = Console()
        console.print(table)
    else:
        print("All matched")


def match(
    directory: str,
    pattern: str,
    *,
    list: Annotated[bool, Option("--list/--no-list", "-l/-L")] = False,
    name_only: Annotated[bool, Option("--name-only/--no-name-only", "-n/-N")] = True,
    invert: Annotated[bool, Option("--invert/--no-invert", "-i/-I")] = False,
    size: Annotated[bool, Option("--size/--no-size", "-s/-S")] = False,
    modified: Annotated[bool, Option("--modified/--no-modified", "-m/-M")] = False,
    limit: Annotated[int, Option("--limit")] = None,
    leaves: Annotated[int, Option("--leaves")] = None,
    by: Annotated[str, Option("--by")] = None,
    desc: Annotated[bool, Option("--desc")] = False,
    block_size: Annotated[int, Option("--block-size")] = 0,
) -> bool:
    p = Directory(path=directory).resolve()
    if not isinstance(p, Directory) or not directory.endswith("/"):
        matched = [p] if p.resolve().match(pattern, name_only=name_only, invert=invert) else []
        all = [] if matched else [p]
    else:
        matched = p.all_match(pattern, name_only=name_only, invert=invert)
        all = p.ls()

    # calculate the overlap
    intersection = _list(set(all) - set(matched))

    # Handle limit
    if limit or leaves:
        if by == "age":
            intersection = sorted(intersection, key=lambda x: x.modified(), reverse=desc)
        elif by == "size":
            intersection = sorted(intersection, key=lambda x: x.size(block_size), reverse=desc)
        elif by is None:
            # Don't do anything
            pass
        else:
            raise NotImplementedError()

        if leaves:
            intersection = intersection[:-leaves]
        if limit:
            intersection = intersection[:limit]

    if list:
        for _ in intersection:
            print(_.as_posix())
    else:
        _unmatched_table(intersection, size=size or by == "size", modified=modified or by == "modified")
    raise Exit(min(len(intersection), 1))


def rematch(
    directory: str,
    pattern: str,
    *,
    list: Annotated[bool, Option("--list/--no-list", "-l/-L")] = False,
    name_only: Annotated[bool, Option("--name-only/--no-name-only", "-n/-N")] = True,
    invert: Annotated[bool, Option("--invert/--no-invert", "-i/-I")] = False,
    size: Annotated[bool, Option("--size/--no-size", "-s/-S")] = False,
    modified: Annotated[bool, Option("--modified/--no-modified", "-m/-M")] = False,
    limit: Annotated[int, Option("--limit")] = None,
    leaves: Annotated[int, Option("--leaves")] = None,
    by: Annotated[str, Option("--by")] = None,
    desc: Annotated[bool, Option("--desc")] = False,
    block_size: Annotated[int, Option("--block-size")] = 0,
) -> bool:
    p = Directory(path=directory).resolve()
    if not isinstance(p, Directory) or not directory.endswith("/"):
        matched = [p] if p.resolve().rematch(pattern, name_only=name_only, invert=invert) else []
        all = [] if matched else [p]
    else:
        matched = p.all_rematch(pattern, name_only=name_only, invert=invert)
        all = p.ls()

    # calculate the overlap
    intersection = _list(set(all) - set(matched))

    # Handle limit
    if limit or leaves:
        if by == "age":
            intersection = sorted(intersection, key=lambda x: x.modified(), reverse=desc)
        elif by == "size":
            intersection = sorted(intersection, key=lambda x: x.size(block_size), reverse=desc)
        elif by is None:
            # Don't do anything
            pass
        else:
            raise NotImplementedError()

        if leaves:
            intersection = intersection[:-leaves]
        if limit:
            intersection = intersection[:limit]

    # return code means everything looked for was matched
    return_code = 0 if not intersection else 1
    if list:
        for _ in intersection:
            print(_.as_posix())
    else:
        _unmatched_table(intersection, size=size or by == "size", modified=modified or by == "modified")
    raise Exit(return_code)


def main(_test: bool = False):
    app = Typer()
    app.command("match")(match)
    app.command("all-match")(match)
    app.command("rematch")(rematch)
    app.command("all-rematch")(rematch)
    if _test:
        return app
    return app()


if __name__ == "__main__":
    main()
