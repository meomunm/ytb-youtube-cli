//! ytb — a git-style YouTube CLI.
//!
//! Search once, then act on results by their 1-based number:
//!
//! ```text
//! ytb search lofi hip hop
//! ytb play 1 3        # queue #1 and #3 in mpv
//! ytb url 2 | pbcopy  # copy the URL of #2
//! ```

mod entry;
mod err;
mod index;
mod proc;
mod render;
mod search;
mod state;

use std::io::IsTerminal;

use err::AppError;

/// A tool-backed action over a set of selected entries.
enum Player {
    Play,
    Listen,
    Open,
    Dl,
    Mp3,
    Ascii,
}

/// Default terminal video-output driver for `ascii` (built into mpv, no extra
/// deps). Override with `YTB_ASCII_VO` (e.g. `caca`, `sixel`).
const DEFAULT_ASCII_VO: &str = "tct";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(run(&args));
}

/// Dispatch a parsed argv and return the process exit code.
fn run(args: &[String]) -> i32 {
    let Some(cmd) = args.first().map(String::as_str) else {
        print_help();
        return 0;
    };
    let rest = &args[1..];

    let result = match cmd {
        "help" | "-h" | "--help" => {
            print_help();
            return 0;
        }
        "search" | "s" => cmd_search(rest),
        "list" | "ls" | "l" => cmd_list(rest),
        "url" | "u" => cmd_url(rest),
        "play" | "p" => cmd_player("play", rest, Player::Play),
        "listen" | "a" => cmd_player("listen", rest, Player::Listen),
        "ascii" | "x" => cmd_player("ascii", rest, Player::Ascii),
        "open" | "o" => cmd_player("open", rest, Player::Open),
        "dl" | "d" | "download" => cmd_player("dl", rest, Player::Dl),
        "mp3" => cmd_player("mp3", rest, Player::Mp3),
        other => Err(AppError::UnknownCommand(other.to_string())),
    };

    match result {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{}", e.message());
            e.exit_code()
        }
    }
}

/// Colors on only when stdout is a TTY and `NO_COLOR` is unset.
fn use_color() -> bool {
    std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal()
}

/// The mpv video-output driver used by `ascii`, from `YTB_ASCII_VO`.
///
/// A blank or unset value uses [`DEFAULT_ASCII_VO`] (`tct`).
fn ascii_vo() -> String {
    ascii_vo_from(std::env::var("YTB_ASCII_VO").ok())
}

/// Pure core of [`ascii_vo`]: pick a driver from an optional raw value.
fn ascii_vo_from(raw: Option<String>) -> String {
    match raw {
        Some(v) if !v.trim().is_empty() => v.trim().to_string(),
        _ => DEFAULT_ASCII_VO.to_string(),
    }
}

/// `search <query...>` — fetch results, save them, and print the list.
fn cmd_search(rest: &[String]) -> Result<i32, AppError> {
    let query = rest.join(" ");
    let query = query.trim();
    if query.is_empty() {
        return Err(AppError::EmptyQuery);
    }
    if !proc::have("yt-dlp") {
        return Err(AppError::MissingTool("yt-dlp".to_string()));
    }

    let entries = search::run_search(query)?;
    if entries.is_empty() {
        // Keep the previous results rather than clobbering them with nothing.
        eprintln!("Không tìm thấy kết quả cho: {query}");
        return Ok(1);
    }

    state::write_state(&entries).map_err(|e| AppError::Io(e.to_string()))?;
    print!("{}", render::render_list(&entries, use_color()));
    Ok(0)
}

/// `list` — reprint the saved results.
fn cmd_list(_rest: &[String]) -> Result<i32, AppError> {
    let entries = state::read_state();
    if entries.is_empty() {
        return Err(AppError::NoState);
    }
    print!("{}", render::render_list(&entries, use_color()));
    Ok(0)
}

/// `url [n...]` — print watch URLs (all, or the selected ones) to stdout.
fn cmd_url(rest: &[String]) -> Result<i32, AppError> {
    let entries = state::read_state();
    if entries.is_empty() {
        return Err(AppError::NoState);
    }
    let picks: Vec<usize> = if rest.is_empty() {
        (1..=entries.len()).collect()
    } else {
        index::parse_all(rest, entries.len())?
    };
    for i in picks {
        println!("{}", entries[i - 1].watch_url());
    }
    Ok(0)
}

