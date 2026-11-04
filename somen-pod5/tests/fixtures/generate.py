"""Generate synthetic POD5 and its three embedded IPC files with pod5 0.3.36."""
from datetime import datetime, timezone
from pathlib import Path
from tempfile import TemporaryDirectory
from uuid import UUID

import numpy as np
import pod5

out = Path(__file__).parent
t = datetime(2024, 1, 1, tzinfo=timezone.utc)
run = pod5.RunInfo(
    acquisition_id="synthetic-acquisition", acquisition_start_time=t,
    adc_max=8191, adc_min=-8192, context_tags={"sample": "synthetic"},
    experiment_name="synthetic", flow_cell_id="FAKE001", flow_cell_product_code="synthetic",
    protocol_name="synthetic", protocol_run_id="synthetic-run", protocol_start_time=t,
    sample_id="synthetic", sample_rate=4000, sequencing_kit="synthetic",
    sequencer_position="X1", sequencer_position_type="synthetic", software="fixture",
    system_name="synthetic", system_type="synthetic", tracking_id={},
)
read = pod5.Read(
    read_id=UUID("00000000-0000-0000-0000-000000000001"),
    pore=pod5.Pore(channel=1, well=1, pore_type="synthetic"),
    calibration=pod5.Calibration(offset=0.0, scale=1.0), read_number=1,
    start_sample=0, median_before=0.0,
    end_reason=pod5.EndReason(pod5.EndReasonEnum.UNKNOWN, False), run_info=run,
    signal=np.array([0, 1, -1, 100], dtype=np.int16),
)
with TemporaryDirectory() as tmp:
    path = Path(tmp) / "synthetic.pod5"
    with pod5.Writer(path, software_name="somen synthetic fixture") as writer:
        writer.add_read(read)
    with pod5.Reader(path) as reader:
        data = path.read_bytes()
        for name in ("read", "signal", "run_info"):
            location = getattr(reader.inner_file_reader, f"get_file_{name}_table_location")()
            (out / f"synthetic-{name}.arrow").write_bytes(data[location.offset:location.offset + location.length])
