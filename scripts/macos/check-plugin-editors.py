#!/usr/bin/env python3
"""Exercise independently linked VST3 editor binaries in isolated macOS processes."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("pump", type=Path, help="Pump bundle executable")
    parser.add_argument("gainsnap", type=Path, help="GainSnap bundle executable")
    parser.add_argument("--legacy-pump", type=Path)
    parser.add_argument("--legacy-gainsnap", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    sdk = os.environ["VST3_SDK_DIR"]
    probe = output / "plugin-editor-probe"
    subprocess.run([
        "xcrun", "clang++", "-std=c++17", "-framework", "Cocoa", "-I", sdk,
        str(Path(__file__).with_name("plugin-editor-probe.mm")), "-o", str(probe),
    ], check=True)
    pump, gain = args.pump.resolve(), args.gainsnap.resolve()
    cases = [
        ("pump-gainsnap", [pump, gain], False),
        ("gainsnap-pump", [gain, pump], False),
        ("multiple-instances", [pump, gain, pump, gain], False),
        ("close-first", [gain, pump], True),
    ]
    for name, legacy, fixed in [
        ("old-pump", args.legacy_pump, gain),
        ("old-gainsnap", args.legacy_gainsnap, pump),
    ]:
        if legacy:
            cases.extend([(name + "-first", [legacy.resolve(), fixed], False),
                          (name + "-last", [fixed, legacy.resolve()], False)])
    results = []
    failed = False
    for name, paths, close_first in cases:
        env = dict(os.environ, RUST_BACKTRACE="1")
        env.pop("PROBE_CLOSE_EACH", None)
        env.pop("PROBE_KEY_PASSTHROUGH", None)
        if not name.startswith("old-"):
            env["PROBE_KEY_PASSTHROUGH"] = "1"
        if close_first:
            env["PROBE_CLOSE_EACH"] = "1"
        try:
            result = subprocess.run([str(probe), *map(str, paths)], env=env,
                                    stdout=subprocess.PIPE, stderr=subprocess.STDOUT,
                                    timeout=90, check=False)
            log = result.stdout
            passed = result.returncode == 0 and b"PASS attach" in log
            # Fully fixed pairs must also be free of duplicate ObjC fixture warnings.
            if not name.startswith("old-"):
                passed = passed and b"implemented in both" not in log
            status = result.returncode
        except subprocess.TimeoutExpired as error:
            log, passed, status = error.stdout or b"", False, "timeout"
        (output / (name + ".log")).write_bytes(log)
        results.append({"case": name, "passed": passed, "status": status})
        failed |= not passed
        print(json.dumps(results[-1]), flush=True)
    binaries = sorted({path for _, paths, _ in cases for path in paths})
    manifest = {"results": results, "binaries": [
        {"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest(),
         "uuid": subprocess.check_output(["xcrun", "dwarfdump", "--uuid", str(path)], text=True).strip()}
        for path in binaries
    ]}
    (output / "results.json").write_text(json.dumps(manifest, indent=2) + "\n")
    raise SystemExit(1 if failed else 0)


if __name__ == "__main__":
    main()
