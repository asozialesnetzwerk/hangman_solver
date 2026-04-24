#!/usr/bin/env -S uv run --script
# /// script
# dependencies = [
#   "zopflipy",
# ]
# ///
from bz2 import compress
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

    tmp_dir = Path(tempfile.mkdtemp(prefix=f"dist-{Path(directory).absolute().name}-")).absolute()

    run("pyproject-build", "-s", "-o", tmp_dir.as_posix())

    [sdist] = list(tmp_dir.glob("*"));

    for arch in ["aarch64", "x86_64"]:  # TODO; "riscv64"
        for manylinux in ["manylinux_2_17", "musllinux_1_1"]:
            image = "ghcr.io/joshix-1/pyo3-maturin:main"
            if "musllinux" in manylinux:
                image += "-musllinux"
            run(
                "podman",
                "run",
                f"--arch={arch}",
                "--rm",
                "--pull=newer",
                f"--volume={directory}:/io",
                f"--volume={tmp_dir.as_posix()}:/dist",
                image,
                "build",
                "--compression-level=1",  # gets recompressed anyway
                "--future-incompat-report",
                "--auditwheel=repair",
                "--out=/dist",
                "--release",
                f"--compatibility={manylinux}",
            )

    targz = sdist.read_bytes()
    tar = gzip.decompress(targz)
    compressor = ZopfliCompressor(ZOPFLI_FORMAT_GZIP)
    zopflitar = compressor.compress(tar) + compressor.flush()
    sdist.write_bytes(zopflitar)
    print(f"{sdist.as_posix()}: {len(zopflitar) / len(targz)}", file=sys.stderr)

    for wheel in tmp_dir.glob("*.whl"):
        print(compress_zip_file(wheel), file=sys.stderr, flush=True)

    print(tmp_dir)

    return 0


sys.exit(main(sys.argv[1:]))
