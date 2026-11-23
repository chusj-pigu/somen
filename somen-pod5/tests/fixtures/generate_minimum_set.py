"""Generate the minimum structural/schema fixture set for somen-pod5.

Extends generate.py's single synthetic read with fixtures covering:

  - version-v{0..6}: real POD5 files at each schema version (v0..v6),
    sourced from nanoporetech/pod5-file-format's own cross-version test
    data (a real R9.4.1 MinION DNA run re-encoded at each schema version).
  - split-{1,2}.pod5: a real split-container pod5 pair (the signal table
    lives in a separate physical file from read/run_info), same source.
  - multi-run-info: two real runs (the v6 fixture above + a real nf-core
    CI fixture) filtered to 2 reads each and merged via `pod5 merge`, so
    reads reference two distinct run_info rows.
  - multi-batch: 5 synthetic reads written with an artificially small
    read/signal table batch size, forcing 3 Arrow record batches per
    table from a handful of reads.
  - raw-signal: 2 synthetic reads written with uncompressed (non-VBZ)
    signal.
  - dictionary-variety: 3 synthetic reads with deliberately distinct
    end_reason/pore_type values, to exercise multi-value dictionary
    decoding (the small real runs sampled while designing this set all
    happened to collapse to a single value for these columns).
  - real: a 3-read subset of the real nf-core CI fixture, as a sanity
    check against a file this project never wrote.

Requires network access, `git` + `git-lfs`, and the real `pod5` package
(`pip install pod5`). This is a one-off generator, not part of the build
or test run -- run it manually and commit its (small) output.
"""
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from uuid import uuid4

import numpy as np
import pod5
import lib_pod5.pod5_format_pybind as p5b

OUT = Path(__file__).parent
# Invoke the `pod5` CLI from the same environment as this interpreter -- a
# stray, older `pod5` elsewhere on PATH can silently fail to open files
# written by a newer schema version (verified: a 0.3.36 global install
# refused a real v6-schema fixture that 0.3.49 read and auto-migrated fine).
POD5_CLI = str(Path(sys.executable).parent / "pod5")
POD5_FILE_FORMAT_REPO = "https://github.com/nanoporetech/pod5-file-format.git"
NFCORE_FIXTURE_URL = (
    "https://raw.githubusercontent.com/nf-core/test-datasets/nanoseq/pod5/batch_0.pod5"
)


def extract_tables(pod5_path: Path, out_prefix: str) -> None:
    """Extracts the read/signal/run_info embedded IPC tables as raw bytes."""
    data = pod5_path.read_bytes()
    with pod5.Reader(pod5_path) as reader:
        for name in ("read", "signal", "run_info"):
            location = getattr(reader.inner_file_reader, f"get_file_{name}_table_location")()
            (OUT / f"{out_prefix}-{name}.arrow").write_bytes(
                data[location.offset : location.offset + location.length]
            )


def small_batch_writer(path: Path, read_batch: int, signal_batch: int, software_name: str):
    """Opens a pod5.Writer with a custom (small) per-table batch size.

    pod5.Writer's public constructor doesn't expose read_table_batch_size /
    signal_table_batch_size, so this reaches into the lower-level
    FileWriterOptions/create_file API and replicates Writer.__init__'s
    remaining bookkeeping. Fragile across pod5 versions by nature of using
    internal attributes -- acceptable here since this is a manual,
    occasionally-rerun generator script, not shipped code.
    """
    options = p5b.FileWriterOptions()
    options.read_table_batch_size = read_batch
    options.signal_table_batch_size = signal_batch
    raw = p5b.create_file(str(path), software_name, options)
    if not raw:
        raise RuntimeError(f"failed to open writer: {p5b.get_error_string()}")
    writer = pod5.Writer.__new__(pod5.Writer)
    writer._path = path.absolute()
    writer._software_name = software_name
    writer._signal_compression_type = options.signal_compression_type
    writer._writer = raw
    writer._end_reasons = {}
    writer._pores = {}
    writer._run_infos = {}
    writer._index_caches = {
        pod5.EndReason: writer._end_reasons,
        str: writer._pores,  # pod5.writer.PoreType is a plain `str` alias
        pod5.RunInfo: writer._run_infos,
    }
    writer._adder_funcs = {
        pod5.EndReason: writer._add_end_reason,
        str: writer._add_pore_type,
        pod5.RunInfo: writer._add_run_info,
    }
    return writer


def make_run_info(name: str) -> pod5.RunInfo:
    t = datetime(2024, 1, 1, tzinfo=timezone.utc)
    return pod5.RunInfo(
        acquisition_id=f"{name}-acquisition", acquisition_start_time=t,
        adc_max=8191, adc_min=-8192, context_tags={"sample": name},
        experiment_name=name, flow_cell_id="FAKE001", flow_cell_product_code="synthetic",
        protocol_name=name, protocol_run_id=f"{name}-run", protocol_start_time=t,
        sample_id=name, sample_rate=4000, sequencing_kit="synthetic",
        sequencer_position="X1", sequencer_position_type="synthetic", software="fixture",
        system_name="synthetic", system_type="synthetic", tracking_id={},
    )


def make_read(
    run_info: pod5.RunInfo,
    read_number: int,
    channel: int = 1,
    pore_type: str = "synthetic",
    end_reason: pod5.EndReasonEnum = pod5.EndReasonEnum.UNKNOWN,
) -> pod5.Read:
    return pod5.Read(
        read_id=uuid4(),
        pore=pod5.Pore(channel=channel, well=1, pore_type=pore_type),
        calibration=pod5.Calibration(offset=0.0, scale=1.0), read_number=read_number,
        start_sample=0, median_before=0.0,
        end_reason=pod5.EndReason(end_reason, False), run_info=run_info,
        signal=np.array([0, 1, -1, 100], dtype=np.int16),
    )


