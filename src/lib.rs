/// Logs location and hash of provided data
#[cfg(feature = "enable")]
#[macro_export]
macro_rules! packet_trace {
    ($location:expr, $payload:block) => {
        {
            let hash = $crate::helpers::do_hash($payload);
            let ts = $crate::helpers::ts();
            $crate::helpers::log::trace!(target: "packet-trace", "{},{:016x},{}", $location, hash, ts);
        }
    };
}

/// Logs location and hash of provided data
#[cfg(not(feature = "enable"))]
#[macro_export]
macro_rules! packet_trace {
    ($location:expr, $payload:block) => {};
}

/// Logs location and hash of provided data
#[cfg(feature = "enable")]
#[macro_export]
macro_rules! packet_trace_maybe {
    ($location:expr, $maybe_payload:block) => {{
        if let Some(payload) = $maybe_payload {
            packet_trace!($location, { payload })
        }
    }};
}

/// Logs location and hash of provided data
#[cfg(not(feature = "enable"))]
#[macro_export]
macro_rules! packet_trace_maybe {
    ($location:expr, $maybe_payload:block) => {};
}

/// Date format used for timestamps
pub const DATE_FORMAT_STR: &str = "%Y-%m-%dT%H:%M:%S%.6f%z";

/// Macro internals
///
/// While this module must be public due to the way Rust
/// expands declarative macros, no guarantees are made regarding
/// this module.
pub mod helpers {
    pub use log;

    pub fn do_hash(data: impl AsRef<[u8]>) -> u64 {
        use std::hash::Hasher;

        let mut hasher = fxhash::FxHasher64::default();
        hasher.write(data.as_ref());

        hasher.finish()
    }

    pub fn ts() -> String {
        chrono::Utc::now()
            .format(crate::DATE_FORMAT_STR)
            .to_string()
    }
}

#[cfg(test)]
mod test {
    use log::LevelFilter;
    use once_cell::sync::OnceCell;
    use serial_test::serial;
    use std::fmt::Write;
    use std::sync::atomic::{AtomicBool, Ordering::SeqCst};
    use std::sync::{Arc, Mutex};

    #[cfg(feature = "enable")]
    use regex::Regex;

    #[cfg(feature = "enable")]
    use crate::DATE_FORMAT_STR;

    struct StringLog(Arc<Mutex<String>>);

    static LOGGER: OnceCell<StringLog> = OnceCell::new();

    impl StringLog {
        fn new() -> Self {
            StringLog(Arc::new(Mutex::new(String::new())))
        }

        fn global() -> &'static Self {
            let first_init = AtomicBool::new(false);
            let result = LOGGER.get_or_init(|| {
                first_init.store(true, SeqCst);
                Self::new()
            });

            if first_init.load(SeqCst) {
                log::set_logger(StringLog::global()).unwrap();
                log::set_max_level(LevelFilter::Trace);
            }

            result
        }

        fn get_string() -> String {
            Self::global().0.lock().unwrap().clone()
        }

        fn clear() {
            Self::global().0.lock().unwrap().clear();
        }
    }

    impl log::Log for StringLog {
        fn enabled(&self, _metadata: &log::Metadata) -> bool {
            true
        }

        fn log(&self, record: &log::Record) {
            let mut buf = self.0.lock().unwrap();
            writeln!(&mut buf, "{}", record.args()).unwrap()
        }

        fn flush(&self) {}
    }

    #[test]
    fn test_invocation_compiles() {
        if false {
            packet_trace!("test-1", { &[1, 2, 3] });
        }
    }

    #[cfg(feature = "enable")]
    #[test]
    #[serial]
    pub fn test_date() {
        StringLog::clear();

        packet_trace!("test-date", { &[1, 2, 3] });
        let output = StringLog::get_string();
        let date = &output["test-date,0123456789abcdef,".len()..output.len() - 1];

        assert!(chrono::DateTime::parse_from_str(&date, DATE_FORMAT_STR).is_ok());
    }

    #[cfg(feature = "enable")]
    #[test]
    #[serial]
    pub fn test_hash() {
        StringLog::clear();

        packet_trace!("test-foo", { &[1, 2, 3] });
        let expected = Regex::new(r#"test-foo,[0-9A-Fa-f]{16}.*\n"#).unwrap();

        assert!(expected.is_match(&StringLog::get_string()));
    }

    #[cfg(feature = "enable")]
    #[test]
    #[serial]
    pub fn test_twice() {
        {
            StringLog::clear();

            packet_trace!("test-foo", { &[1, 2, 3] });
            let expected = Regex::new(r#"test-foo,[0-9A-Fa-f]{16}.*\n"#).unwrap();

            assert!(expected.is_match(&StringLog::get_string()));
        }

        {
            StringLog::clear();

            packet_trace!("test-bar", { &[1, 2, 3] });
            let expected = Regex::new(r#"test-bar,[0-9A-Fa-f]{16}.*\n"#).unwrap();

            assert!(expected.is_match(&StringLog::get_string()));
        }
    }

    #[cfg(feature = "enable")]
    #[test]
    #[serial]
    pub fn test_seq_3() {
        StringLog::clear();

        packet_trace!("test-foo", { &[1, 2, 3] });
        packet_trace!("test-bar", { b"test data" });
        packet_trace!("test-baz", { vec![0u8, 12, 13, 22] });
        let expected = Regex::new(&format!(
            "{}{}{}",
            r#"test-foo,[0-9A-Fa-f]{16}.*\n"#,
            r#"test-bar,[0-9A-Fa-f]{16}.*\n"#,
            r#"test-baz,[0-9A-Fa-f]{16}.*\n"#,
        ))
        .unwrap();

        assert!(expected.is_match(&StringLog::get_string()));
    }

    #[cfg(not(feature = "enable"))]
    #[test]
    #[serial]
    pub fn test_disable() {
        StringLog::clear();

        packet_trace!("test-foo", { &[1, 2, 3] });
        packet_trace!("test-bar", { b"test data" });
        packet_trace!("test-baz", { vec![0u8, 12, 13, 22] });
        let expected = "";

        assert_eq!(StringLog::get_string(), expected);
    }
}
