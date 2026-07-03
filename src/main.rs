use clap::Parser;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::UNIX_EPOCH;

mod gencomp;

#[derive(Parser)]
#[command(name = "expls")]
#[command(about = "ls with extension-based colors and modification-time gradients")]
#[command(long_about = None)]
struct Cli {
    /// Show hidden files and directories (starting with '.')
    #[arg(short = 'a', long = "all")]
    all: bool,

    /// Target directory (default: current directory)
    #[arg(short = 'p', long = "path", default_value = ".")]
    path: String,

    /// Recursion depth for subdirectories
    #[arg(short = 'd', long = "depth")]
    depth: Option<u32>,

    /// Reverse gradient: old = dark, new = light
    #[arg(long = "reverse")]
    reverse: bool,

    /// Generate shell completion files into ./completions and exit (hidden)
    #[arg(long, hide = true, default_value_t = false)]
    pub completions: bool,
}

/// ディレクトリエントリ1件分の情報。
struct FileEntry {
    /// ファイル名（パスを含まない）。
    name: String,
    /// ディレクトリなら `true`。
    is_dir: bool,
    /// 小文字化した拡張子。ディレクトリは `"dir"`、拡張子なしは空文字列。
    extension: String,
    /// Unixエポック秒での最終更新時刻。取得失敗時は `0`。
    modified_secs: u64,
    /// 表示インデントのネスト深さ（0 = トップレベル）。
    indent: u32,
}

/// 拡張子に対応するHSLの色相値（0.0〜360.0）を返す。
///
/// 既知の拡張子には固有の色相を割り当て、同じ言語・カテゴリのファイルが
/// 視覚的に同系色でグループ化されるようにしている。
/// `"dir"` はディレクトリを表す内部キー。
/// 未知の拡張子には対応するhueがないため `None` を返す。
fn extension_hue(ext: &str) -> Option<f32> {
    Some(match ext {
        "rs" => 25.0,
        "py" => 215.0,
        "js" | "jsx" | "mjs" | "cjs" => 50.0,
        "ts" | "tsx" => 195.0,
        "md" | "mdx" => 175.0,
        "txt" => 165.0,
        "toml" | "yaml" | "yml" => 100.0,
        "json" | "jsonc" => 80.0,
        "sh" | "bash" | "zsh" | "fish" => 275.0,
        "html" | "htm" => 12.0,
        "css" | "scss" | "sass" | "less" => 305.0,
        "go" => 185.0,
        "c" | "h" => 355.0,
        "cpp" | "cc" | "cxx" | "hpp" | "hxx" => 5.0,
        "java" => 20.0,
        "kt" | "kts" => 60.0,
        "rb" => 350.0,
        "php" => 265.0,
        "swift" => 30.0,
        "hs" | "lhs" => 240.0,
        "lua" => 140.0,
        "vim" | "nvim" => 115.0,
        "sql" => 55.0,
        "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" | "ico" | "bmp" => 325.0,
        "mp4" | "mov" | "avi" | "mkv" | "webm" => 345.0,
        "mp3" | "wav" | "flac" | "ogg" | "aac" => 255.0,
        "zip" | "tar" | "gz" | "bz2" | "xz" | "7z" | "rar" | "zst" => 40.0,
        "pdf" => 10.0,
        "doc" | "docx" | "odt" => 230.0,
        "xls" | "xlsx" | "ods" | "csv" => 130.0,
        "ppt" | "pptx" | "odp" => 15.0,
        "ex" | "exs" => 290.0,
        "clj" | "cljs" | "cljc" => 170.0,
        "scala" => 355.0,
        "dart" => 200.0,
        "r" | "rmd" => 65.0,
        "tex" | "sty" | "cls" => 160.0,
        "lock" | "sum" => 160.0,
        "env" | "cfg" | "conf" | "ini" | "properties" => 155.0,
        "dir" => 220.0,
        _ => return None,
    })
}