def gen_raw_signal(tmp: Path) -> None:
    path = tmp / "raw-signal.pod5"
    run = make_run_info("raw-signal")
    with pod5.Writer(
        path, software_name="somen raw-signal fixture",
        signal_compression_type=pod5.SignalType.UncompressedSignal,
    ) as w:
        w.add_read(make_read(run, read_number=1))
        w.add_read(make_read(run, read_number=2))
    extract_tables(path, "raw-signal")


def gen_dictionary_variety(tmp: Path) -> None:
    path = tmp / "dictionary-variety.pod5"
    run = make_run_info("dictionary-variety")
    with pod5.Writer(path, software_name="somen dictionary-variety fixture") as w:
        w.add_read(make_read(run, read_number=1, pore_type="not_set", end_reason=pod5.EndReasonEnum.UNKNOWN))
        w.add_read(make_read(run, read_number=2, pore_type="rapid", end_reason=pod5.EndReasonEnum.SIGNAL_POSITIVE))
        w.add_read(make_read(run, read_number=3, pore_type="rapid", end_reason=pod5.EndReasonEnum.MUX_CHANGE))
    extract_tables(path, "dictionary-variety")


def gen_multi_batch(tmp: Path) -> None:
    path = tmp / "multi-batch.pod5"
    run = make_run_info("multi-batch")
    w = small_batch_writer(path, read_batch=2, signal_batch=2, software_name="somen multi-batch fixture")
    try:
        for i in range(5):
            w.add_read(make_read(run, read_number=i + 1, channel=i + 1))
    finally:
        w.close()
    extract_tables(path, "multi-batch")


def fetch_pod5_file_format_test_data(tmp: Path) -> Path:
    clone = tmp / "pod5-file-format"
    subprocess.run(
        ["git", "clone", "--depth", "1", "--filter=blob:none", POD5_FILE_FORMAT_REPO, str(clone)],
        check=True,
    )
    subprocess.run(["git", "lfs", "pull", "--include", "test_data/*.pod5"], cwd=clone, check=True)
    return clone / "test_data"


def gen_versions_and_split(test_data: Path, tmp: Path) -> None:
    # All 7 files are the same original run re-encoded at each schema
    # version, so the same read_ids exist in each -- subset every version
    # down to 2 reads so the (otherwise ~1.3 MB, mostly-signal) fixture
    # stays minimal.
    with pod5.Reader(test_data / "multi_fast5_zip_v0.pod5") as r:
        ids = [str(read.read_id) for read in list(r.reads())[:2]]
    for v in range(7):
        subset = filter_to_ids(test_data / f"multi_fast5_zip_v{v}.pod5", ids, tmp / f"version-v{v}.pod5")
        extract_tables(subset, f"version-v{v}")
    (OUT / "split-1.pod5").write_bytes((test_data / "split_1_v4.pod5").read_bytes())
    (OUT / "split-2.pod5").write_bytes((test_data / "split_2_v4.pod5").read_bytes())


def fetch_nfcore_fixture(tmp: Path) -> Path:
    path = tmp / "nfcore_batch_0.pod5"
    subprocess.run(["curl", "-sL", NFCORE_FIXTURE_URL, "-o", str(path)], check=True)
    return path


def filter_to_ids(src: Path, ids: list, out_path: Path) -> Path:
    ids_file = out_path.with_suffix(".ids.txt")
    ids_file.write_text("\n".join(ids))
    subprocess.run([POD5_CLI, "filter", str(src), "-o", str(out_path), "-i", str(ids_file)], check=True)
    return out_path


def gen_real_subset(nfcore_path: Path, tmp: Path) -> None:
    with pod5.Reader(nfcore_path) as r:
        ids = [str(read.read_id) for read in list(r.reads())[:3]]
    out_path = filter_to_ids(nfcore_path, ids, tmp / "real.pod5")
    extract_tables(out_path, "real")


def gen_multi_run_info(version6_path: Path, nfcore_path: Path, tmp: Path) -> None:
    with pod5.Reader(version6_path) as r:
        v6_ids = [str(read.read_id) for read in list(r.reads())[:2]]
    with pod5.Reader(nfcore_path) as r:
        nf_ids = [str(read.read_id) for read in list(r.reads())[:2]]

    v6_subset = filter_to_ids(version6_path, v6_ids, tmp / "v6-subset.pod5")
    nf_subset = filter_to_ids(nfcore_path, nf_ids, tmp / "nf-subset.pod5")

    merged = tmp / "multi-run-info.pod5"
    subprocess.run([POD5_CLI, "merge", str(v6_subset), str(nf_subset), "-o", str(merged)], check=True)
    extract_tables(merged, "multi-run-info")


def main() -> None:
    with tempfile.TemporaryDirectory() as tmpdir:
        tmp = Path(tmpdir)

        print("Fetching pod5-file-format test_data (git lfs)...")
        test_data = fetch_pod5_file_format_test_data(tmp)
        gen_versions_and_split(test_data, tmp)

        print("Fetching nf-core nanoseq pod5 fixture...")
        nfcore_path = fetch_nfcore_fixture(tmp)
        gen_real_subset(nfcore_path, tmp)
        gen_multi_run_info(test_data / "multi_fast5_zip_v6.pod5", nfcore_path, tmp)

        print("Writing synthetic structural fixtures...")
        gen_raw_signal(tmp)
        gen_dictionary_variety(tmp)
        gen_multi_batch(tmp)

    print("Done.")


if __name__ == "__main__":
    main()
