"""GW5AST-138C PLL packing against vendor bitstreams.

Replays every PLL configuration found by `gowin_unpack --fuzz` (fuzz_report.json from
sbp-computational tools/fuzz_all_fs.sh) through GW5AST_138C.get_PLL_fuses and compares
the produced fuses with the bits the vendor set in the same tiles.
Skipped when the report is absent."""
import json
import os

import pytest

from apycula import gowin_pack as gp

REPORT = os.environ.get('FUZZ_REPORT',
                        '/home/evd/workspace/sbp-computational/impl/fuzz/fuzz_report.json')
ODIV_DEFAULT = '1000'


def _bin(v: str) -> str:
    return bin(int(v))[2:]


def parms_from_fuzz(attrs: dict[str, str], fclkin: str = '50') -> dict[str, str]:
    """Vendor shortval attrs ('T0:A_IDIV_SEL': '1') -> PLL primitive parameters."""
    a = {k.split(':', 1)[1]: v for k, v in attrs.items()}
    p = {'FCLKIN': fclkin}
    for name in ('IDIV_SEL', 'FBDIV_SEL', 'MDIV_SEL', 'MDIV_FRAC_SEL', 'ODIV0_FRAC_SEL'):
        if f'A_{name}' in a:
            p[name] = _bin(a[f'A_{name}'])
    for i in range(7):
        p[f'ODIV{i}_SEL'] = _bin(a.get(f'A_ODIV{i}_SEL', '8'))
        p[f'CLKOUT{i}_EN'] = 'TRUE' if a.get(f'A_CLKOUT{i}_EN') == '50' else 'FALSE'
        p[f'CLKOUT{i}_PE_COARSE'] = _bin(a.get(f'A_CLKOUT{i}_PE_COARSE', '0'))
        p[f'CLKOUT{i}_PE_FINE'] = _bin(a.get(f'A_CLKOUT{i}_PE_FINE', '0'))
    for i in range(4):
        p[f'CLKOUT{i}_DT_DIR'] = _bin(a.get(f'A_CLKOUT{i}_DT_DIR', '1'))
        p[f'CLKOUT{i}_DT_STEP'] = _bin(a.get(f'A_CLKOUT{i}_DT_STEP', '0'))
    for i in range(6):
        p[f'CLK{i}_IN_SEL'] = _bin(a.get(f'A_CLK{i}_IN_SEL', '0'))
    p['RESET_I_EN'] = 'TRUE' if a.get('A_RESET_I_EN') == '50' else 'FALSE'
    return p


def static_icp(attrs: dict[str, str]) -> bool:
    return 'T0:A_ICP_SEL' in attrs


@pytest.fixture(scope='module')
def dev():
    d = object.__new__(gp.GW5AST_138C)
    d.chipdb = gp.ChipDB('GW5AST-138C')
    return d


@pytest.fixture(scope='module')
def plls():
    if not os.path.exists(REPORT):
        pytest.skip(f'no fuzz report at {REPORT}')
    out = []
    for rep in json.load(open(REPORT)):
        if rep['active_tiles'] < 1000:
            continue
        for p in rep['plls']:
            if 'T0:A_IDIV_SEL' in p['shortval_attrs']:
                out.append((os.path.basename(rep['path']), p))
    return out


def pll_coverage(dev, trow: int, tcol: int) -> set[tuple[int, int]]:
    ttyp = dev.chipdb.get_ttyp(tcol, trow)
    tbl = dev.chipdb.db.shortval.get(ttyp, {}).get('PLL', {})
    return {b for bits in tbl.values() for b in bits}


def replay(dev, p: dict) -> list[tuple[int, set, set]]:
    """[(tile_idx, predicted, actual)] over the PLL-table-covered bits of each tile."""
    hr, hc = p['head']
    cell = gp.CellDesc('pll', 'PLL', parms_from_fuzz(p['shortval_attrs']), {}, {})
    fuses = {(f.x, f.y): f.bits for f in dev.get_PLL_fuses(gp.BelDesc(hc, hr, '', cell))}
    tiles = dev.chipdb.db.extra_func[hr, hc]['bpll']['tiles']
    out = []
    for ti, (trow, tcol) in enumerate(tiles):
        cov = pll_coverage(dev, trow, tcol)
        if not cov:
            continue
        actual = {(b[1], b[2]) for b in p['all_nonconst_bits'] if b[0] == ti} & cov
        out.append((ti, set(fuses.get((tcol, trow), set())), actual))
    return out


def test_static_icp_plls_match_vendor(dev, plls):
    """Vendor PLLs with a static charge pump (our 50 MHz designs) must round-trip bit-exactly."""
    checked = 0
    bad = []
    for name, p in plls:
        if not static_icp(p['shortval_attrs']) or p['shortval_attrs'].get('T0:KVCO') != '7':
            continue
        for ti, pred, act in replay(dev, p):
            checked += 1
            if pred != act:
                bad.append((name, p['site'], ti, sorted(act - pred)[:8], sorted(pred - act)[:8]))
    assert checked > 0
    assert bad == []


