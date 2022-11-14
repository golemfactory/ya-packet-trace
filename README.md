# ya-packet-trace
Utility for tracing VPN packets

## Usage
At any interesting point in the flow of VPN packets, invoke the macro like so:
```rust
packet_trace!(
    "possibly-slow-subsystem-before",
    {
        // code returning AsRef<[u8]> corresponding to the packet payload
    }
);
```

Then, if the `ya-packet-trace` is compiled with the `enabled` feature, a log like
`possibly-slow-subsystem-before,<HASH>` (where `<HASH>` is 16-character, 0-padded hex of FxHash output)
will be printed at `TRACE` level to target `packet-trace`.