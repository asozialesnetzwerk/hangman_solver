#!/usr/bin/env -S uv run --script
# /// script
# dependencies = [
#   "zopflipy",
# ]
# ///

import io
import sys
import zipfile
from collections.abc import Sequence
from pathlib import Path
from zopfli import ZipFile

def compress_zip_file(path: Path) -> str | None:
    data = io.BytesIO(path.read_bytes())

    with ZipFile(data, "r", zipfile.ZIP_DEFLATED) as source:
        with ZipFile(path, "w", zipfile.ZIP_DEFLATED) as out:
            for zip_info in source.infolist():
                out.writestr(zip_info, source.read(zip_info.filename))

    return None

def main(args: Sequence[str]) -> str | int:
    if not args:
        return f"USAGE: {sys.argv[0]} FILES.."
    for arg in args:
        path = Path(arg)
        if not path.is_file():
            return f"{path.as_posix()} is not a file"
        if err := compress_zip_file(path):
            return f"Could not compress {path.as_posix()}: {err}"

    return 0

sys.exit(main(sys.argv[1:]))