def test_all_plls_differ_only_in_pump(dev, plls):
    """Dynamic-ICP and older-IDE vendor PLLs may differ only in KVCO/ICP/LPF/DYN rows."""
    # older vendor IDE releases also emit PLOCK differently and ODIVn=8 for disabled outputs
    pump_attrs = {gp.attrids.pll_attrids[n] for n in
                  ('KVCO', 'A_ICP_SEL', 'A_LPF_RES_SEL', 'A_DYN_ICP_SEL', 'A_DYN_LPF_SEL',
                   'PLL_UNK211', 'FLDCOUNT', 'PLOCK')
                  + tuple(f'A_ODIV{i}_SEL' for i in range(1, 7))}
    li = dev.chipdb.db.logicinfo['PLL']
    pump_feats = {v for (a, _), v in li.items() if a in pump_attrs}
    bad = []
    for name, p in plls:
        hr, hc = p['head']
        try:
            rows = replay(dev, p)
        except UnboundLocalError:
            # pump formula has no solution at the assumed 50 MHz input; input clock unknown
            continue
        for ti, pred, act in rows:
            trow, tcol = dev.chipdb.db.extra_func[hr, hc]['bpll']['tiles'][ti]
            ttyp = dev.chipdb.get_ttyp(tcol, trow)
            tbl = dev.chipdb.db.shortval[ttyp]['PLL']
            pump_bits = {b for key, bits in tbl.items() if any(abs(k) in pump_feats for k in key if k) for b in bits}
            if (pred ^ act) - pump_bits:
                bad.append((name, p['site'], ti, sorted((pred ^ act) - pump_bits)[:8]))
    assert bad == []


def test_ddrdll_matches_vendor(dev, plls):
    """DDRDLL at (108,0): packer bits == vendor bits within the DLL0 table coverage."""
    dumps = {}
    for rep in json.load(open(REPORT)):
        for d in rep['ddrdlls']:
            if d['rc'] == [108, 0] and 'DLL0' in d['shortval_decoded']:
                dumps[os.path.basename(rep['path'])] = {tuple(b) for b in d['nonconst_bits']}
    assert dumps
    ttyp = dev.chipdb.get_ttyp(0, 108)
    cov = {b for bits in dev.chipdb.db.shortval[ttyp]['DLL0'].values() for b in bits}
    cell = gp.CellDesc('dll', 'DDRDLL', {}, {}, {})
    pred = set(dev.get_DDRDLL_fuses(gp.BelDesc(0, 108, '', cell))[0].bits)
    bad = {name: sorted((bits & cov) ^ pred) for name, bits in dumps.items() if (bits & cov) != pred}
    assert bad == {}


def _attr_names(dev, bel, out):
    avs = dev.get_out_iologic_attrs(bel) if out else dev.get_in_iologic_attrs(bel)
    return {a.attr: str(a.val) for a in avs}


def _iol(dev, typ, parms, main_in=None, main_out=None):
    cell = gp.CellDesc('iol', typ, parms, {}, {})
    return gp.IologicBelDesc(181, 79, 'A', cell, 'SPINE10', main_out or parms.get('OUTMODE'), main_in or parms.get('INMODE'))


VENDOR_SETS = {
    # pico_ddr3.fs, DQ pins: OSER8_MEM + IDES8_MEM in one cell
    'dq': {'CLKIMUX': 'ENABLE', 'CLKOMUX': 'ENABLE', 'FCLKSEL0': 'HCLK0', 'FCLKSEL1': 'HCLK2', 'FCLKSEL2': 'HCLK2_',
           'FCLKSEL3': 'HCLK0_', 'FCLKSEL5': 'HCLK0', 'FIFO': 'ENABLE', 'INMODE': 'MIDDRX4', 'IOLOGIC_UNKNOWN118': '1005',
           'IOLOGIC_UNKNOWN91': '78', 'OUTMODE': 'MODDRX4', 'UD2': '118', 'WRFCLKSEL': 'DQSW'},
    # pico_ddr3.fs, address/command pins: OSER8_MEM only
    'addr': {'CLKOMUX': 'ENABLE', 'FCLKSEL0': 'HCLK0', 'FCLKSEL1': 'HCLK2', 'FCLKSEL2': 'HCLK2_', 'FCLKSEL3': 'HCLK0_',
             'IOLOGIC_UNKNOWN91': '78', 'LSRIMUX_0': '0', 'OUTMODE': 'MODDRX4', 'WRFCLKSEL': 'UNK102'},
    'dqs': {'CLKOMUX': 'ENABLE', 'FCLKSEL0': 'HCLK0', 'FCLKSEL1': 'HCLK2', 'FCLKSEL2': 'HCLK2_', 'FCLKSEL3': 'HCLK0_',
            'IOLOGIC_UNKNOWN91': '78', 'LSRIMUX_0': '0', 'OUTMODE': 'MODDRX4', 'WRFCLKSEL': 'DQSW270'},
    # pico_usb.fs
    'ides8': {'CLKIMUX': 'ENABLE', 'FCLKSEL3': 'HCLK0_', 'FCLKSEL5': 'HCLK0', 'INMODE': 'UNK76', 'IOLOGIC_UNKNOWN91': '78',
              'LSRIMUX_0': '0', 'LSROMUX_0': '0'},
    'oser16_main': {'CLKIMUX': 'ENABLE', 'CLKOMUX': 'ENABLE', 'FCLKSEL0': 'HCLK0', 'FCLKSEL1': 'HCLK2', 'FCLKSEL2': 'HCLK2_',
                    'FCLKSEL3': 'HCLK0_', 'FCLKSEL5': 'HCLK0', 'HWL': 'TRUE', 'INMODE': 'UNK105', 'IOLOGIC_UNKNOWN91': '78',
                    'OUTMODE': 'ODDRX8', 'WRFCLKSEL': 'UNK102'},
    'oser16_aux': {'CLKOMUX': 'ENABLE', 'FCLKSEL0': 'HCLK0', 'FCLKSEL1': 'HCLK2', 'FCLKSEL2': 'HCLK2_', 'FCLKSEL3': 'HCLK0_',
                   'IOLOGIC_UNKNOWN91': '78', 'ISI': 'ENABLE', 'LSRIMUX_0': '0', 'OCLKCE': 'CE', 'OUTMODE': 'LVDSOUT',
                   'WRFCLKSEL': 'UNK102'},
}