/// HSL色空間の値をRGBに変換する。
///
/// - `h`: 色相（0.0〜360.0）
/// - `s`: 彩度（0.0〜1.0）
/// - `l`: 明度（0.0〜1.0）
///
/// 彩度がほぼ0の場合はグレースケールとして扱う。
/// 各チャンネルは0〜255の `u8` にクランプされる。
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    if s < 0.001 {
        let v = (l.clamp(0.0, 1.0) * 255.0) as u8;
        return (v, v, v);
    }
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let h1 = h / 60.0;
    let x = c * (1.0 - (h1 % 2.0 - 1.0).abs());
    let (r1, g1, b1) = match h1 as u32 {
        0 => (c, x, 0.0_f32),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    (
        ((r1 + m).clamp(0.0, 1.0) * 255.0) as u8,
        ((g1 + m).clamp(0.0, 1.0) * 255.0) as u8,
        ((b1 + m).clamp(0.0, 1.0) * 255.0) as u8,
    )
}

/// グラデーション係数 `t` をもとにファイルの表示色を返す。
///
/// - `t = 0.0`: 最新ファイル → 彩度高・明度低（濃い・鮮やか）
/// - `t = 1.0`: 最古ファイル → 彩度低・明度高（薄い・淡い）
///
/// `hue` が `Some` の場合はHSLでグラデーションを計算し、
/// `None`（未知の拡張子）の場合はグレースケール（100〜205）で返す。
fn gradient_color(hue: Option<f32>, t: f32) -> (u8, u8, u8) {
    match hue {
        Some(h) => {
            let s = 0.85 - t * 0.55; // 0.85 → 0.30
            let l = 0.45 + t * 0.30; // 0.45 → 0.75
            hsl_to_rgb(h, s, l)
        }
        None => {
            let v = (100.0 + t * 105.0) as u8; // dark gray → light gray
            (v, v, v)
        }
    }
}

/// テキストをANSI 24-bitカラーエスケープシーケンスで囲んで返す。
///
/// 出力形式: `\x1b[38;2;R;G;Bm{text}\x1b[0m`
fn colorize(text: &str, r: u8, g: u8, b: u8) -> String {
    format!("\x1b[38;2;{};{};{}m{}\x1b[0m", r, g, b, text)
}

