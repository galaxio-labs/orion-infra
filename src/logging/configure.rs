use flexi_logger::{DeferredNow, FileSpec, FlexiLoggerError, style};
use std::io::Write;
use std::path::Path;
use std::str::FromStr;

use log::Record;
use serde_derive::{Deserialize, Serialize};

const DEFAULT_FILE: &str = "[unnamed]";
#[derive(PartialEq, Deserialize, Serialize, Clone, Debug)]
pub struct LogConf {
    pub level: String,
    pub output: Output,
    pub position: bool,
}
impl LogConf {
    pub fn alpha() -> Self {
        LogConf {
            level: "debug".to_string(),
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, Default)]
pub enum Output {
    #[default]
    Console,
    File(String),
}

impl Default for LogConf {
    fn default() -> Self {
        LogConf {
            level: String::from("warn,ctrl=info,launch=info"),
            output: Output::Console,
            position: false,
        }
    }
}

impl FromStr for LogConf {
    type Err = log::ParseLevelError;
    fn from_str(debug: &str) -> Result<Self, Self::Err> {
        Ok(LogConf {
            level: debug.to_string(),
            ..Default::default()
        })
    }
}

impl LogConf {
    pub fn new_console(level: &str) -> Self {
        LogConf {
            level: level.to_string(),
            output: Output::Console,
            ..Default::default()
        }
    }
    pub fn new_file(level: &str, path: &str) -> Self {
        LogConf {
            level: level.to_string(),
            output: Output::File(path.to_string()),
            ..Default::default()
        }
    }
    pub fn rec_positon(mut self) -> Self {
        self.position = true;
        self
    }
}

const MAX_LOG: usize = 100;

/*
// 新增静态变量
static INIT_ONCE: Once = Once::new();
type LogInitResult =Result<(), FlexiLoggerError>;
static mut INIT_RESULT: Option<LogInitResult> = None;

// 新增通用一次性执行函数
pub fn configure_once<F> (func: F) -> Option<LogInitResult>
where F: FnOnce() -> LogInitResult,
{
    unsafe {
        INIT_ONCE.call_once(|| {
            //func()?;
            INIT_RESULT = Some(func());
        });
        let init_out = INIT_RESULT.take();
        return init_out;

    }
    None
}
*/
pub fn configure_logging(conf: &LogConf) -> Result<(), FlexiLoggerError> {
    let logger = flexi_logger::Logger::try_with_str(conf.level.as_str())?
        // 设置日志展示演示，从左到右分别是ERROR、WARN、INFO、DEBUG、TRACE级别日志的颜色，数字范围[0..255], 数字前面加‘b’表示加粗
        .set_palette("b1;3;2;4;6".to_string());
    let logger = if conf.position {
        logger.format(log_format_position)
    } else {
        logger.format(log_format_normal)
    };

    match &conf.output {
        Output::Console => {
            logger.start()?;
        }
        Output::File(path) => {
            logger
                .log_to_file(
                    FileSpec::default()
                        .directory(path)
                        .basename("log")
                        .suppress_timestamp(),
                )
                .start()?;
        }
    }
    Ok(())
}

fn log_format_position(
    w: &mut dyn Write,
    _now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    let level = record.level();
    write!(
        w,
        "{:8} < {:20}> {:<15} {}",
        style(level).paint(format!("{level:<8}")),
        style(level).paint(format!("{:<20}", get_position(record))),
        style(level).paint(format!("{:<15}", get_target(record, 12))),
        style(level).paint(get_content(record)),
    )
}

fn log_format_normal(
    w: &mut dyn Write,
    _now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    let level = record.level();
    write!(
        w,
        "{:<8}-{:<25} {}",
        style(level).paint(format!("{level:<8}")),
        style(level).paint(format!("{:<25}", get_target(record, 20))),
        style(level).paint(get_content(record)),
    )
}

fn get_position(record: &Record) -> String {
    let mut target = match record.file() {
        None => DEFAULT_FILE.to_string(),
        Some(path) => {
            let mut file: Vec<&str> = path.split('/').collect();
            file.pop().unwrap_or(DEFAULT_FILE).to_string()
        }
    };

    if let Some(line) = record.line() {
        target = format!("{target}:{line}");
    }

    target
}

fn get_target(record: &Record, max: usize) -> String {
    let mut target = record.target().to_string();
    if target.len() > max {
        target = "...".into();
        let mut ancestors = Path::new(record.target()).ancestors();
        ancestors.next();
        ancestors.next();
        ancestors.next();
        if let Some(prefix) = ancestors.next()
            && let Ok(ends) = Path::new(record.target()).strip_prefix(prefix)
        {
            target = format!(".../{}", ends.to_str().unwrap_or(""));
        }
    }
    format!("[{target}]")
}

fn get_content(record: &Record) -> String {
    let data = record.args().to_string();
    let head = data.chars().take(MAX_LOG).collect::<String>();
    let others = data.chars().skip(MAX_LOG).collect::<String>();
    format!("{head}{others}")
}

#[cfg(test)]
mod tests {
    use super::*;