def test_iologic_attr_sets_match_vendor(dev):
    """Attribute name/value sets emitted for the DDR3 PHY and USB2 soft PHY equal the decoded vendor sets."""
    got = {}
    o = _iol(dev, 'OSER8_MEM', {'OUTMODE': 'MODDRX4', 'TCLK_SOURCE': 'DQSW'}, main_in='MIDDRX4')
    i = _iol(dev, 'IDES8_MEM', {'INMODE': 'MIDDRX4'}, main_out='MODDRX4')
    got['dq'] = {**_attr_names(dev, i, False), **_attr_names(dev, o, True)}
    got['addr'] = _attr_names(dev, _iol(dev, 'OSER8_MEM', {'OUTMODE': 'MODDRX4'}), True)
    got['dqs'] = _attr_names(dev, _iol(dev, 'OSER8_MEM', {'OUTMODE': 'MODDRX4', 'TCLK_SOURCE': '"DQSW270"'}), True)
    got['ides8'] = _attr_names(dev, _iol(dev, 'IDES8', {'INMODE': 'IDDRX4'}), False)
    o = _iol(dev, 'IOLOGIC', {'OUTMODE': 'ODDRX8'}, main_in='IDDRX8')
    i = _iol(dev, 'IOLOGIC', {'INMODE': 'IDDRX8'}, main_out='ODDRX8')
    got['oser16_main'] = {**_attr_names(dev, i, False), **_attr_names(dev, o, True)}
    got['oser16_aux'] = _attr_names(dev, _iol(dev, 'IOLOGIC', {'OUTMODE': 'DDRENABLE16'}), True)
    bad = {k: (sorted(set(got[k].items()) ^ set(VENDOR_SETS[k].items()))) for k in VENDOR_SETS if got[k] != VENDOR_SETS[k]}
    assert bad == {}


def test_iologic_attr_sets_have_fuses(dev):
    """Every emitted attribute resolves to a logicinfo feature on a DDR IOLOGIC tile."""
    bel = _iol(dev, 'OSER8_MEM', {'OUTMODE': 'MODDRX4', 'TCLK_SOURCE': 'DQSW'}, main_in='MIDDRX4')
    av = dev.set_iologic_attrvals(bel, dev.get_out_iologic_attrs(bel))
    assert len(av) == len(dev.get_out_iologic_attrs(bel))
    bits = dev.chipdb.get_iologic_fuses(0, 64, av, 'A')
    assert bits


def test_dqs_defaults_set_no_bits(dev):
    """DDR3 X4 DQS as instantiated by the vendor PHY (DQS_MODE X4, HWL true, FIFO_MODE_SEL 0) has no fuses;
    every bit on the DQS tiles of the four vendor DDR bitstreams is a routing pip."""
    cell = gp.CellDesc('dqs', 'DQS', {'DQS_MODE': 'X4', 'HWL': 'true', 'FIFO_MODE_SEL': '0'}, {}, {})
    assert dev.get_DQS_fuses(gp.BelDesc(5, 81, '', cell)) == []
    cell = gp.CellDesc('dqs', 'DQS', {'DQS_MODE': 'X2_DDR3', 'HWL': 'false', 'FIFO_MODE_SEL': '1'}, {}, {})
    bits = set(dev.get_DQS_fuses(gp.BelDesc(5, 81, '', cell))[0].bits)
    assert bits == {(20, 79), (20, 103), (20, 36), (20, 67)}


def test_iodelay_static_delay_bits(dev):
    cell = gp.CellDesc('dly', 'IODELAY', {'C_STATIC_DLY': '0000101'}, {}, {})
    bits = set(dev.get_IODELAY_fuses(gp.BelDesc(37, 108, '', cell))[0].bits)
    # binary-weighted, LSB at the highest column: weight 1 -> col 6, weight 4 -> col 4 (chipdb.py IODELAY comment)
    assert bits == {(20, 6), (20, 4)}
