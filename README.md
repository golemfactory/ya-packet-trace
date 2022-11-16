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

packet_trace_maybe!(
    "whatever",
    {
        // code returning Option<AsRef<[u8]>> corresponding to the packet payload
    }
);
```

Then, if the `ya-packet-trace` is compiled with the `enabled` feature, a log like
`possibly-slow-subsystem-before,<HASH>,<TS>` will be printed at `TRACE` level
to target `packet-trace`.

* `<HASH>` is 16-character, 0-padded hex-digest of FxHash output
* `<TS>` is current datetime as formatted by [chrono](https://crates.io/crates/chrono)
with the following format string: `%Y-%m-%dT%H:%M:%S%.6f%z`.