    macro_rules! test_record {
        (target = $target:expr, file = $file:expr, line = $line:expr) => {
            // `format_args!` temporaries only live for the enclosing statement, so the
            // record must be built and consumed in a single expression.
            log::Record::builder()
                .args(format_args!("payload"))
                .level(log::Level::Info)
                .target($target)
                .file($file)
                .line($line)
                .build()
        };
    }

    #[test]
    fn default_conf_logs_warnings_to_the_console_without_position() {
        let conf = LogConf::default();

        assert_eq!(conf.level, "warn,ctrl=info,launch=info");
        assert_eq!(conf.output, Output::Console);
        assert!(!conf.position);
    }

    #[test]
    fn output_defaults_to_console() {
        assert_eq!(Output::default(), Output::Console);
    }

    #[test]
    fn alpha_raises_the_level_to_debug() {
        assert_eq!(LogConf::alpha().level, "debug");
        assert_eq!(LogConf::alpha().output, Output::Console);
    }

    #[test]
    fn constructors_set_the_level_and_target() {
        let console = LogConf::new_console("info");
        assert_eq!(console.level, "info");
        assert_eq!(console.output, Output::Console);

        let file = LogConf::new_file("trace", "/var/log/gx");
        assert_eq!(file.level, "trace");
        assert_eq!(file.output, Output::File("/var/log/gx".to_string()));
    }

    #[test]
    fn rec_positon_switches_the_format_on() {
        assert!(!LogConf::new_console("info").position);
        assert!(LogConf::new_console("info").rec_positon().position);
    }

    #[test]
    fn from_str_uses_the_input_as_level() {
        let conf: LogConf = "debug".parse().expect("parse");

        assert_eq!(conf.level, "debug");
        assert_eq!(conf.output, Output::Console);
    }

    #[test]
    fn get_position_uses_the_file_name_and_line() {
        let rec = test_record!(
            target = "app",
            file = Some("src/alpha/beta.rs"),
            line = Some(42)
        );

        assert_eq!(get_position(&rec), "beta.rs:42");
    }

    #[test]
    fn get_position_omits_the_line_when_unknown() {
        let rec = test_record!(target = "app", file = Some("src/only_file.rs"), line = None);

        assert_eq!(get_position(&rec), "only_file.rs");
    }

    #[test]
    fn get_position_falls_back_when_the_file_is_missing() {
        let rec = test_record!(target = "app", file = None, line = None);

        assert_eq!(get_position(&rec), DEFAULT_FILE);
    }

    #[test]
    fn get_target_keeps_short_targets_intact() {
        let rec = test_record!(target = "infra::conf", file = None, line = None);

        assert_eq!(get_target(&rec, 20), "[infra::conf]");
    }

    #[test]
    fn get_target_shortens_slash_separated_targets() {
        let rec = test_record!(target = "a/b/c/d/e/f", file = None, line = None);

        assert_eq!(get_target(&rec, 3), "[.../d/e/f]");
    }

    #[test]
    fn get_target_collapses_long_module_paths() {
        // 模块路径用 `::` 分隔，没有 `/`，祖先链只剩自身，所以取不到可以缩短的前缀。
        let rec = test_record!(
            target = "orion_infra::logging::configure::inner",
            file = None,
            line = None
        );

        assert_eq!(get_target(&rec, 20), "[...]");
    }

    #[test]
    fn get_content_returns_the_record_payload() {
        let rec = test_record!(target = "app", file = None, line = None);

        assert_eq!(get_content(&rec), "payload");
    }

    #[test]
    fn configure_logging_propagates_an_invalid_spec_error() {
        // 含多个 '/' 的规格无法解析，应在启动 logger 前就报错，不会触碰进程级全局状态。
        let conf = LogConf::new_console("too/many/slashes");

        assert!(configure_logging(&conf).is_err());
    }
}
