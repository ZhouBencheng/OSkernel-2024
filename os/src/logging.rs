use log::{Level, LevelFilter, Metadata, Record, Log};

struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }
    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        let color = match record.level() {
            Level::Error => 31, // Red
            Level::Warn => 93,  // BrightYellow
            Level::Info => 34,  // Blue
            Level::Debug => 32, // Green
            Level::Trace => 90, // BrightBlack
        };
        println!( // 根据日志等级打印不同颜色的日志
            "\u{1B}[{}m[{:>5}] {}\u{1B}[0m",
            color,
            record.level(),
            record.args(),
        );
    }
    fn flush(&self) {}
}

/// 日志系统初始化函数
pub fn init() {
    static LOGGER: SimpleLogger = SimpleLogger;
    log::set_logger(&LOGGER).unwrap();
    log::set_max_level(match option_env!("LOG") { // 获取环境变量中LOG的值
        Some("TRACE") => LevelFilter::Trace,
        Some("INFO") => LevelFilter::Info,
        Some("WARN") => LevelFilter::Warn,
        Some("ERROR") => LevelFilter::Error,
        Some("DEGUB") => LevelFilter::Debug,
        _ => {println!("Failed to match a level"); LevelFilter::Off}
    });
}