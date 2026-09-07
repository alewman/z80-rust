Copies of `examples/conformance/` from z80-python at commit 530cad3 (MIT):
the two example manifests and their committed reference traces. The
`rung1` integration test runs the manifests on this core and compares
every record with the reference's, ignoring only the `mnemonic` and
`operands` text an external producer omits.
