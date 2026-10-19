//! `.fs` reader (`apycula.bslib.read_bitstream`) into a packed bit matrix, and the per-tile
//! views of it (`chipdb.tile_bitmap`).

use crate::chipdb::Device;
use std::collections::HashMap;

/// Row-major bit matrix, 64 bits per word.
pub struct BitMatrix {
    pub height: usize,
    pub width: usize,
    words: usize,
    data: Vec<u64>,
}

impl BitMatrix {
    fn new(height: usize, width: usize) -> Self {
        let words = width.div_ceil(64);
        BitMatrix { height, width, words, data: vec![0; height * words] }
    }

    fn set(&mut self, y: usize, x: usize) {
        self.data[y * self.words + x / 64] |= 1u64 << (x % 64);
    }

    pub fn get(&self, y: usize, x: usize) -> bool {
        y < self.height && x < self.width && (self.data[y * self.words + x / 64] >> (x % 64)) & 1 == 1
    }

    /// Any bit set in rows y..y+h, cols x..x+w.
    fn any(&self, y: usize, x: usize, h: usize, w: usize) -> bool {
        (y..(y + h).min(self.height)).any(|r| (x..(x + w).min(self.width)).any(|c| self.get(r, c)))
    }
}

/// One tile's window into the device bitmap.
#[derive(Clone, Copy)]
pub struct TileBits<'a> {
    pub m: &'a BitMatrix,
    pub y: usize,
    pub x: usize,
}

impl TileBits<'_> {
    pub fn get(&self, row: i32, col: i32) -> bool {
        row >= 0 && col >= 0 && self.m.get(self.y + row as usize, self.x + col as usize)
    }
}

struct Frames {
    rows: Vec<Vec<u8>>,
    is_5a: bool,
}

fn line_bytes(bits: &[u8]) -> Vec<u8> {
    bits.chunks(8)
        .map(|c| c.iter().fold(0u8, |a, b| (a << 1) | (b - b'0')))
        .collect()
}

/// Device-ID line -> (padding, compress_padding, is 5A series).
fn device_padding(ba: &[u8]) -> Option<(usize, usize, bool)> {
    match ba {
        [6, 0, 0, 0, 0x11, 0, 0x58, 0x1b] | [6, 0, 0, 0, 0x11, 0, b'H', 0x1b] => Some((4, 44, false)),
        [6, 0, 0, 0, 0x09, 0, 0x28, 0x1b] | [6, 0, 0, 0, 0x01, 0, b'h', 0x1b] => Some((0, 0, false)),
        [6, 0, 0, 0, 0x01, 0x20, b'h', 0x1b] => Some((0, 0, false)),
        [6, 0, 0, 0, 0x01, 0, b'8', 0x1b] | [6, 0, 0, 0, 0x01, 0, 0x98, 0x1b] => Some((0, 8, false)),
        [6, 0, 0, 0, 0, 0, 0x08, 0x1b] => Some((0, 16, false)),
        // GW5A-25A, GW5AT-60B, GW5AST-138C
        [6, 0, 0, 0, 0, 1, 0x28, 0x1b] | [6, 0, 0, 0, 0, 1, 0x48, 0x1b] | [6, 0, 0, 0, 0, 1, 0x08, 0x1b] => Some((3, 43, true)),
        _ => None,
    }
}

/// Header state while scanning the command lines before the frames.
struct Hdr {
    preamble: usize,
    frames: usize,
    compressed: bool,
    keys: HashMap<Vec<u8>, usize>,
    pad: usize,
    cpad: usize,
    is_5a: bool,
}

fn header_line(h: &mut Hdr, ba: &[u8]) -> Option<()> {
    if ba.first() == Some(&0x10) && ba.len() >= 8 && (u64::from_be_bytes(ba[..8].try_into().ok()?) >> 13) & 1 == 1 {
        h.compressed = true;
    }
    if ba.first() == Some(&0x51) && ba.len() >= 8 {
        let key = |b: u8| format!("{b:08b}").into_bytes();
        h.keys.insert(key(ba[5]), 8);
        if ba[6] != 0 {
            h.keys.insert(key(ba[6]), 4);
            if ba[7] != 0 {
                h.keys.insert(key(ba[7]), 2);
            }
        }
    }
    if h.preamble == 0 && ba.first() == Some(&0x3b) {
        h.frames = ba[2..].iter().fold(0usize, |a, b| (a << 8) | *b as usize);
    }
    if h.preamble == 0 && ba.first() == Some(&0x06) {
        let Some((p, c, five)) = device_padding(ba) else {
            eprintln!("unsupported device id {ba:02x?}");
            return None;
        };
        (h.pad, h.cpad, h.is_5a) = (p, c, five);
    }
    h.preamble = h.preamble.saturating_sub(1);
    Some(())
}

