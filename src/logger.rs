use anyhow::Result;
use log::kv::{Key, Value};
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::Error;
use std::io::Write;
use std::sync::Mutex;
use structured_logger::{Builder, Writer};

// A custom logger writer that writes both the log message and any associated
// fields to stdout.
#[derive(Debug)]
struct TerminalWriter<W: Write + Sync + Send + 'static> {
    writer: Mutex<RefCell<Box<W>>>,
}

impl<W: Write + Sync + Send + 'static> TerminalWriter<W> {
    fn new(writer: W) -> Self {
        TerminalWriter {
            writer: Mutex::new(RefCell::new(Box::new(writer))),
        }
    }

    fn new_writer(writer: W) -> Box<dyn Writer> {
        Box::new(TerminalWriter::new(writer))
    }
}

impl<W: Write + Sync + Send + 'static> Writer for TerminalWriter<W> {
    fn write_log(&self, value: &BTreeMap<Key<'_>, Value<'_>>) -> Result<(), Error> {
        // Some fields are essential and treated differently, so check we have them.

        let level = if let Some(level) = value.get("level") {
            level.to_string()
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "missing level field".to_string(),
            ));
        };

        let msg = if let Some(msg) = value.get("message") {
            msg.to_string()
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "missing message field".to_string(),
            ));
        };

        let timestamp = if let Some(timestamp) = value.get("timestamp") {
            timestamp.to_string()
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "missing timestamp field".to_string(),
            ));
        };

        // Gather up the remaining fields
        let mut fields = vec![];

        for (k, v) in value {
            match k.as_str() {
                "level" | "message" | "timestamp" => continue,
                _ => (),
            };

            let pair = format!("{}={}", k, v);

            fields.push(pair);
        }

        let summary = fields.join(" ");

        let fmt = format!("{}:{}: {}\t\t{}\n", timestamp, level, msg, summary);

        let w = self
            .writer
            .lock()
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

        if let Ok(mut w) = w.try_borrow_mut() {
            w.as_mut().write_all(fmt.as_bytes()).map_err(Error::from)?;
        } else {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "cannot access writer".to_string(),
            ));
        }

        Ok(())
    }
}

pub fn setup_logging(log_level: &str, use_json: bool) -> Result<()> {
    let writer = std::io::stdout();

    let structured_writer = match use_json {
        true => structured_logger::json::new_writer(writer),
        false => TerminalWriter::new_writer(writer),
    };

    Builder::with_level(log_level)
        .with_target_writer("*", structured_writer)
        .init();

    Ok(())
}
