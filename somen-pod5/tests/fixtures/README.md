# Test fixtures

`synthetic-*.arrow` and the fixtures produced by `generate_minimum_set.py` are
the embedded read/signal/run_info Arrow IPC tables extracted from small POD5
files, used directly by `IPCReader`-level tests (see
`somen-pod5/src/io/ipc_reader.rs`).

Regenerate the synthetic-only set with `generate.py`, and the full minimum
structural/schema set with `generate_minimum_set.py` (see its module
docstring for what each fixture covers and why). Both require a real `pod5`
install (`pip install pod5`); the latter also requires network access and
`git`/`git-lfs`.

## Provenance and licensing of real-data fixtures

`generate_minimum_set.py` pulls a few reads (never a whole sequencing run)
from two upstream projects:

- `version-v{0..6}-*`, `split-{1,2}.pod5` — subset from
  [nanoporetech/pod5-file-format](https://github.com/nanoporetech/pod5-file-format)'s
  own `test_data/` (its cross-schema-version and split-container regression
  fixtures), © Oxford Nanopore Technologies PLC, MPL-2.0.
- `real-*`, and one side of `multi-run-info-*` — subset from
  [nf-core/test-datasets](https://github.com/nf-core/test-datasets) (`nanoseq`
  branch, `pod5/batch_0.pod5`), MIT licensed.

All other fixtures (`raw-signal-*`, `dictionary-variety-*`, `multi-batch-*`,
and the other side of `multi-run-info-*`) are synthetic, generated directly
by the script.