/// `dir` 以下のエントリを再帰的に収集し `entries` に追記する。
///
/// - `show_hidden`: `false` のとき `.` で始まるエントリをスキップする。
/// - `max_depth`: `Some(n)` のとき `current_depth < n` の場合のみサブディレクトリへ再帰する。
///   `None` のとき再帰しない（トップレベルのみ列挙）。
/// - `current_depth`: 現在の再帰深さ。呼び出し元は `0` を渡す。
///
/// エントリはファイル名のアルファベット順にソートされる。
/// `dir` の読み取りに失敗した場合は標準エラーに警告を出して早期リターンする。
fn collect_entries(
    dir: &Path,
    show_hidden: bool,
    max_depth: Option<u32>,
    current_depth: u32,
    entries: &mut Vec<FileEntry>,
) {
    let read_dir = match fs::read_dir(dir) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("expls: {}: {}", dir.display(), e);
            return;
        }
    };

    let mut children: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
    children.sort_by_key(|e| e.file_name());

    for entry in children {
        let name = entry.file_name().to_string_lossy().to_string();
        if !show_hidden && name.starts_with('.') {
            continue;
        }

        let path = entry.path();
        let is_dir = path.is_dir();
        let extension = if is_dir {
            "dir".to_string()
        } else {
            path.extension()
                .map(|e| e.to_string_lossy().to_lowercase())
                .unwrap_or_default()
        };

        let modified_secs = path
            .metadata()
            .and_then(|m| m.modified())
            .map(|t| t.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs())
            .unwrap_or(0);

        entries.push(FileEntry {
            name,
            is_dir,
            extension,
            modified_secs,
            indent: current_depth,
        });

        if is_dir {
            if let Some(max) = max_depth {
                if current_depth < max {
                    collect_entries(&path, show_hidden, max_depth, current_depth + 1, entries);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::{Path, PathBuf};

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(name: &str) -> Self {
            let path = std::env::temp_dir().join(format!("expls_test_{}", name));
            fs::create_dir_all(&path).unwrap();
            TempDir { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.path);
        }
    }

    // --- extension_hue ---

    #[test]
    fn test_extension_hue_known() {
        assert_eq!(extension_hue("rs"), Some(25.0));
        assert_eq!(extension_hue("py"), Some(215.0));
        assert_eq!(extension_hue("js"), Some(50.0));
        assert_eq!(extension_hue("ts"), Some(195.0));
        assert_eq!(extension_hue("dir"), Some(220.0));
    }

    #[test]
    fn test_extension_hue_unknown_returns_none() {
        assert_eq!(extension_hue("xyz"), None);
        assert_eq!(extension_hue(""), None);
        assert_eq!(extension_hue("unknown"), None);
    }

    #[test]
    fn test_extension_hue_aliases_share_hue() {
        assert_eq!(extension_hue("jsx"), extension_hue("js"));
        assert_eq!(extension_hue("tsx"), extension_hue("ts"));
        assert_eq!(extension_hue("yaml"), extension_hue("yml"));
        assert_eq!(extension_hue("bash"), extension_hue("sh"));
    }

    // --- hsl_to_rgb ---

    #[test]
    fn test_hsl_to_rgb_primary_colors() {
        assert_eq!(hsl_to_rgb(0.0, 1.0, 0.5), (255, 0, 0));   // red
        assert_eq!(hsl_to_rgb(120.0, 1.0, 0.5), (0, 255, 0)); // green
        assert_eq!(hsl_to_rgb(240.0, 1.0, 0.5), (0, 0, 255)); // blue
    }

    #[test]
    fn test_hsl_to_rgb_secondary_colors() {
        assert_eq!(hsl_to_rgb(60.0, 1.0, 0.5), (255, 255, 0));   // yellow
        assert_eq!(hsl_to_rgb(180.0, 1.0, 0.5), (0, 255, 255));  // cyan
        assert_eq!(hsl_to_rgb(300.0, 1.0, 0.5), (255, 0, 255));  // magenta
    }

    #[test]
    fn test_hsl_to_rgb_achromatic() {
        assert_eq!(hsl_to_rgb(0.0, 0.0, 1.0), (255, 255, 255)); // white
        assert_eq!(hsl_to_rgb(0.0, 0.0, 0.0), (0, 0, 0));       // black
    }

    #[test]
    fn test_hsl_to_rgb_gray_is_equal_channels() {
        let (r, g, b) = hsl_to_rgb(180.0, 0.0, 0.5);
        assert_eq!(r, g);
        assert_eq!(g, b);
    }

    #[test]
    fn test_hsl_to_rgb_output_in_valid_range() {
        // arbitrary inputs should never panic or overflow
        for h in [0.0_f32, 90.0, 180.0, 270.0, 359.9] {
            for &s in &[0.0_f32, 0.5, 1.0] {
                for &l in &[0.0_f32, 0.5, 1.0] {
                    let _ = hsl_to_rgb(h, s, l); // must not panic
                }
            }
        }
    }

    // --- gradient_color ---

    #[test]
    fn test_gradient_color_newest_darker_than_oldest() {
        // t=0.0 (newest) → vivid, t=1.0 (oldest) → pale
        let (r0, g0, b0) = gradient_color(Some(120.0), 0.0);
        let (r1, g1, b1) = gradient_color(Some(120.0), 1.0);
        let brightness0 = r0 as u32 + g0 as u32 + b0 as u32;
        let brightness1 = r1 as u32 + g1 as u32 + b1 as u32;
        assert!(brightness0 < brightness1, "newest should be darker than oldest");
    }

    #[test]
    fn test_gradient_color_brightness_monotone() {
        // brightness must strictly increase as t goes 0→1
        let steps = 5usize;
        let mut prev = 0u32;
        for i in 0..steps {
            let t = i as f32 / (steps - 1) as f32;
            let (r, g, b) = gradient_color(Some(200.0), t);
            let brightness = r as u32 + g as u32 + b as u32;
            assert!(brightness > prev, "brightness should increase at t={}", t);
            prev = brightness;
        }
    }

    #[test]
    fn test_gradient_color_none_hue_is_gray() {
        let (r, g, b) = gradient_color(None, 0.5);
        assert_eq!(r, g);
        assert_eq!(g, b);
    }

    #[test]
    fn test_gradient_color_none_hue_endpoints() {
        // dark gray at t=0, light gray at t=1
        assert_eq!(gradient_color(None, 0.0), (100, 100, 100));
        assert_eq!(gradient_color(None, 1.0), (205, 205, 205));
    }

    // --- colorize ---

    #[test]
    fn test_colorize_exact_format() {
        assert_eq!(colorize("hi", 10, 20, 30), "\x1b[38;2;10;20;30mhi\x1b[0m");
    }

    #[test]
    fn test_colorize_contains_text() {
        let result = colorize("hello", 255, 128, 0);
        assert!(result.contains("hello"));
    }

    #[test]
    fn test_colorize_starts_with_escape() {
        let result = colorize("x", 0, 0, 0);
        assert!(result.starts_with("\x1b[38;2;"));
    }

    #[test]
    fn test_colorize_ends_with_reset() {
        let result = colorize("x", 0, 0, 0);
        assert!(result.ends_with("\x1b[0m"));
    }

    // --- collect_entries ---

    #[test]
    fn test_collect_entries_lists_files() {
        let tmp = TempDir::new("list_files");
        fs::write(tmp.path().join("a.rs"), "").unwrap();
        fs::write(tmp.path().join("b.py"), "").unwrap();

        let mut entries = Vec::new();
        collect_entries(tmp.path(), false, None, 0, &mut entries);

        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(entries.len(), 2);
        assert!(names.contains(&"a.rs"));
        assert!(names.contains(&"b.py"));
    }

    #[test]
    fn test_collect_entries_alphabetical_order() {
        let tmp = TempDir::new("alpha_order");
        for name in &["z.txt", "a.txt", "m.txt"] {
            fs::write(tmp.path().join(name), "").unwrap();
        }

        let mut entries = Vec::new();
        collect_entries(tmp.path(), false, None, 0, &mut entries);

        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(names, vec!["a.txt", "m.txt", "z.txt"]);
    }

    #[test]
    fn test_collect_entries_hides_dotfiles_by_default() {
        let tmp = TempDir::new("hide_dotfiles");
        fs::write(tmp.path().join("visible.txt"), "").unwrap();
        fs::write(tmp.path().join(".hidden"), "").unwrap();

        let mut entries = Vec::new();
        collect_entries(tmp.path(), false, None, 0, &mut entries);

        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].name, "visible.txt");
    }

    #[test]
    fn test_collect_entries_shows_dotfiles_with_all_flag() {
        let tmp = TempDir::new("show_dotfiles");
        fs::write(tmp.path().join("visible.txt"), "").unwrap();
        fs::write(tmp.path().join(".hidden"), "").unwrap();

        let mut entries = Vec::new();
        collect_entries(tmp.path(), true, None, 0, &mut entries);

        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn test_collect_entries_no_recursion_without_depth() {
        let tmp = TempDir::new("no_recurse");
        let sub = tmp.path().join("subdir");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("nested.rs"), "").unwrap();
        fs::write(tmp.path().join("top.rs"), "").unwrap();

        let mut entries = Vec::new();
        collect_entries(tmp.path(), false, None, 0, &mut entries);

        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert_eq!(entries.len(), 2);
        assert!(names.contains(&"top.rs"));
        assert!(names.contains(&"subdir"));
        assert!(!names.contains(&"nested.rs"));
    }

    #[test]
    fn test_collect_entries_depth_1_recurses_once() {
        let tmp = TempDir::new("depth1");
        let sub = tmp.path().join("subdir");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("nested.rs"), "").unwrap();
        fs::write(tmp.path().join("top.rs"), "").unwrap();

        let mut entries = Vec::new();
        collect_entries(tmp.path(), false, Some(1), 0, &mut entries);

        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"top.rs"));
        assert!(names.contains(&"subdir"));
        assert!(names.contains(&"nested.rs"));
    }

    #[test]
    fn test_collect_entries_depth_0_does_not_recurse() {
        let tmp = TempDir::new("depth0");
        let sub = tmp.path().join("subdir");
        fs::create_dir(&sub).unwrap();
        fs::write(sub.join("nested.rs"), "").unwrap();

        let mut entries = Vec::new();
        collect_entries(tmp.path(), false, Some(0), 0, &mut entries);

        let names: Vec<&str> = entries.iter().map(|e| e.name.as_str()).collect();
        assert!(names.contains(&"subdir"));
        assert!(!names.contains(&"nested.rs"));
    }

    #[test]
    fn test_collect_entries_extension_detection() {
        let tmp = TempDir::new("ext_detect");
        fs::write(tmp.path().join("main.rs"), "").unwrap();
        fs::write(tmp.path().join("script.py"), "").unwrap();
        fs::write(tmp.path().join("noext"), "").unwrap();

        let mut entries = Vec::new();
        collect_entries(tmp.path(), false, None, 0, &mut entries);

        let find = |name: &str| entries.iter().find(|e| e.name == name).unwrap();
        assert_eq!(find("main.rs").extension, "rs");
        assert_eq!(find("script.py").extension, "py");
        assert_eq!(find("noext").extension, "");
    }

    #[test]
    fn test_collect_entries_directory_extension_is_dir() {
        let tmp = TempDir::new("dir_ext");
        fs::create_dir(tmp.path().join("mydir")).unwrap();

        let mut entries = Vec::new();
        collect_entries(tmp.path(), false, None, 0, &mut entries);

        let dir_entry = entries.iter().find(|e| e.name == "mydir").unwrap();
        assert_eq!(dir_entry.extension, "dir");
        assert!(dir_entry.is_dir);
    }

    #[test]
    fn test_collect_entries_indent_reflects_depth() {
        let tmp = TempDir::new("indent_depth");
        let sub = tmp.path().join("sub");
        fs::create_dir(&sub).unwrap();
        let sub2 = sub.join("sub2");
        fs::create_dir(&sub2).unwrap();
        fs::write(sub2.join("deep.rs"), "").unwrap();

        let mut entries = Vec::new();
        collect_entries(tmp.path(), false, Some(2), 0, &mut entries);

        let find = |name: &str| entries.iter().find(|e| e.name == name).unwrap();
        assert_eq!(find("sub").indent, 0);
        assert_eq!(find("sub2").indent, 1);
        assert_eq!(find("deep.rs").indent, 2);
    }

    #[test]
    fn test_collect_entries_empty_dir() {
        let tmp = TempDir::new("empty_dir");
        let mut entries = Vec::new();
        collect_entries(tmp.path(), false, None, 0, &mut entries);
        assert!(entries.is_empty());
    }
}

