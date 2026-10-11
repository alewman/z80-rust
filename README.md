# z80-rust

A Rust Z80 instruction core transcribed from
[z80-python](https://github.com/alewman/z80-python) and proven equivalent to
it at every processor boundary: boundary kind, T-states, instruction bytes,
and all of `CPUState`, not merely "passes ZEX at the end".

The module layout mirrors the reference so the two can be read side by
side. Each `src/*.rs` file is the transcription of one `z80_python/*.py`
module (`flags`, `state`, `core`, `alu`, `blocks`, `control`, `dispatch`,
`index`, `index_dispatch`, `io`, `loads`, `rotate`, `cpu`, `timing`), with the same
handler names, the same explicit if-chain dispatch, and the hardware
comments carried over. Nothing is redesigned; where a difference was forced
by the language it is described in the module header.

## Certification

Pinned oracles, as `docs/conformance.md` in z80-python requires:

| What | Version |
| --- | --- |
| z80-python (reference core and conformance kit) | main at commit `b62891b` (after 0.4.0: the EI delay fix, RESET clearing I and R, the opt-in CMOS `OUT (C),0`, and the access-timing table; the commit megadrive-python pins). Earlier pins: `cab1598`, `530cad3` |
| Trace schema | version 1 |
| SingleStepTests/z80 corpus | revision `ebe1875d48f374bcfd4b505d8eb8ee751568b5f7` |
| raxoft/z80test | release 1.2a |
| FUSE Z80 core tests | release 1.6.0 (`fuse-1.6.0.tar.gz`, SHA-256 `3a8fedf2…047096`) |

Ladder status at this commit, each rung reproduced with the script named
(`scripts/`) on Linux x86_64 with rustc 1.93.1, CPython 3.14.4 and PyPy
7.3.20 / Python 3.11.13:

| Rung | What | Result | Script |
| --- | --- | --- | --- |
| 1 | All three example manifests diff clean against the reference | `flags-and-branches`, `interrupts`, `prefix-sequences`: `traces are identical` | `rung1.sh` |
| 2 | SingleStepTests, 1,604 files: registers, RAM, port order, T-states, and on every case the kinds and order of the accesses `step()` makes against the access-timing table | `TOTAL: 1604000 passed, 0 failed, 0 not implemented / 1604000 cases` | `rung2.sh` |
| 3 | ZEXALL and ZEXDOC diffed in lockstep against the reference (`cpm-minimal`), 116 segments of 50,000,000 records each | `zexall: every segment identical` and `zexdoc: every segment identical`: 5,764,169,474 records and 46,734,975,782 T-states each, stopped on `cpm_exit`. At `b62891b`: 6 h 55 min and 7 h 30 min wall with 30 PyPy processes (at `cab1598`: 6 h 58 min and 6 h 43 min wall with 30 PyPy processes (at `530cad3`: 6 h 55 min and 7 h 11 min) | `rung3.sh` |
| 4 | z80test natively: `z80full`, `z80ccf`, `z80memptr` | all three `Result: all tests passed.` | `rung4.sh` |
| 5 | The ten interrupt scenarios of `validation/interrupt_crosscheck.py` as manifests with events (now shipped upstream in `examples/conformance/interrupts/`), plus three for the behaviour `b62891b` changed: `ei-ei-rearms-delay`, `ei-di-leaves-delay-zero`, `reset-clears-i-r` (each diverges on the `cab1598` core) | all thirteen `traces are identical` | `rung5.sh` |
| 6 | FUSE 1.6.0's Z80 core test set, 1,356 emulator-derived cases, with the six divergences z80-python explains pinned as strict expected failures | `1350 agree, 6 expected divergences, 0 unexpected` | `rung6.sh` |
| 7 | `timing::instruction_timing` answers as the reference's `z80_python.timing.instruction_timing` does, for every form the lookup distinguishes (unprefixed, CB, all 256 ED, DD/FD, DD/FD CB with two displacements, stray prefixes one and two deep) | `instruction_timing: 6180 forms, every answer identical` | `rung7.sh` |

CI (`.github/workflows/ci.yml`) reproduces rungs 1, 2, 5, 6, and 7 on every push, plus
`cargo fmt --check`, `cargo clippy -D warnings`, and `cargo test` (which
replays the two example manifests against copies of the reference traces
without needing Python).

### Reproducing

```text
scripts/fetch_z80python.sh           # z80-python at b62891b into external/, with a venv
scripts/fetch_test_vectors.sh        # SingleStepTests/z80 at the pinned revision into external/
scripts/rung1.sh
scripts/rung2.sh
# place zexall.com / zexdoc.com in conformance/zex/ (SHA-256 in conformance/README.md)
JOBS=30 scripts/rung3.sh zexall path/to/pypy3
JOBS=30 scripts/rung3.sh zexdoc path/to/pypy3
scripts/rung4.sh path/to/z80test-1.2a   # directory holding the .tap files
scripts/rung5.sh
scripts/fetch_fuse_tests.sh            # FUSE 1.6.0 z80/tests into external/
scripts/rung6.sh
scripts/rung7.sh
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
`Err(Fault)` in exactly the situation where the reference raises
`NotImplementedError` (IM 0 with a non-RST vector byte, below) and leaves
the state as the reference leaves it.

## Stated limitations

- **IM 0 accepts only the eight RST opcodes.** A real device can put any
  instruction on the bus in interrupt mode 0, and a multi-byte one is
  fetched from the bus over further acknowledge cycles. Both cores stop with
  an error for a non-RST byte and leave the request pending. No oracle in
  the ladder exercises non-RST IM 0 (SingleStepTests, z80test, ZEX, and FUSE
  all skip it), and supporting it changes the shape of the pending-request
  field and therefore the trace schema, so it stays a stated limitation
  until a target machine needs it. Same as the reference,
  `docs/interrupt-lifecycle.md`.
- Bus-level timing, memory contention, WAIT states, interrupt-acknowledge
  callbacks, and daisy chains are outside both cores' claims, as
  `docs/conformance.md` in the reference says. `step()` does not count
  T-states as it goes; where in an instruction each access falls is
  answered beside the core, from a table (below).

## Access timing

`timing::instruction_timing(bytes)` takes an instruction's bytes, as a host
peeks them at PC before `step()`, and returns every memory and I/O access
the instruction can make, in order, each with its T-state offset, plus each
way it can execute (`(t_states, accesses_made)`). It is for boards that
arbitrate a shared bus, such as the Mega Drive's. It transcribes
`z80_python/timing.py`; the table (`src/access_timing.rs`) is generated from
the reference's by `scripts/gen_access_timing.py`, and the reference derives
it from the SingleStepTests corpus's per-T-state bus samples, so its tier is
emulator-derived. Interrupt acknowledge cycles and the HALT idle cycle are
not covered.

The behaviour fixes at `b62891b`, each from the reference:

- **EI re-arms the delay** ([z80-python#18](https://github.com/alewman/z80-python/pull/18)):
  the delay ends as the instruction after EI starts, so `EI; EI` takes a
  pending interrupt only after the next instruction (Young 5.5), and
  `EI; DI` leaves `ei_delay` 0 rather than the -1 the reference's
  `CPUState` rejects.
- **RESET clears I and R** (Zilog UM0080, "RESET"; found by megadrive-python,
  whose Sonic 3 sound driver reads R after a board reset).
- **`Z80::cmos_out_c_zero`** (default `false`): the undocumented
  `OUT (C),0` drives 0xFF, as CMOS parts such as the Game Gear's are
  reported to. Reported, not hardware-captured.

The rest of `cab1598..b62891b` is internal to the reference (the callable
bus, the speed rungs' one-table dispatch and collapsed IX/IY handlers,
tooling, documentation) and changes no behaviour; this port keeps its
explicit if-chain dispatch and handler names, which still read side by side
with the reference's rules and citations.

Binaries:

- `z80-trace <manifest>`: run a conformance manifest and stream its trace
  (`docs/trace-schema.md`) to stdout or `--out`.
- `z80-vectors <dir>`: the SingleStepTests runner.
- `z80-z80test <tap>...`: the z80test runner.
- `z80-timing`: reads instruction bytes in hex on stdin and prints
  `instruction_timing` for each (rung 7).
- `z80-fuse <dir>`: the FUSE core-test runner (rung 6). FUSE's expectations
  are emulator-derived; six cases disagree with the hardware-derived
  oracles, for reasons z80-python's `docs/validation.md` records, and the
  runner requires exactly those six to diverge.

## Prefix runs

At `cab1598` the reference gained hardware-faithful handling of runs of
DD/FD prefixes and of DD/FD before ED ([z80-python#5](https://github.com/alewman/z80-python/pull/5):
each stray prefix is a 4-T-state M1 that bumps R, the last one decides IX or
IY, an ED after a prefix runs the ED instruction unchanged, and no interrupt
is accepted inside the run). `src/index_dispatch.rs` transcribes it, and the
bytes an instruction occupies are now unbounded, since the run is. The rule
is documentation-derived (Sean Young, *The Undocumented Z80 Documented*
v0.91, sections 3.7 and 6.1, chapter 5) and has no hardware-captured vector;
z80-python's `docs/validation.md` states that tier, and this port inherits
it. `Fault::UnhandledIndexOpcode` is now unreachable and kept only as the
table's completeness guard.

## Notes for the reference

Three things this port turned up went upstream and are merged:
[z80-python#3](https://github.com/alewman/z80-python/pull/3) (`CPUState` has
28 fields where the docs said 29, and the interrupt-scenario manifests in
`conformance/interrupts/` shipped as `examples/conformance/interrupts/` with
their reference traces), [#4](https://github.com/alewman/z80-python/pull/4)
(the reference's CI workflow had failed to parse since before this work), and
[#5](https://github.com/alewman/z80-python/pull/5) (prefix runs, above).

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
