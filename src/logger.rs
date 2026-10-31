use std::{
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    sync::OnceLock,
};

use env_logger::Target;
use log::LevelFilter;

const MAX_LOG_BYTES: u64 = 10 * 1024 * 1024;
static LOGGER_INITIALIZED: OnceLock<()> = OnceLock::new();

struct MultiWriter {
    console: Box<dyn Write + Send>,
    file: Box<dyn Write + Send>,
}

impl MultiWriter {
    fn new(console: Box<dyn Write + Send>, file: Box<dyn Write + Send>) -> Self {
        Self { console, file }
    }
}

impl Write for MultiWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.console.write_all(bytes)?;
        self.file.write_all(bytes)?;
        Ok(bytes.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.console.flush()?;
        self.file.flush()
    }
}

fn default_log_path() -> PathBuf {
    home::home_dir()
        .map(|home| home.join(".zeedle.log"))
        .unwrap_or_else(|| PathBuf::from(".zeedle.log"))
}

fn should_rotate(file_size: u64) -> bool {
    file_size > MAX_LOG_BYTES
}

fn format_entry(
    timestamp: &str,
    level: &str,
    file: Option<&str>,
    line: Option<u32>,
    message: &str,
) -> String {
    format!(
        "[{timestamp} | {level} | {}:{}] --> {message}",
        file.unwrap_or("unknown"),
        line.unwrap_or(0)
    )
}

fn prepare_log_file(path: &Path) -> io::Result<fs::File> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if let Ok(metadata) = fs::metadata(path)
        && should_rotate(metadata.len())
    {
        fs::remove_file(path)?;
    }
    fs::OpenOptions::new().create(true).append(true).open(path)
}

fn build_logger(path: &Path, console: Box<dyn Write + Send>) -> io::Result<env_logger::Logger> {
    let file = prepare_log_file(path)?;
    let output = Box::new(MultiWriter::new(console, Box::new(file)));
    let mut builder = env_logger::Builder::new();
    builder
        .filter(None, LevelFilter::Info)
        .target(Target::Pipe(output))
        .format(|buffer, record| {
            let timestamp = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
            writeln!(
                buffer,
                "{}",
                format_entry(
                    &timestamp,
                    record.level().as_str(),
                    record.file(),
                    record.line(),
                    &record.args().to_string(),
                )
            )
        });
    Ok(builder.build())
}

pub fn init_default_logger(path: Option<&Path>) {
    LOGGER_INITIALIZED.get_or_init(|| {
        let path = path.map(Path::to_path_buf).unwrap_or_else(default_log_path);
        match build_logger(&path, Box::new(io::stdout())) {
            Ok(logger) => {
                if let Err(error) = log::set_boxed_logger(Box::new(logger)) {
                    eprintln!(
                        "could not initialize Zeedle logger at {}: {error}",
                        path.display()
                    );
                } else {
                    log::set_max_level(LevelFilter::Info);
                }
            }
            Err(error) => eprintln!("could not open Zeedle log {}: {error}", path.display()),
        }

        let previous_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            log::error!("panic: {info}");
            previous_hook(info);
        }));
    });
}

#[cfg(test)]
mod tests {
    use std::{
        io::{self, Write},
        sync::{Arc, Mutex},
    };

    use log::Log;

    use super::{MultiWriter, build_logger, format_entry, prepare_log_file, should_rotate};

    #[derive(Clone, Default)]
    struct SharedBuffer(Arc<Mutex<Vec<u8>>>);

    impl Write for SharedBuffer {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn formats_timestamp_level_and_source_location() {
        assert_eq!(
            format_entry(
                "2026-10-31 12:00:00",
                "WARN",
                Some("src/lib.rs"),
                Some(42),
                "bad file"
            ),
            "[2026-10-31 12:00:00 | WARN | src/lib.rs:42] --> bad file"
        );
    }

    #[test]
    fn substitutes_unknown_source_location() {
        assert_eq!(
            format_entry("time", "INFO", None, None, "started"),
            "[time | INFO | unknown:0] --> started"
        );
    }

    #[test]
    fn rotates_only_when_log_is_over_ten_mib() {
        assert!(!should_rotate(10 * 1024 * 1024));
        assert!(should_rotate(10 * 1024 * 1024 + 1));
    }

    #[test]
    fn writes_each_message_to_console_and_file() {
        let console = SharedBuffer::default();
        let file = SharedBuffer::default();
        let console_bytes = Arc::clone(&console.0);
        let file_bytes = Arc::clone(&file.0);
        let mut writer = MultiWriter::new(Box::new(console), Box::new(file));

        writer.write_all(b"line\n").unwrap();
        writer.flush().unwrap();

        assert_eq!(&*console_bytes.lock().unwrap(), b"line\n");
        assert_eq!(&*file_bytes.lock().unwrap(), b"line\n");
    }

    #[test]
    fn configured_logger_formats_and_writes_records_to_both_sinks() {
        let path =
            std::env::temp_dir().join(format!("zeedle-log-record-{}.log", std::process::id()));
        let console = SharedBuffer::default();
        let console_bytes = Arc::clone(&console.0);
        let logger = build_logger(&path, Box::new(console)).unwrap();
        let record = log::Record::builder()
            .args(format_args!("logger integration line"))
            .level(log::Level::Info)
            .file(Some("src/test.rs"))
            .line(Some(19))
            .build();

        logger.log(&record);
        logger.flush();

        let console_line = String::from_utf8(console_bytes.lock().unwrap().clone()).unwrap();
        let file_line = std::fs::read_to_string(&path).unwrap();
        let expected_tail = " | INFO | src/test.rs:19] --> logger integration line\n";
        assert!(console_line.starts_with('['));
        assert!(console_line.ends_with(expected_tail));
        assert!(file_line.starts_with('['));
        assert!(file_line.ends_with(expected_tail));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn clears_an_oversized_log_before_appending() {
        let path =
            std::env::temp_dir().join(format!("zeedle-log-rotate-{}.log", std::process::id()));
        let file = std::fs::File::create(&path).unwrap();
        file.set_len(10 * 1024 * 1024 + 1).unwrap();
        drop(file);

        let file = prepare_log_file(&path).unwrap();
        assert_eq!(file.metadata().unwrap().len(), 0);
        drop(file);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn keeps_a_log_at_the_rotation_limit() {
        let path = std::env::temp_dir().join(format!("zeedle-log-keep-{}.log", std::process::id()));
        let file = std::fs::File::create(&path).unwrap();
        file.set_len(10 * 1024 * 1024).unwrap();
        drop(file);

        let file = prepare_log_file(&path).unwrap();
        assert_eq!(file.metadata().unwrap().len(), 10 * 1024 * 1024);
        drop(file);
        let _ = std::fs::remove_file(path);
    }
}
