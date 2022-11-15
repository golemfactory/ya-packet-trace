use ya_packet_trace::{packet_trace, packet_trace_maybe};

fn main() {
    env_logger::init();

    packet_trace!("main::1", { &[1, 2, 3] });
    packet_trace!("main::2", { b"123" });
    packet_trace_maybe!("main::2", { None::<Vec<u8>> });
    packet_trace_maybe!("main::2", { Some(b"12") });
}
