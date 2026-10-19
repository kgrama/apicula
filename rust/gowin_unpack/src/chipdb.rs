//! Chip database: the subset of `apycula.chipdb.Device` the unpacker reads, loaded from the
//! same `<device>.msgpack.xz` (msgspec encodes dataclasses as maps, tuples and sets as arrays).

use indexmap::IndexMap;
use serde::Deserialize;
use std::collections::HashMap;
use std::io::Read;

pub type Coord = (i32, i32);
pub type Bits = Vec<Coord>;
/// `{dest: {src: bits}}`, in the database's order.
pub type PipTable = IndexMap<String, IndexMap<String, Bits>>;
/// `{feature tuple: bits}` (shortval / longval tables).
pub type FuseTable = IndexMap<Vec<i32>, Bits>;

/// A portmap value: one wire name or a nested list of them.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum PortVal {
    One(String),
    Many(Vec<PortVal>),
}

impl PortVal {
    /// The wire name when this is a single wire.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            PortVal::One(s) => Some(s),
            PortVal::Many(_) => None,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Bel {
    pub flags: IndexMap<i64, Bits>,
    pub modes: IndexMap<String, Bits>,
    pub portmap: IndexMap<String, PortVal>,
    pub fuse_cell_offset: Option<Coord>,
}

impl Bel {
    /// Union of every mode's bits.
    pub fn mode_bits(&self) -> Vec<Coord> {
        let mut v: Vec<Coord> = self.modes.values().flatten().copied().collect();
        v.sort_unstable();
        v.dedup();
        v
    }

    /// Single-wire port name.
    pub fn port(&self, name: &str) -> Option<&str> {
        self.portmap.get(name).and_then(|p| p.as_str())
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Tile {
    pub width: usize,
    pub height: usize,
    pub ttyp: i32,
    pub pips: PipTable,
    pub clock_pips: PipTable,
    pub bels: IndexMap<String, Bel>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(default)]
pub struct Device {
    pub grid: Vec<Vec<i32>>,
    pub tiles: HashMap<i32, Tile>,
    pub pinout: HashMap<String, HashMap<String, IndexMap<String, (String, Vec<String>)>>>,
    pub pin_bank: HashMap<String, i32>,
    pub logicinfo: HashMap<String, IndexMap<Coord, i32>>,
    pub shortval: HashMap<i32, HashMap<String, FuseTable>>,
    pub longval: HashMap<i32, HashMap<String, FuseTable>>,
    #[serde(rename = "const")]
    pub consts: HashMap<i32, Bits>,
    pub hclk_pips: IndexMap<Coord, PipTable>,
    pub nodes: IndexMap<String, (String, Vec<(i32, i32, String)>)>,
    pub corner_tiles_io: HashMap<Coord, String>,
}

impl Device {
    pub fn rows(&self) -> usize {
        self.grid.len()
    }

    pub fn cols(&self) -> usize {
        self.grid.first().map_or(0, |r| r.len())
    }

    /// Tile type data at (row, col), 0-based.
    pub fn tile(&self, row: usize, col: usize) -> &Tile {
        &self.tiles[&self.grid[row][col]]
    }

    /// `{code: (attr, value)}`: the reverse of a logicinfo table.
    pub fn rev_logicinfo(&self, name: &str) -> HashMap<i32, Coord> {
        self.logicinfo
            .get(name)
            .map(|t| t.iter().map(|(av, code)| (*code, *av)).collect())
            .unwrap_or_default()
    }

    /// `{bank: (row, col)}` of tiles holding a `BANKn` bel.
    pub fn bank_tiles(&self) -> IndexMap<i32, (usize, usize)> {
        let mut res = IndexMap::new();
        for row in 0..self.rows() {
            for col in 0..self.cols() {
                for bel in self.tile(row, col).bels.keys() {
                    if let Some(n) = bel.strip_prefix("BANK").and_then(|s| s.parse().ok()) {
                        res.insert(n, (row, col));
                    }
                }
            }
        }
        res
    }
}

/// Load `<path>` (xz-compressed MessagePack).  Logs and returns None on any failure.
pub fn load_chipdb(path: &str) -> Option<Device> {
    let file = std::fs::File::open(path)
        .map_err(|e| eprintln!("chipdb {path}: {e}"))
        .ok()?;
    let mut raw = Vec::new();
    xz2::read::XzDecoder::new(file)
        .read_to_end(&mut raw)
        .map_err(|e| eprintln!("chipdb {path}: xz: {e}"))
        .ok()?;
    rmp_serde::from_slice(&raw)
        .map_err(|e| eprintln!("chipdb {path}: msgpack: {e}"))
        .ok()
}

const UTURN: [(char, char); 4] = [('N', 'S'), ('S', 'N'), ('E', 'W'), ('W', 'E')];

fn uturn(d: char) -> char {
    UTURN.iter().find(|(a, _)| *a == d).map_or(d, |(_, b)| *b)
}

/// `chipdb.wire2global`: 1-based (row, col) and a tile-local wire to its global name.
pub fn wire2global(row: i64, col: i64, rows: i64, cols: i64, wire: &str) -> String {
    if wire == "VCC" || wire == "VSS" {
        return wire.to_string();
    }
    let b = wire.as_bytes();
    // re.match(r"([NESW])([128]\d)(\d)"): a 4-character prefix match, the rest is dropped
    let inter = b.len() >= 4
        && matches!(b[0], b'N' | b'E' | b'S' | b'W')
        && matches!(b[1], b'1' | b'2' | b'8')
        && b[2].is_ascii_digit()
        && b[3].is_ascii_digit();
    if !inter {
        return format!("R{row}C{col}_{wire}");
    }
    let mut dir = b[0] as char;
    let num = &wire[1..3];
    let seg = (b[3] - b'0') as i64;
    let (dr, dc) = match dir {
        'N' => (1, 0),
        'E' => (0, -1),
        'S' => (-1, 0),
        _ => (0, 1),
    };
    let (mut r, mut c) = (row + dr * seg, col + dc * seg);
    if r < 1 {
        r = 1 - r;
        dir = uturn(dir);
    }
    if c < 1 {
        c = 1 - c;
        dir = uturn(dir);
    }
    if r > rows {
        r = 2 * rows + 1 - r;
        dir = uturn(dir);
    }
    if c > cols {
        c = 2 * cols + 1 - c;
        dir = uturn(dir);
    }
    format!("R{r}C{c}_{dir}{num}")
}

/// `chipdb.rc2tbrl_0`: 0-based (row, col) to an `IO<edge><n>` pin name prefix.
pub fn loc2pin_name(db: &Device, row: usize, col: usize) -> String {
    let edge = db
        .corner_tiles_io
        .get(&(row as i32, col as i32))
        .cloned()
        .unwrap_or_else(|| {
            if row == 0 {
                "T".into()
            } else if row == db.rows() - 1 {
                "B".into()
            } else if col == 0 {
                "L".into()
            } else {
                "R".into()
            }
        });
    let idx = if edge == "T" || edge == "B" { col + 1 } else { row + 1 };
    format!("IO{edge}{idx}")
}
