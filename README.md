# z80-rust

A Rust Z80 instruction core transcribed from
[z80-python](https://github.com/alewman/z80-python) and proven equivalent to
it at every processor boundary: boundary kind, T-states, instruction bytes,
and all of `CPUState`, not merely "passes ZEX at the end".

The module layout mirrors the reference so the two can be read side by
side. Each `src/*.rs` file is the transcription of one `z80_python/*.py`
module (`flags`, `state`, `core`, `alu`, `blocks`, `control`, `dispatch`,
`index`, `index_dispatch`, `io`, `loads`, `rotate`, `cpu`), with the same
handler names, the same explicit if-chain dispatch, and the hardware
comments carried over. Nothing is redesigned; where a difference was forced
by the language it is described in the module header.

## Certification

Pinned oracles, as `docs/conformance.md` in z80-python requires:

| What | Version |
| --- | --- |
| z80-python (reference core and conformance kit) | 0.4.0.dev0 at commit `530cad3` |
| Trace schema | version 1 |
| SingleStepTests/z80 corpus | revision `ebe1875d48f374bcfd4b505d8eb8ee751568b5f7` |
| raxoft/z80test | release 1.2a |

Ladder status at this commit, each rung reproduced with the script named
(`scripts/`) on Linux x86_64 with rustc 1.93.1, CPython 3.14.4 and PyPy
7.3.20 / Python 3.11.13:

| Rung | What | Result | Script |
| --- | --- | --- | --- |
| 1 | Both example manifests diff clean against the reference | `flags-and-branches: traces are identical`, `interrupts: traces are identical` | `rung1.sh` |
| 2 | SingleStepTests, 1,604 files: registers, RAM, port order, T-states | `TOTAL: 1604000 passed, 0 failed, 0 not implemented / 1604000 cases` | `rung2.sh` |
| 3 | ZEXALL diffed in lockstep against the reference (`cpm-minimal`), 116 segments of 50,000,000 records | `zexall: every segment identical` and `zexdoc: every segment identical`: 5,764,169,474 records and 46,734,975,782 T-states each, stopped on `cpm_exit`; 6 h 55 min and 7 h 11 min wall with 30 PyPy processes | `rung3.sh` |
| 4 | z80test natively: `z80full`, `z80ccf`, `z80memptr` | all three `Result: all tests passed.` | `rung4.sh` |
| 5 | The ten interrupt scenarios of `validation/interrupt_crosscheck.py` as manifests with events | all ten `traces are identical` | `rung5.sh` |

CI (`.github/workflows/ci.yml`) reproduces rungs 1 and 2 on every push, plus
`cargo fmt --check`, `cargo clippy -D warnings`, and `cargo test` (which
replays the two example manifests against copies of the reference traces
without needing Python).

### Reproducing

```text
scripts/fetch_z80python.sh           # z80-python at 530cad3 into external/, with a venv
scripts/fetch_test_vectors.sh        # SingleStepTests/z80 at the pinned revision into external/
scripts/rung1.sh
scripts/rung2.sh
# place zexall.com / zexdoc.com in conformance/zex/ (SHA-256 in conformance/README.md)
JOBS=30 scripts/rung3.sh zexall path/to/pypy3
JOBS=30 scripts/rung3.sh zexdoc path/to/pypy3
scripts/rung4.sh path/to/z80test-1.2a   # directory holding the .tap files
scripts/rung5.sh
```

Rung 3 runs the reference at a few tens of thousands of records per second
against billions of records, so `rung3.sh` splits the run into segments:
`z80-trace --checkpoint-every N` writes a manifest every N records holding
the full memory image and every `CpuState` field, and the segments are
diffed in parallel. Each segment starts from the state the previous one
ended in, so a divergence anywhere is reported by the segment holding it,
and a clean result on every segment is a clean result for the whole run.
`SEGMENT=0 JOBS=1` runs the single unsegmented pipe instead.

## Using the core

```rust
use z80_rust::{Bus, Z80};

struct Flat([u8; 0x10000]);

impl Bus for Flat {
    fn read_byte(&mut self, addr: u16) -> u8 { self.0[usize::from(addr)] }
    fn write_byte(&mut self, addr: u16, value: u8) { self.0[usize::from(addr)] = value; }
    fn read_port(&mut self, _addr: u16) -> u8 { 0xFF }
    fn write_port(&mut self, _addr: u16, _value: u8) {}
}

let mut cpu = Z80::new(Flat([0; 0x10000]));
cpu.bus.0[..2].copy_from_slice(&[0x3E, 0x2A]); // LD A,0x2A
let t_states = cpu.step().unwrap();
assert_eq!((t_states, cpu.a), (7, 0x2A));
```

`step()` services an asserted RESET, a latched NMI, or an acceptable
maskable interrupt before fetching, exactly as the reference's `step()`
does; `request_*` / `clear_*` are the lifecycle API; `capture_state()` and
`restore_state()` move the complete processor state. `step()` returns
`Err(Fault)` in exactly the situations where the reference raises
`NotImplementedError` (a DD/FD prefix followed by DD, FD, or ED; IM 0 with a
non-RST vector byte) and leaves the state as the reference leaves it.

Binaries:

- `z80-trace <manifest>`: run a conformance manifest and stream its trace
  (`docs/trace-schema.md`) to stdout or `--out`.
- `z80-vectors <dir>`: the SingleStepTests runner.
- `z80-z80test <tap>...`: the z80test runner.

## Notes for the reference

Two things this port turned up are proposed upstream in
[z80-python#3](https://github.com/alewman/z80-python/pull/3): `CPUState` has
28 fields where three places in the docs say 29, and the interrupt-scenario
manifests in `conformance/interrupts/` are shipped as
`examples/conformance/interrupts/` with their reference traces.

## Where the brief is

[docs/handoff-brief.md](docs/handoff-brief.md) is the prompt this
repository was built from, verbatim, and [docs/build-record.md](docs/build-record.md)
is what happened when it was run.

## Speed

A plain `step()` loop over ZEXALL with the `cpm-minimal` traps (no trace
records, no state capture) runs the 5,764,169,474 instructions in 116 s on
one core of an i9-13900K: about 50 million instructions per second, or a
400 MHz Z80. `z80-trace --no-trace`, which also captures the state before
and after every boundary, takes 151 s.

## License

MIT, like the reference. The SingleStepTests corpus (MIT), the ZEX
exercisers (GPL-2.0), and z80test (MIT) are external and not bundled.
