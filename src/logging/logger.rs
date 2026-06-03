use std::io::Write;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::logging::level::LogLevel;

#[derive(Debug, Clone)]
pub struct LoggerConfig {
    pub file_path: PathBuf,
    pub max_file_size: u64,
    pub level: LogLevel,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            file_path: PathBuf::from("logs/rustyengine.log"),
            max_file_size: 5 * 1024 * 1024,
            level: LogLevel::Debug,
        }
    }
}

#[derive(Debug, Clone)]
pub struct LogMessage {
    pub level: LogLevel,
    pub target: String,
    pub message: String,
    pub timestamp_millis: u128,
}

impl LogMessage {
    pub fn new(level: LogLevel, target: impl Into<String>, message: impl Into<String>) -> Self {
        let timestamp_millis = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or_default();

        Self {
            level,
            target: target.into(),
            message: message.into(),
            timestamp_millis,
        }
    }
}

pub enum SinkMessage {
    Log(LogMessage),
    Flush,
    Shutdown,
}

pub trait LogSink: Send {
    fn write(&self, message: &LogMessage);
    fn flush(&self);
    fn shutdown(&self);
}

pub struct ConsoleSink;

impl ConsoleSink {
    pub fn new() -> Self {
        Self
    }
}

impl LogSink for ConsoleSink {
    fn write(&self, message: &LogMessage) {
        println!(
            "[{}] [{}] {}: {}",
            message.timestamp_millis,
            message.level,
            message.target,
            message.message
        );
    }

    fn flush(&self) {}
    fn shutdown(&self) {}
}

pub struct FileSink {
    path: PathBuf,
    max_size: u64,
    current_size: std::sync::Mutex<u64>,
}

impl FileSink {
    pub fn new(path: PathBuf, max_size: u64) -> Self {
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let initial_size = std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0);

        Self {
            path,
            max_size,
            current_size: std::sync::Mutex::new(initial_size),
        }
    }

    fn rollover(&self, message_size: u64) {
        let size = self.current_size.lock().unwrap();
        if *size + message_size > self.max_size && *size > 0 {
            drop(size);

            let appendix = format!(".{}", timestamp_now_millis());
            let mut roll_path = self.path.clone();
            let file_name = roll_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("log")
                .to_string();

            let mut new_name = file_name;
            new_name.push_str(&appendix);
            roll_path.set_file_name(new_name);

            let _ = std::fs::rename(&self.path, &roll_path);
            let _ = std::fs::File::create(&self.path);

            let mut size = self.current_size.lock().unwrap();
            *size = 0;
        }
    }

    pub fn path(&self) -> &PathBuf {
        &self.path
    }
}

impl LogSink for FileSink {
    fn write(&self, message: &LogMessage) {
        let formatted = format!(
            "[{}] [{}] {}: {}\n",
            message.timestamp_millis, message.level, message.target, message.message
        );
        let message_size = formatted.len() as u64;

        self.rollover(message_size);

        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let _ = file.write_all(formatted.as_bytes());
        }

        let mut size = self.current_size.lock().unwrap();
        *size += message_size;
    }

    fn flush(&self) {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let _ = std::io::Write::flush(&mut file);
        }
    }

    fn shutdown(&self) {}
}

pub struct Logger {
    config: LoggerConfig,
    sender: crossbeam_channel::Sender<SinkMessage>,
}

impl Logger {
    pub fn init(config: LoggerConfig) -> Arc<Self> {
        let _ = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&config.file_path);

        let sender = FileSinkWriter::start(&config);

