#!/usr/bin/env python3
"""diff_bits.py — XOR-diff two bitstreams, print the set-bit (row,col) coords.
  $PY diff_bits.py <baseline.fs> <cell.fs>
Reuses apycula.bslib (same method as legacy/indices.py).  This is the fuse-extraction
primitive: baseline=empty design, cell=design with ONE primitive -> the diff bits are
that primitive's config fuses + routing.
"""
import sys, os, json
sys.path.insert(0, os.environ.get("APICULA_ROOT", "."))
import numpy as np
from apycula.bslib import read_bitstream

# read_bitstream -> (bitmap, hdr, ftr, extra); bitmap is a bitmatrix (list/np wrapper).
# Coerce to a uint8 2D array. NOTE: bitmap cols are BYTE-cols (== chipdb tile.width units).
base = np.asarray(read_bitstream(sys.argv[1])[0], dtype=np.uint8)
cell = np.asarray(read_bitstream(sys.argv[2])[0], dtype=np.uint8)
if base.shape != cell.shape:
    print(f"shape mismatch {base.shape} vs {cell.shape}", file=sys.stderr); sys.exit(1)
xor = np.unpackbits(base ^ cell, axis=1)          # bit-cols = byte-col*8 + bitpos
coords = np.transpose(np.nonzero(xor)).astype(int)
# report BOTH the bit frame and the byte-col/bitpos (the chipdb tile.width frame)
out = [[int(r), int(c), int(c // 8), int(c % 8)] for r, c in coords]  # row, bitcol, bytecol, bit
print(json.dumps({"n_bits": len(out), "fmt": "[row, bitcol, bytecol, bitpos]", "bits": out}))