fn main() {
    let cli = Cli::parse();

    // 隠しオプション --completions が指定されたら補完ファイルを生成して終了する。
    if cli.completions {
        gencomp::generate(Path::new("completions"));
        return;
    }

    let target = Path::new(&cli.path);
    if !target.exists() {
        eprintln!("expls: {}: No such file or directory", cli.path);
        std::process::exit(1);
    }
    if !target.is_dir() {
        eprintln!("expls: {}: Not a directory", cli.path);
        std::process::exit(1);
    }

    let mut entries: Vec<FileEntry> = Vec::new();
    collect_entries(target, cli.all, cli.depth, 0, &mut entries);

    // Group entry indices by extension, then compute gradient t per entry
    let mut ext_indices: HashMap<String, Vec<usize>> = HashMap::new();
    for (i, entry) in entries.iter().enumerate() {
        ext_indices.entry(entry.extension.clone()).or_default().push(i);
    }

    let mut entry_t: Vec<f32> = vec![0.0; entries.len()];
    for indices in ext_indices.values_mut() {
        // Sort newest first
        indices.sort_by(|&a, &b| entries[b].modified_secs.cmp(&entries[a].modified_secs));
        let n = indices.len();
        for (rank, &idx) in indices.iter().enumerate() {
            let t = if n > 1 { rank as f32 / (n - 1) as f32 } else { 0.0 };
            entry_t[idx] = if cli.reverse { 1.0 - t } else { t };
        }
    }

    for (i, entry) in entries.iter().enumerate() {
        let indent = "  ".repeat(entry.indent as usize);
        let hue = extension_hue(&entry.extension);
        let t = entry_t[i];
        let (r, g, b) = gradient_color(hue, t);
        let display_name = if entry.is_dir {
            format!("{}/", entry.name)
        } else {
            entry.name.clone()
        };
        println!("{}{}", indent, colorize(&display_name, r, g, b));
    }
}