        Arc::new(Self { config, sender })
    }

    pub fn console_only() -> Arc<Self> {
        let config = LoggerConfig::default();
        let sender = ConsoleOnlyWriter::start();
        Arc::new(Self { config, sender })
    }

    pub fn log(&self, level: LogLevel, target: impl Into<String>, message: impl Into<String>) {
        if level > self.config.level {
            return;
        }
        let message = SinkMessage::Log(LogMessage::new(level, target, message));
        let _ = self.sender.send(message);
    }

    pub fn debug(&self, target: impl Into<String>, message: impl Into<String>) {
        self.log(LogLevel::Debug, target, message);
    }

    pub fn info(&self, target: impl Into<String>, message: impl Into<String>) {
        self.log(LogLevel::Info, target, message);
    }

    pub fn warn(&self, target: impl Into<String>, message: impl Into<String>) {
        self.log(LogLevel::Warn, target, message);
    }

    pub fn error(&self, target: impl Into<String>, message: impl Into<String>) {
        self.log(LogLevel::Error, target, message);
    }

    pub fn flush(&self) {
        let _ = self.sender.send(SinkMessage::Flush);
    }

    pub fn shutdown(&self) {
        let _ = self.sender.send(SinkMessage::Shutdown);
    }

    pub fn config(&self) -> &LoggerConfig {
        &self.config
    }
}

impl Drop for Logger {
    fn drop(&mut self) {
        let _ = self.sender.send(SinkMessage::Flush);
        let _ = self.sender.send(SinkMessage::Shutdown);
    }
}

struct ConsoleOnlyWriter {
    receiver: crossbeam_channel::Receiver<SinkMessage>,
}

impl ConsoleOnlyWriter {
    fn start() -> crossbeam_channel::Sender<SinkMessage> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let writer = Self { receiver };
        std::thread::spawn(move || writer.run());
        sender
    }

    fn run(self) {
        let sink = ConsoleSink::new();
        while let Ok(message) = self.receiver.recv() {
            match message {
                SinkMessage::Log(log) => sink.write(&log),
                SinkMessage::Flush => sink.flush(),
                SinkMessage::Shutdown => break,
            }
        }
        sink.shutdown();
    }
}

struct FileSinkWriter {
    receiver: crossbeam_channel::Receiver<SinkMessage>,
    config: LoggerConfig,
}

impl FileSinkWriter {
    fn start(config: &LoggerConfig) -> crossbeam_channel::Sender<SinkMessage> {
        let (sender, receiver) = crossbeam_channel::unbounded();
        let config = config.clone();
        let writer = Self { receiver, config };
        std::thread::spawn(move || writer.run());
        sender
    }

    fn run(self) {
        let sink = FileSink::new(self.config.file_path.clone(), self.config.max_file_size);
        while let Ok(message) = self.receiver.recv() {
            match message {
                SinkMessage::Log(log) => sink.write(&log),
                SinkMessage::Flush => sink.flush(),
                SinkMessage::Shutdown => break,
            }
        }
        sink.shutdown();
    }
}

#[derive(Debug, Clone, Copy, Default)]
struct TimestampProvider;

impl TimestampProvider {
    fn now_millis(&self) -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or_default()
    }
}

fn timestamp_now_millis() -> u128 {
    TimestampProvider::default().now_millis()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::sync::OnceLock;

    static TEST_LOCK: OnceLock<()> = OnceLock::new();

    fn unique_test_path() -> std::path::PathBuf {
        let mut path = std::env::temp_dir();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        path.push(format!("rustyengine-logging-test-{ts}.log"));
        path
    }

    #[test]
    fn test_logger_console_only_creates_messages() {
        let logger = crate::logging::Logger::console_only();
        logger.info("console_only_test", "hello console");
    }

    #[test]
    fn test_logger_file_init_creates_path() {
        let _ = TEST_LOCK.get_or_init(|| {
            let _ = fs::remove_dir_all("logs/test-logs-keep");
            ();
        });

        let dir = unique_test_path();
        let _ = fs::remove_file(&dir);
        let config = LoggerConfig {
            file_path: dir.clone(),
            max_file_size: 500,
            level: LogLevel::Debug,
        };
        let logger = crate::logging::Logger::init(config);
        logger.info("file_init", "file_init_message");

        logger.flush();
        logger.shutdown();

        let _ = fs::remove_file(dir);
    }

    #[test]
    fn test_logger_shutdown_flushes_and_stops() {
        let config = LoggerConfig {
            file_path: unique_test_path(),
            ..LoggerConfig::default()
        };
        let logger = crate::logging::Logger::init(config);
        logger.info("shutdown_test", "shutdown_message");
        logger.shutdown();

        std::thread::sleep(std::time::Duration::from_millis(50));
    }
}
