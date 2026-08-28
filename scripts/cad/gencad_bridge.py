#!/usr/bin/env python3
"""GenCAD -> NeoTrix GenCadWire JSON bridge.

Wraps ferdous-alam/GenCAD `inference_gencad.py`, converts its `cadlib`
command output into the `GenCadWire` JSON consumed by
`neotrix l2_world_impl::cad_generator::GenCadWire`.

Deployment:
  1. git clone https://github.com/ferdous-alam/GenCAD
  2. pip install -r GenCAD/requirements.txt   # includes pythonocc-core
  3. Download pretrained ckpt into GenCAD/data/ckpt/
  4. Run:
       python3 gencad_bridge.py \
           --gencad_dir /path/to/GenCAD \
           --image_path input.png \
           --ckpt /path/to/GenCAD/data/ckpt/gencad.pt \
           --out /tmp/neotrix_gencad_out.json

The Rust side (`Backend::Model`) invokes this script with
`-image_path <img> --ckpt <ckpt> --out <out.json>` and parses the
produced JSON. Adjust `cadlib_to_gencad_wire` field names to match the
exact `cadlib` version you deploy (see GenCAD/cadlib for the command schema).
"""
import argparse
import json
import sys


def cadlib_to_gencad_wire(cad_commands):
    """Map GenCAD cadlib command tokens -> NeoTrix GenCadWire primitives."""
    out = []
    for c in cad_commands:
        op = c.get("command")
        if op in ("SOL", "CIRCLE"):
            out.append(
                {"op": "circle", "cx": c.get("cx", 0.0), "cy": c.get("cy", 0.0), "r": c.get("r", 1.0)}
            )
        elif op == "LINE":
            out.append(
                {"op": "line", "x1": c["x1"], "y1": c["y1"], "x2": c["x2"], "y2": c["y2"]}
            )
        elif op == "ARC":
            out.append(
                {
                    "op": "arc",
                    "cx": c.get("cx", 0.0),
                    "cy": c.get("cy", 0.0),
                    "r": c.get("r", 1.0),
                    "a0": c.get("a0", 0.0),
                    "a1": c.get("a1", 3.141592653589793),
                }
            )
        elif op in ("EXTRUDE", "EXT"):
            out.append({"op": "extrude", "depth": c.get("depth", 5.0)})
        elif op == "FILLET":
            out.append({"op": "fillet", "radius": c.get("radius", 0.2)})
        elif op in ("BOOL", "BOOLEAN"):
            out.append(
                {"op": "bool", "bool_op": c.get("bool_op", "union"), "target": c.get("target", "")}
            )
    return {"commands": out}


def main():
    ap = argparse.ArgumentParser(description="GenCAD -> NeoTrix GenCadWire bridge")
    ap.add_argument("-image_path", required=True, help="input image (GenCAD is image-conditioned)")
    ap.add_argument("--ckpt", required=True, help="GenCAD pretrained checkpoint")
    ap.add_argument("--out", required=True, help="output GenCadWire JSON path")
    ap.add_argument("--gencad_dir", default=".", help="path to cloned GenCAD repo")
    args = ap.parse_args()

    sys.path.insert(0, args.gencad_dir)
    import inference_gencad as g

    # Replace with the actual GenCAD inference entry point signature.
    cad_commands = g.run_inference(args.image_path, args.ckpt)
    wire = cadlib_to_gencad_wire(cad_commands)
    with open(args.out, "w") as f:
        json.dump(wire, f)
    print(f"[gencad_bridge] wrote {len(wire['commands'])} commands -> {args.out}")


if __name__ == "__main__":
    main()
