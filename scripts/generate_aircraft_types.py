"""Generate the embedded validation lookup from data/AircraftTypes.json."""
import json
from pathlib import Path

root = Path(__file__).resolve().parents[1]
categories = {}
for aircraft in json.loads((root / "data/AircraftTypes.json").read_text()):
    designator = aircraft["Designator"].strip().upper()
    if designator:
        categories.setdefault(designator, set()).update(aircraft["WTC"].split("/"))
lookup = {key: "/".join(sorted(values)) for key, values in sorted(categories.items())}
(root / "assets/aircraft-types.json").write_text(json.dumps(lookup, indent=2) + "\n")
