//! Application errors with Vietnamese, fix-oriented messages.

/// Every failure the CLI can surface to the user.
#[derive(Debug, PartialEq, Eq)]
pub enum AppError {
    /// Unknown subcommand.
    UnknownCommand(String),
    /// No saved results yet.
    NoState,
    /// A command that needs indices was given none.
    NoArgs(String),
    /// An argument was not a valid 1-based index.
    BadIndex(String),
    /// A valid index pointed past the end of the list.
    IndexOutOfRange { got: usize, max: usize },
    /// A required external tool is not installed.
    MissingTool(String),
    /// Search was invoked without a query.
    EmptyQuery,
    /// The search tool ran but failed.
    SearchFailed(String),
    /// An I/O error (state file, spawning a process, ...).
    Io(String),
}

impl AppError {
    /// A human-facing message: what went wrong and how to fix it.
    pub fn message(&self) -> String {
        match self {
            AppError::UnknownCommand(c) => {
                format!("Lệnh không hợp lệ: '{c}'. Chạy 'ytb help' để xem hướng dẫn.")
            }
            AppError::NoState => {
                "Chưa có kết quả nào. Hãy tìm trước: ytb search <từ khoá>".to_string()
            }
            AppError::NoArgs(c) => {
                format!("Lệnh '{c}' cần ít nhất một số thứ tự. Ví dụ: ytb {c} 1 2 3")
            }
            AppError::BadIndex(arg) => {
                format!("Số thứ tự không hợp lệ: '{arg}'. Chỉ nhận số nguyên bắt đầu từ 1.")
            }
            AppError::IndexOutOfRange { got, max } => format!(
                "Số thứ tự {got} vượt quá danh sách (chỉ có 1..{max}). Chạy 'ytb list' để xem lại."
            ),
            AppError::MissingTool(t) => format!("Thiếu '{t}'. Cài bằng: brew install {t}"),
            AppError::EmptyQuery => {
                "Cần từ khoá tìm kiếm. Ví dụ: ytb search lofi hip hop".to_string()
            }
            AppError::SearchFailed(m) => format!("Tìm kiếm thất bại: {m}"),
            AppError::Io(m) => format!("Lỗi hệ thống: {m}"),
        }
    }

    /// Process exit code for this error (always non-zero).
    pub fn exit_code(&self) -> i32 {
        match self {
            AppError::UnknownCommand(_) => 2,
            AppError::NoArgs(_) | AppError::BadIndex(_) | AppError::EmptyQuery => 2,
            _ => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_tool_message_is_exact() {
        assert_eq!(
            AppError::MissingTool("mpv".to_string()).message(),
            "Thiếu 'mpv'. Cài bằng: brew install mpv"
        );
    }

    #[test]
    fn no_state_hints_search() {
        assert!(AppError::NoState.message().contains("ytb search"));
    }

    #[test]
    fn unknown_command_names_it() {
        let m = AppError::UnknownCommand("frobnicate".to_string()).message();
        assert!(m.contains("frobnicate"));
        assert!(m.contains("ytb help"));
    }

    #[test]
    fn usage_errors_exit_2() {
        assert_eq!(AppError::EmptyQuery.exit_code(), 2);
        assert_eq!(AppError::BadIndex("x".to_string()).exit_code(), 2);
        assert_eq!(AppError::UnknownCommand("x".to_string()).exit_code(), 2);
        assert_eq!(AppError::NoState.exit_code(), 1);
    }
}
