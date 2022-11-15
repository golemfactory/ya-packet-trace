use ya_packet_trace::packet_trace;

fn main() {
    env_logger::init();

    packet_trace!("main::1", { &[1, 2, 3] });
    packet_trace!("main::2", { b"123" });
}
