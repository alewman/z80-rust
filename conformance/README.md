# Conformance manifests

Manifests in the format `docs/conformance.md` in z80-python defines. Each one
pins down a complete machine so the reference core and this core can be run
in lockstep:

```text
z80-trace <manifest.json> | python -m z80_python.conformance diff <manifest.json> -
```

- `zex/` -- `zexall.json` and `zexdoc.json` for rung 3. The GPL-2.0 ZEX
  binaries are not bundled: place or symlink `zexall.com` (SHA-256
  `6e2da551…8537e8`) and `zexdoc.com` (`34923a7e…f2ae8f`) beside the
  manifests. Load address 0x0100, SP 0xF000, word 0x00F0 at address 6, the
  `cpm-minimal` host, exactly as `validation/zex.py` sets the machine up.
- `interrupts/` -- rung 5: the ten scenarios of
  `validation/interrupt_crosscheck.py` as manifests with `events`. Each
  keeps the cross-check's default initial state (SP 0xFFF0, IFF1 = IFF2 = 1,
  IM 1) unless the scenario overrides it, and `max_steps` is the exact number
  of `step()` calls the Python side of the cross-check makes, so the last
  record's `after` state is the state the cross-check compares.
