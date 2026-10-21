# GW5AST-138C GTR12_QUAD open-flow smoke test

Proves a design instantiating the Gowin GTR12 high-speed SerDes (`GTR12_QUAD`) places,
routes, and packs to a working `.fs` through the pure apicula open flow — no vendor IDE.

## Flow

```sh
# 1) synth (GTR12_QUAD is a blackbox -> survives synthesis)
yosys -p "read_verilog gtr_smoke.v; synth_gowin -json gtr_smoke-synth.json -family gw5a -setundef"

# 2) place & route (GTR bel @ anchor X27Y0 / X99Y0; INET tap arcs routed)
nextpnr-himbaechel --timing-allow-fail -r --top top \
    --json gtr_smoke-synth.json --write gtr_smoke.json \
    --device GW5AST-LV138PG484AC1/I0 --vopt cst=gtr_smoke.cst

# 3) pack, baking the reverse-engineered CSR command block (config is CSR, not tile fuses)
python -m apycula.gowin_pack --cpu_as_gpio -d GW5AST-138C \
    --serdes_csr serdes.csr -o gtr_smoke.fs gtr_smoke.json
```

`serdes.csr` is the validated 375-write UPAR command block (from
`serdes_fuzz_artifacts/gtr/`); all 375 128-bit lines it emits are byte-exact against the
vendor oracle `gtr_q0.fs`.

## What this exercises vs. what is still a gap

Routed here: the `.dat`-confirmed `INET_*` tap bits (the bits that genuinely reach fabric).
The `GTR12_QUAD` bel exposes ALL prim_sim ports (so any real instantiation binds), but the
`FABRIC_*` functional ports are not yet routed to fabric — the vendor's dedicated
`FABRIC_* <-> fabric` routing mesh is not recovered from the current RE artifacts. That
full-width functional routing is the remaining step toward ECP5-DCU (Trellis) parity.
