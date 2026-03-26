#!/usr/bin/env -S uv run --script
# /// script
# dependencies = [
#   "zopflipy",
# ]
# ///
import gzip
import io
import sys
import zipfile
import tempfile
import subprocess
from collections.abc import Sequence
from pathlib import Path
from zopfli import ZipFile, ZopfliCompressor, ZOPFLI_FORMAT_GZIP


def run(*args: str) -> None:
    print("+", *args, file=sys.stderr, flush=True);
    subprocess.run(args, check=True, shell=False, stdout=sys.stderr.buffer)


def compress_zip_file(path: Path) -> str:
    data = io.BytesIO(bytes_ := path.read_bytes())

    with ZipFile(data, "r") as source:
        with ZipFile(path, "w", zipfile.ZIP_DEFLATED) as out:
            for zip_info in source.infolist():
                out.writestr(zip_info, source.read(zip_info.filename))

    return f"{path.as_posix()}: {path.stat().st_size / len(bytes_)}"


def main(args: Sequence[str]) -> str | int:
    if len(args) != 1:
        return f"USAGE: {sys.argv[0]} <DIR>"

    [directory] = args

    tmp_dir = Path(tempfile.mkdtemp()).absolute()

    run("pyproject-build", "-s", "-o", tmp_dir.as_posix())

    [sdist] = list(tmp_dir.glob("*"));

    targz = sdist.read_bytes()
    tar = gzip.decompress(targz)
    compressor = ZopfliCompressor(ZOPFLI_FORMAT_GZIP)
    zopflitar = compressor.compress(tar) + compressor.flush()
    compression_result = f"{sdist.as_posix()}: {len(zopflitar) / len(targz)}"
    sdist.write_bytes(zopflitar)

    for arch in ["x86_64", "aarch64"]:  # , "riscv64"
        run(
            "podman",
            "run",
            f"--arch={arch}",
            "--rm",
            f"--volume={directory}:/io",
            f"--volume={tmp_dir.as_posix()}:/dist",
            "ghcr.io/pyo3/maturin",
            "build",
            "--out=/dist",
            "--release",
        )

    print(compression_result)
    for wheel in tmp_dir.glob("*.whl"):
        print(compress_zip_file(wheel))

    return 0


sys.exit(main(sys.argv[1:]))
