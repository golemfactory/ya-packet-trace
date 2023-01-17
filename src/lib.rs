pub enum WriteTarget {
    Log,
    Write(Box<dyn std::io::Write + Send>),
}

#[cfg(feature = "enable")]
static WRITE_TARGET: std::sync::Mutex<WriteTarget> = std::sync::Mutex::new(WriteTarget::Log);

#[allow(unused_variables)]
pub fn set_write_target(wt: WriteTarget) {
    #[cfg(feature = "enable")]
    {
        *WRITE_TARGET.lock().unwrap() = wt;
    }
}

/// Logs location and hash of provided data
#[cfg(feature = "enable")]
#[macro_export]
macro_rules! packet_trace {
    ($location:expr, $payload:block) => {{
        $crate::helpers::do_write($location, $payload);
    }};
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
            $crate::packet_trace!($location, { payload })
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
#[cfg(feature = "enable")]
pub mod helpers {
    pub fn do_write(location: impl std::fmt::Display, payload: impl AsRef<[u8]>) {
        use crate::{WriteTarget, WRITE_TARGET};

        let sz = payload.as_ref().len();
        let hash = do_hash(payload);
        let ts = ts();

        match &mut *WRITE_TARGET.lock().unwrap() {
            WriteTarget::Log => {
                log::trace!(target: "packet-trace", "{},{:016x},{},{}", location, hash, ts, sz);
            }
            WriteTarget::Write(w) => {
                writeln!(w, "{},{:016x},{},{}", location, hash, ts, sz).unwrap();
            }
        }
    }

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
            if record.target() == "packet-trace" {
                let mut buf = self.0.lock().unwrap();
                writeln!(&mut buf, "{}", record.args()).unwrap()
            }
        }

        fn flush(&self) {}
    }

    impl std::io::Write for &StringLog {
        fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
            let text = std::str::from_utf8(buf).unwrap();
            let mut string = self.0.lock().unwrap();
            string.push_str(text);

            Ok(buf.len())
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
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
        let date = &output["test-date,0123456789abcdef,".len()..output.len() - ",3\n".len()];

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

    #[cfg(feature = "enable")]
    #[test]
    #[serial]
    pub fn test_custom_write_target() {
        use crate::{set_write_target, WriteTarget};

        StringLog::clear();

        set_write_target(WriteTarget::Write(Box::new(StringLog::global())));
        packet_trace!("test-foo", { &[1, 2, 3] });
        set_write_target(WriteTarget::Log);

        let output = StringLog::get_string();

        let expected = Regex::new(r#"test-foo,[0-9A-Fa-f]{16}.*\n"#).unwrap();
        assert!(expected.is_match(&output));

        let date = &output["test-date,0123456789abcdef,".len()..output.len() - ",3\n".len()];
        assert!(chrono::DateTime::parse_from_str(&date, DATE_FORMAT_STR).is_ok());
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