/// The shared body of `play`/`listen`/`open`/`dl`/`mp3`.
fn cmd_player(name: &str, rest: &[String], player: Player) -> Result<i32, AppError> {
    let entries = state::read_state();
    if entries.is_empty() {
        return Err(AppError::NoState);
    }
    if rest.is_empty() {
        return Err(AppError::NoArgs(name.to_string()));
    }

    // Validate EVERY index before doing anything (no partial execution).
    let picks = index::parse_all(rest, entries.len())?;
    let urls: Vec<String> = picks.iter().map(|&i| entries[i - 1].watch_url()).collect();

    let (bin, mut cmd_args, dep) = match player {
        Player::Play => ("mpv", vec![], "mpv"),
        Player::Listen => ("mpv", vec!["--no-video".to_string()], "mpv"),
        Player::Ascii => ("mpv", vec![format!("--vo={}", ascii_vo())], "mpv"),
        Player::Open => ("open", vec![], "open"),
        Player::Dl => (
            "yt-dlp",
            vec!["-o".to_string(), "%(title)s.%(ext)s".to_string()],
            "yt-dlp",
        ),
        Player::Mp3 => (
            "yt-dlp",
            vec![
                "-x".to_string(),
                "--audio-format".to_string(),
                "mp3".to_string(),
                "-o".to_string(),
                "%(title)s.%(ext)s".to_string(),
            ],
            "yt-dlp",
        ),
    };

    if !proc::have(dep) {
        return Err(AppError::MissingTool(dep.to_string()));
    }
    // mp3 extraction needs ffmpeg in addition to yt-dlp.
    if matches!(player, Player::Mp3) && !proc::have("ffmpeg") {
        return Err(AppError::MissingTool("ffmpeg".to_string()));
    }

    // One invocation with all URLs: preserves the mpv queue and typed order.
    cmd_args.extend(urls);
    proc::run_inherit(bin, &cmd_args)
}

/// Print usage to stdout.
fn print_help() {
    println!(
        "ytb — YouTube từ terminal, thao tác theo số thứ tự (kiểu git)

CÁCH DÙNG:
  ytb <lệnh> [số thứ tự...]

LỆNH:
  search, s <từ khoá>   Tìm video, lưu lại danh sách kết quả
  list,   ls, l         In lại danh sách kết quả gần nhất
  play,   p <n...>      Phát video bằng mpv (giữ hàng đợi)
  listen, a <n...>      Phát chỉ âm thanh (mpv --no-video)
  ascii,  x <n...>      Phát video dạng ký tự ngay trong terminal
  open,   o <n...>      Mở trên trình duyệt (macOS: open)
  url,    u [n...]      In URL ra stdout (không kèm gì → in tất cả)
  dl,     d <n...>      Tải video (yt-dlp)
  mp3       <n...>      Tải & chuyển sang mp3 (cần ffmpeg)
  help, -h, --help      Hiện trợ giúp này

VÍ DỤ:
  ytb search lofi hip hop
  ytb play 1 3 2
  ytb url 2 | pbcopy

BIẾN MÔI TRƯỜNG:
  YTB_COUNT      Số kết quả mỗi lần tìm (mặc định 20)
  YTB_ASCII_VO   Driver video của 'ascii' (mặc định tct; vd caca, sixel)
  NO_COLOR       Đặt bất kỳ giá trị nào để tắt màu

Danh sách kết quả lưu ở: ${{XDG_CACHE_HOME:-$HOME/.cache}}/ytb/results.tsv"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_args_prints_help_exit_0() {
        assert_eq!(run(&[]), 0);
    }

    #[test]
    fn help_exits_0() {
        assert_eq!(run(&["help".to_string()]), 0);
    }

    #[test]
    fn unknown_command_is_nonzero() {
        assert_eq!(run(&["frobnicate".to_string()]), 2);
    }

    #[test]
    fn ascii_vo_defaults_to_tct() {
        assert_eq!(ascii_vo_from(None), "tct");
        assert_eq!(ascii_vo_from(Some("".to_string())), "tct");
        assert_eq!(ascii_vo_from(Some("   ".to_string())), "tct");
    }

    #[test]
    fn ascii_vo_uses_override() {
        assert_eq!(ascii_vo_from(Some("caca".to_string())), "caca");
        assert_eq!(ascii_vo_from(Some("  sixel ".to_string())), "sixel");
    }
}