/// `bitarr` after optional decompression: the content bits of one frame line.
fn frame_bits(h: &Hdr, raw: &[u8]) -> Vec<u8> {
    let (src, pad): (Vec<u8>, usize) = if h.compressed {
        let cut = raw.len().saturating_sub(64);
        let mut out = Vec::with_capacity(raw.len() * 2);
        for chunk in raw[..cut].chunks(8) {
            match h.keys.get(chunk) {
                Some(n) => out.extend(std::iter::repeat_n(b'0', 8 * n)),
                None => out.extend_from_slice(chunk),
            }
        }
        out.extend_from_slice(&raw[cut..]);
        (out, h.cpad)
    } else {
        (raw.to_vec(), h.pad)
    };
    let s = trim(&src);
    if s.len() < pad + 64 {
        return Vec::new();
    }
    s[pad..s.len() - 64].to_vec()
}

fn trim(s: &[u8]) -> &[u8] {
    let a = s.iter().position(|c| !c.is_ascii_whitespace()).unwrap_or(s.len());
    let b = s.iter().rposition(|c| !c.is_ascii_whitespace()).map_or(a, |p| p + 1);
    &s[a..b]
}

fn read_frames(text: &[u8]) -> Option<Frames> {
    let mut h = Hdr { preamble: 3, frames: 0, compressed: false, keys: HashMap::new(), pad: 0, cpad: 0, is_5a: false };
    let mut rows = Vec::new();
    let mut in_hdr = true;
    for line in text.split_inclusive(|c| *c == b'\n') {
        if line.starts_with(b"//") {
            continue;
        }
        if h.frames == 0 {
            if in_hdr {
                header_line(&mut h, &line_bytes(trim(line)))?;
                in_hdr = h.frames == 0;
            }
            // footer and slots: the unpacker only prints slot attributes, not needed here
            continue;
        }
        // python keeps the raw line incl. its newline for the compressed path
        rows.push(frame_bits(&h, line));
        h.frames -= 1;
    }
    Some(Frames { rows, is_5a: h.is_5a })
}

/// Read a `.fs` and return the device bitmap (fliplr, then transposed on GW5A parts).
pub fn read_bitstream(path: &str) -> Option<BitMatrix> {
    let text = std::fs::read(path).map_err(|e| eprintln!("{path}: {e}")).ok()?;
    let f = read_frames(&text)?;
    let w0 = f.rows.iter().map(|r| r.len()).max().unwrap_or(0);
    let n = f.rows.len();
    let mut m = if f.is_5a { BitMatrix::new(w0, n) } else { BitMatrix::new(n, w0) };
    for (j, row) in f.rows.iter().enumerate() {
        let len = row.len();
        for (k, b) in row.iter().enumerate() {
            if *b == b'1' {
                let i = len - 1 - k; // fliplr
                if f.is_5a { m.set(i, j) } else { m.set(j, i) }
            }
        }
    }
    Some(m)
}

/// `chipdb.tile_bitmap`: (row, col) -> window, for tiles with any bit set.
pub fn tile_bitmap<'a>(db: &Device, m: &'a BitMatrix) -> Vec<((usize, usize), TileBits<'a>)> {
    let mut res = Vec::new();
    let mut y = 0;
    for r in 0..db.rows() {
        let mut x = 0;
        let mut h = 0;
        for c in 0..db.cols() {
            let td = db.tile(r, c);
            if m.any(y, x, td.height, td.width) {
                res.push(((r, c), TileBits { m, y, x }));
            }
            x += td.width;
            h = td.height;
        }
        y += h;
    }
    res
}
