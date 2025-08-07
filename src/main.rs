use crossterm::event::{Event, KeyCode, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, size};
use crossterm::{
    cursor, execute,
    style::{self, Color, Stylize},
    terminal::{Clear, ClearType},
};
use pad::PadStr;
use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

macro_rules! add_clean_entry {
    ($clean_entries:expr, $username:expr, $(
        ($path:expr, $desc:expr),
    )+) => {
        $($clean_entries.push(CleanEntry {
            path: format!("/Users/{}/{}", $username, $path),
            description: $desc.into(),
            score: 1.0,
            size: None,
            selected: false,
        });)+
    };
}

// 定义一个函数来格式化文件大小
fn format_size(size: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * 1024;
    const GIB: u64 = MIB * 1024;
    const TIB: u64 = GIB * 1024;

    if size < KIB {
        format!("{size} B")
    } else if size < MIB {
        format!("{:.2} KiB", size as f64 / KIB as f64)
    } else if size < GIB {
        format!("{:.2} MiB", size as f64 / MIB as f64)
    } else if size < TIB {
        format!("{:.2} GiB", size as f64 / GIB as f64)
    } else {
        format!("{:.2} TiB", size as f64 / TIB as f64)
    }
}

fn check_size(path: &Path) -> Result<u64, io::Error> {
    if !path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "错误: 路径 '{}' 不存在。",
                path.as_os_str().to_string_lossy()
            ),
        ));
    }
    if !path.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::NotADirectory,
            format!(
                "错误: 路径 '{}' 不是一个目录。",
                path.as_os_str().to_string_lossy()
            ),
        ));
    }

    let mut total_size: u64 = 0;
    for entry in WalkDir::new(path) {
        match entry {
            Ok(entry) => {
                let metadata = entry.metadata();
                match metadata {
                    Ok(metadata) => {
                        if metadata.is_file() {
                            total_size += metadata.len();
                        }
                    }
                    Err(e) => {
                        eprintln!("警告: 无法获取文件元数据 '{:?}': {}", entry.path(), e);
                    }
                }
            }
            Err(e) => {
                eprintln!("警告: 遍历目录时出错: {e}");
            }
        }
    }

    Ok(total_size)
}

fn clean_electron(clean_entries: &mut Vec<CleanEntry>, root: String, app: &str) {
    clean_entries.push(CleanEntry {
        path: format!("{root}/Cache"),
        description: format!("{app} 缓存"),
        score: 1.0,
        size: None,
        selected: false,
    });
    clean_entries.push(CleanEntry {
        path: format!("{root}/Code Cache"),
        description: format!("{app} 缓存"),
        score: 1.0,
        size: None,
        selected: false,
    });
    clean_entries.push(CleanEntry {
        path: format!("{root}/GPUCache"),
        description: format!("{app} 缓存"),
        score: 1.0,
        size: None,
        selected: false,
    });
    clean_entries.push(CleanEntry {
        path: format!("{root}/page_cache"),
        description: format!("{app} 缓存"),
        score: 1.0,
        size: None,
        selected: false,
    });
    clean_entries.push(CleanEntry {
        path: format!("{root}/logs"),
        description: format!("{app} 日志"),
        score: 1.0,
        size: None,
        selected: false,
    });
}

fn clean_jetbrains(clean_entries: &mut Vec<CleanEntry>, root: String) {
    // ("PyCharm", "2024.3")
    let mut dirs: Vec<(String, String)> = vec![];
    if let Ok(read_dir) = std::fs::read_dir(&root) {
        for entry in read_dir {
            if let Ok(entry) = entry {
                if entry.metadata().is_ok() && entry.metadata().unwrap().is_dir() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    let mut app_name = String::new();
                    for c in name.chars() {
                        if c.is_alphabetic() {
                            app_name.push(c);
                        } else {
                            break;
                        }
                    }
                    let version = name[app_name.len()..].to_string();
                    if !version.is_empty() && version.contains('.') {
                        dirs.push((app_name, version));
                    }
                }
            }
        }
    }
    let mut keep_dirs: HashMap<String, String> = HashMap::new();
    for (app_name, version) in dirs {
        if let Some(old_version) = keep_dirs.get(&app_name) {
            if version > *old_version {
                clean_entries.push(CleanEntry {
                    path: format!("{root}/{app_name}{old_version}"),
                    description: format!("{app_name} 的旧版本 {old_version}"),
                    score: 0.8,
                    size: None,
                    selected: false,
                });
                keep_dirs.insert(app_name, version);
            } else {
                clean_entries.push(CleanEntry {
                    path: format!("{root}/{app_name}{version}"),
                    description: format!("{app_name} 的旧版本 {version}"),
                    score: 0.8,
                    size: None,
                    selected: false,
                });
            }
        } else {
            keep_dirs.insert(app_name, version);
        }
    }
    // Loop through the keep_dirs to clean the latest versions
    if root.contains("Caches") {
        for (app_name, version) in keep_dirs {
            let path = format!("{root}/{app_name}{version}");
            clean_entries.push(CleanEntry {
                path: format!("{path}/intellij-rust/crates-local-index-cargo-home"),
                description: format!("{app_name} 的 Rust 插件缓存"),
                score: 0.8,
                size: None,
                selected: false,
            });
            clean_entries.push(CleanEntry {
                path: format!("{path}/intellij-rust/macros"),
                description: format!("{app_name} 的 Rust 插件缓存"),
                score: 0.8,
                size: None,
                selected: false,
            });
            clean_entries.push(CleanEntry {
                path: format!("{path}/caches"),
                description: format!("{app_name} 的 IDE 缓存"),
                score: 0.8,
                size: None,
                selected: false,
            });
        }
    }
}

#[allow(unused)]
struct CleanEntry {
    path: String,
    description: String,
    score: f32,
    size: Option<u64>, // 缓存计算的大小
    selected: bool,    // 是否被选中
}

// 交互式UI状态
struct UIState {
    entries: Vec<CleanEntry>,
    current_index: u16,
    scroll_offset: u16,
    terminal_height: u16,
    terminal_width: u16,
    total_selected_size: u64,
    selected_count: usize,
    show_small_files: bool, // 是否显示小于10MB的文件
}

impl UIState {
    fn new(entries: Vec<CleanEntry>) -> io::Result<Self> {
        let (width, height) = size()?;
        Ok(UIState {
            entries,
            current_index: 0,
            scroll_offset: 0,
            terminal_height: height,
            terminal_width: width,
            total_selected_size: 0,
            selected_count: 0,
            show_small_files: false, // 默认隐藏小文件
        })
    }

    fn visible_height(&self) -> u16 {
        // 保留空间给标题和状态栏
        self.terminal_height.saturating_sub(4)
    }

    fn get_visible_entries(&self) -> Vec<(usize, &CleanEntry)> {
        if self.show_small_files {
            self.entries.iter().enumerate().collect()
        } else {
            self.entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| entry.size.unwrap_or(0) >= 10 * 1024 * 1024) // 10MB
                .collect()
        }
    }

    fn toggle_current_selection(&mut self) {
        let visible_entries = self.get_visible_entries();
        if let Some((actual_index, _)) = visible_entries.get(self.current_index as usize) {
            let actual_index = *actual_index;
            drop(visible_entries);
            if let Some(entry) = self.entries.get_mut(actual_index) {
                entry.selected = !entry.selected;
                if let Some(size) = entry.size {
                    if entry.selected {
                        self.total_selected_size += size;
                        self.selected_count += 1;
                    } else {
                        self.total_selected_size = self.total_selected_size.saturating_sub(size);
                        self.selected_count = self.selected_count.saturating_sub(1);
                    }
                }
            }
        }
    }

    fn select_all(&mut self) {
        self.total_selected_size = 0;
        self.selected_count = 0;
        for entry in &mut self.entries {
            // 只选择大于等于10MB的文件
            if entry.size.unwrap_or(0) >= 10 * 1024 * 1024 {
                entry.selected = true;
                if let Some(size) = entry.size {
                    self.total_selected_size += size;
                    self.selected_count += 1;
                }
            }
        }
    }

    fn deselect_all(&mut self) {
        for entry in &mut self.entries {
            entry.selected = false;
        }
        self.total_selected_size = 0;
        self.selected_count = 0;
    }

    fn toggle_small_files_display(&mut self) {
        self.show_small_files = !self.show_small_files;
        // 重置当前索引和滚动偏移
        self.current_index = 0;
        self.scroll_offset = 0;
    }

    fn move_up(&mut self) {
        if self.current_index > 0 {
            self.current_index -= 1;
            if self.current_index < self.scroll_offset {
                self.scroll_offset = self.current_index;
            }
        }
    }

    fn move_down(&mut self) {
        let visible_entries = self.get_visible_entries();
        if self.current_index < visible_entries.len().saturating_sub(1) as u16 {
            self.current_index += 1;
            let visible_height = self.visible_height();
            if self.current_index >= self.scroll_offset + visible_height {
                self.scroll_offset = self.current_index - visible_height + 1;
            }
        }
    }

    fn page_up(&mut self) {
        let page_size = self.visible_height();
        self.current_index = self.current_index.saturating_sub(page_size);
        self.scroll_offset = self.scroll_offset.saturating_sub(page_size);
    }

    fn page_down(&mut self) {
        let page_size = self.visible_height();
        let visible_entries = self.get_visible_entries();
        let max_index = visible_entries.len().saturating_sub(1) as u16;
        self.current_index = (self.current_index + page_size).min(max_index);

        let visible_height = self.visible_height();
        if self.current_index >= self.scroll_offset + visible_height {
            self.scroll_offset = self.current_index - visible_height + 1;
        }
    }

    fn get_selected_entries(&self) -> Vec<&CleanEntry> {
        self.entries.iter().filter(|entry| entry.selected).collect()
    }
}

// 渲染系统
fn render_ui(ui_state: &UIState) -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    // 渲染标题
    execute!(
        stdout,
        style::Print("MacLean - 系统清理工具".bold().with(Color::Cyan)),
        style::Print("\r\n"),
        style::Print(
            "方向键导航，空格选择，Enter删除，S切换小文件显示，Esc退出".with(Color::DarkGrey)
        ),
        style::Print("\r\n\r\n")
    )?;

    let visible_entries = ui_state.get_visible_entries();
    let visible_height = ui_state.visible_height() as usize;
    let start_index = ui_state.scroll_offset as usize;
    let end_index = (start_index + visible_height).min(visible_entries.len());

    // 渲染项目列表
    let items_display = &visible_entries[start_index..end_index];
    let desc_width = items_display
        .iter()
        .map(|(_, entry)| entry.description.len() as u16)
        .max()
        .unwrap_or(0);
    let size_width = items_display
        .iter()
        .map(|(_, entry)| entry.size.map_or(0, |s| format_size(s).len()) as u16)
        .max()
        .unwrap_or(0);
    let path_width = ui_state
        .terminal_width
        .saturating_sub(desc_width)
        .saturating_sub(size_width)
        .saturating_sub(10); // 10 for checkbox and padding
    for (i, (_, entry)) in items_display.iter().enumerate() {
        let display_index = start_index + i;
        let is_current = display_index as u16 == ui_state.current_index;

        // 判断文件大小是否小于10MB
        let is_small_file = entry.size.unwrap_or(0) < 10 * 1024 * 1024;

        // 选择状态指示符
        let checkbox = if entry.selected { "✓" } else { "□" };

        let checkbox_color = if entry.selected {
            Color::Green
        } else if is_small_file {
            Color::DarkGrey // 小文件用暗灰色
        } else {
            Color::White
        };

        // 截断和格式化路径
        let truncated_path = if entry.path.len() as u16 > path_width {
            format!(
                "...{}",
                &entry.path[entry
                    .path
                    .len()
                    .saturating_sub(path_width.saturating_sub(3) as usize)..]
            )
        } else {
            entry.path.clone()
        };

        // 当前行背景色和文字样式
        let (name_style, path_style, size_style) = (
            {
                let desc = entry.description.clone().pad_to_width(desc_width as usize);
                if is_current {
                    desc.black().on_white()
                } else if is_small_file {
                    desc.with(Color::DarkGrey)
                } else {
                    desc.white()
                }
            },
            if path_width > 7 {
                truncated_path
                    .pad_to_width(path_width as usize)
                    .with(Color::Cyan)
            } else {
                "".to_string().with(Color::Yellow)
            },
            format_size(entry.size.unwrap_or(0))
                .pad_to_width(size_width as usize)
                .with(Color::Yellow),
        );

        execute!(
            stdout,
            style::Print(format!("{checkbox} ").with(checkbox_color)),
            style::Print(format!("{name_style} ")),
            style::Print(format!("{size_style} ")),
            style::Print(format!("{path_style}")),
            style::Print("\r\n")
        )?;
    }

    // 渲染滚动指示器和文件显示状态
    let total_count = ui_state.entries.len();
    let visible_count = visible_entries.len();
    let small_files_count = if ui_state.show_small_files {
        0
    } else {
        total_count - visible_count
    };
    let scroll_info = if visible_entries.len() > visible_height {
        format!(
            "第 {}-{} 项，共 {} 项可见",
            start_index + 1,
            end_index,
            visible_count
        )
    } else {
        "".to_string()
    };

    // 显示文件过滤状态
    let filter_info = if ui_state.show_small_files {
        if small_files_count > 0 {
            format!("显示全部文件 (包含 {small_files_count} 个小文件)")
        } else {
            "显示全部文件".to_string()
        }
    } else {
        format!("隐藏小于10MB文件 ({small_files_count} 个已隐藏)")
    };

    execute!(
        stdout,
        cursor::MoveTo(0, (ui_state.terminal_height - 1) as u16),
        style::Print(filter_info.with(Color::DarkGrey))
    )?;

    // 渲染状态栏
    let status = format!(
        "{} 已选择: {} 项, 总大小: {} | Ctrl+A:全选 Ctrl+D:取消全选 S:切换小文件",
        scroll_info,
        ui_state.selected_count,
        format_size(ui_state.total_selected_size)
    );
    execute!(
        stdout,
        cursor::MoveTo(0, ui_state.terminal_height as u16),
        style::Print(status.with(Color::Blue).bold())
    )?;

    stdout.flush()?;
    Ok(())
}

// 确认对话框
fn show_confirmation_dialog(selected_entries: &[&CleanEntry]) -> io::Result<bool> {
    let mut stdout = io::stdout();

    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    let total_size: u64 = selected_entries.iter().filter_map(|entry| entry.size).sum();

    execute!(
        stdout,
        style::Print("确认删除".red().bold()),
        style::Print("\r\n\r\n"),
        style::Print(format!(
            "即将删除 {} 个项目，总大小: {}\r\n",
            selected_entries.len(),
            format_size(total_size)
        )),
        style::Print("删除的项目:\r\n".yellow())
    )?;

    for entry in selected_entries.iter().take(10) {
        execute!(stdout, style::Print(format!("• {}\r\n", entry.description)))?;
    }

    if selected_entries.len() > 10 {
        execute!(
            stdout,
            style::Print(format!("... 还有 {} 个项目\n", selected_entries.len() - 10))
        )?;
    }

    execute!(
        stdout,
        style::Print("\r\n"),
        style::Print("确定要删除这些文件吗？".red().bold()),
        style::Print("\r\n"),
        style::Print("Y/Enter: 确认删除    N/Esc: 取消".with(Color::DarkGrey))
    )?;

    stdout.flush()?;

    loop {
        if let Event::Key(key_event) = crossterm::event::read()? {
            match key_event.code {
                KeyCode::Char('y') | KeyCode::Char('Y') | KeyCode::Enter => return Ok(true),
                KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => return Ok(false),
                _ => {}
            }
        }
    }
}

// 执行删除操作
fn execute_cleanup(selected_entries: &[&CleanEntry]) -> io::Result<()> {
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    execute!(
        stdout,
        style::Print("正在清理文件...".green().bold()),
        style::Print("\r\n\r\n")
    )?;

    let mut success_count = 0;
    let mut error_count = 0;

    for (index, entry) in selected_entries.iter().enumerate() {
        let progress = ((index + 1) as f32 / selected_entries.len() as f32 * 100.0) as u32;

        execute!(
            stdout,
            cursor::MoveTo(0, 2),
            Clear(ClearType::FromCursorDown),
            style::Print(format!(
                "进度: [{}%] {}/{}\r\n",
                progress,
                index + 1,
                selected_entries.len()
            )),
            style::Print(format!("正在删除: {}\r\n", entry.description))
        )?;
        stdout.flush()?;

        let result = std::fs::remove_dir_all(&entry.path);
        match result {
            Ok(_) => {
                success_count += 1;
                execute!(
                    stdout,
                    style::Print(format!("✓ 删除成功: {}\r\n", entry.description).green())
                )?;
            }
            Err(e) => {
                if e.kind() != io::ErrorKind::NotFound {
                    error_count += 1;
                    execute!(
                        stdout,
                        style::Print(
                            format!("✗ 删除失败: {} - {}\r\n", entry.description, e).red()
                        )
                    )?;
                } else {
                    success_count += 1; // 文件不存在也算成功
                }
            }
        }
    }

    execute!(
        stdout,
        style::Print("\r\n"),
        style::Print(format!("清理完成！成功: {success_count}, 失败: {error_count}").bold()),
        style::Print("\r\n\r\n按任意键退出...")
    )?;
    stdout.flush()?;

    // 等待用户按键
    crossterm::event::read()?;
    Ok(())
}

fn main() -> io::Result<()> {
    let _args: Vec<String> = env::args().collect();
    let started = std::time::Instant::now();

    let username = whoami::username();
    println!("当前用户：{username}");

    let mut clean_entries: Vec<CleanEntry> = Vec::with_capacity(100);

    #[cfg(target_os = "macos")]
    add_clean_entry! {
        clean_entries, username,
        ("Library/Caches/Microsoft Edge", "Microsoft Edge 缓存"),
        ("Library/Caches/Google/Chrome", "Google Chrome 缓存"),
        ("Library/Caches/Google/Jib", "Google Jib 缓存"),
        ("Library/Caches/com.hnc.Discord.ShipIt", "Discord 自动更新缓存"),
        ("Library/Caches/ms-playwright", "Playwright 缓存"),
        ("Library/Caches/Homebrew/downloads", "Homebrew 下载缓存"),
        ("Library/Containers/com.microsoft.onenote.mac/Data/Library/Logs", "OneNote 日志"),
        ("Library/Containers/com.microsoft.Powerpoint/Data/Library/Logs", "PowerPoint 日志"),
        ("Library/Containers/com.shangguanyangguang.MyZip/Data/tmp", "MyZip 临时文件"),
        ("Library/Containers/com.netease.163music/Data/Library/Caches", "网易云音乐缓存"),
        ("Library/Caches/Yarn", "Yarn (yarnpkg) 缓存"),
        ("Library/Caches/electron", "未知来源 electron 二进制缓存"),
        ("Library/Application Support/Microsoft/EdgeUpdater", "Microsoft Edge 自动更新"),
        ("Library/Containers/com.tencent.qq/Data/Library/Record", "QQ 录屏文件"),
        ("Library/Group Containers/UBF8T346G9.OneDriveStandaloneSuite/FileProviderLogs", "OneDrive 日志"),
        ("Library/Logs/OneDrive", "OneDrive 日志"),
        ("Library/Containers/com.apple.mediaanalysisd/Data/Library/Caches", "mediaanalysisd 缓存"),
        ("Library/Containers/com.tencent.meeting/Data/Library/Global/Data/DynamicResourcePackage",
        "腾讯会议下载缓存"),
        ("Library/Application Support/Caches", "不知道什么应用的缓存"),
        ("Library/Containers/com.tencent.meeting/Data/Library/Global/Logs", "腾讯会议日志"),
        ("Library/Application Support/Adobe/Common/Media Cache Files", "Adobe Media Cache"),
        ("Library/Application Support/Adobe/Common/Media Cache", "Adobe Media Cache"),
        ("Library/Application Support/zoom.us/AutoUpdater", "Zoom 自动更新"),
        ("Library/Logs/JetBrains", "JetBrains 日志"),
    }
    #[cfg(target_os = "macos")]
    {
        #[cfg(feature = "experimental")]
        add_clean_entry!(
            clean_entries,
            "Library/Caches/typescript",
            "typescript 缓存"
        );
        clean_electron(
            &mut clean_entries,
            format!("/Users/{username}/Library/Application Support/Code"),
            "VSCode",
        );
        clean_electron(
            &mut clean_entries,
            format!("/Users/{username}/Library/Application Support/Code - Insiders"),
            "VSCode - Insiders",
        );
        clean_electron(
            &mut clean_entries,
            format!("/Users/{username}/Library/Application Support/discord"),
            "Discord",
        );
        clean_electron(
            &mut clean_entries,
            format!("/Users/{username}/Library/Application Support/Notion/Partitions/notion"),
            "Notion",
        );
        clean_electron(
            &mut clean_entries,
            format!("/Users/{username}/Library/Application Support/cnkiexpress"),
            "中国知网express",
        );
        clean_electron(
            &mut clean_entries,
            format!("/Users/{username}/Library/Application Support/quark-cloud-drive"),
            "夸克网盘",
        );
        clean_idea_projects(
            &mut clean_entries,
            &format!("/Users/{username}/IdeaProjects"),
        )?;
        #[cfg(feature = "experimental")]
        if let Ok(read_dir) = std::fs::read_dir(format!("/Users/{username}/WebstormProjects")) {
            for entry in read_dir {
                if let Ok(entry) = entry {
                    if entry.metadata()?.is_dir() {
                        walk_and_delete(
                            &mut clean_entries,
                            ["node_modules"],
                            entry.path(),
                            30 * 24 * 60 * 60,
                        )
                    }
                }
            }
        }
        if let Ok(read_dir) = std::fs::read_dir(format!(
            "/Users/{username}/Library/Containers/com.tencent.qq/Data/Library/Application Support/QQ"
        )) {
            for entry in read_dir {
                if let Ok(entry) = entry {
                    if entry.metadata()?.is_dir()
                        && entry.file_name().to_string_lossy().starts_with("nt_qq")
                    {
                        let mut nt_data = entry.path().clone();
                        nt_data.push("nt_data");
                        if nt_data.exists() {
                            let mut log = nt_data.clone();
                            log.push("log");
                            clean_entries.push(CleanEntry {
                                path: log.to_string_lossy().into(),
                                description: "QQ 日志".into(),
                                score: 1.0,
                                size: None,
                                selected: false,
                            });
                        }
                    }
                }
            }
        }
        clean_jetbrains(
            &mut clean_entries,
            format!("/Users/{username}/Library/Application Support/JetBrains"),
        );
        clean_jetbrains(
            &mut clean_entries,
            format!("/Users/{username}/Library/Caches/JetBrains"),
        );
    }

    println!("扫描时间: {:?}", started.elapsed());
    println!("开始扫描磁盘空间占用情况...\n");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    // 扫描阶段 - 计算所有条目的大小
    let total_entries = clean_entries.len();
    for (index, entry) in clean_entries.iter_mut().enumerate() {
        execute!(
            stdout,
            cursor::SavePosition,
            cursor::MoveTo(0, 0),
            Clear(ClearType::CurrentLine),
            style::Print(
                format!(
                    "扫描进度: [{}/{}] {}% - 当前扫描: {}",
                    index + 1,
                    total_entries,
                    ((index + 1) as f32 / total_entries as f32 * 100.0) as u32,
                    &entry.description
                )
                .with(Color::Green)
            ),
            cursor::RestorePosition
        )?;
        stdout.flush()?;

        // 计算目录大小并存储到条目中
        let path = Path::new(&entry.path);
        if let Ok(size) = check_size(path) {
            entry.size = Some(size);
            entry.selected = false; // 初始状态为未选中
        }
    }

    // 过滤掉没有大小或大小为0的条目
    clean_entries.retain(|entry| entry.size.is_some() && entry.size.unwrap() > 0);

    if clean_entries.is_empty() {
        execute!(stdout, Clear(ClearType::All))?;
        disable_raw_mode()?;
        println!("没有找到可清理的文件！");
        return Ok(());
    }

    // 按大小排序（从大到小）
    clean_entries.sort_by(|a, b| b.size.unwrap_or(0).cmp(&a.size.unwrap_or(0)));

    // 创建UI状态
    let mut ui_state = UIState::new(clean_entries)?;

    // 主交互循环
    loop {
        render_ui(&ui_state)?;

        match crossterm::event::read()? {
            Event::Key(key_event) => {
                match key_event.code {
                    // 导航控制
                    KeyCode::Up => ui_state.move_up(),
                    KeyCode::Down => ui_state.move_down(),
                    KeyCode::PageUp => ui_state.page_up(),
                    KeyCode::PageDown => ui_state.page_down(),

                    // 选择控制
                    KeyCode::Char(' ') => ui_state.toggle_current_selection(),

                    // 全选/取消全选
                    KeyCode::Char('a') | KeyCode::Char('A')
                        if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        ui_state.select_all();
                    }
                    KeyCode::Char('d') | KeyCode::Char('D')
                        if key_event.modifiers.contains(KeyModifiers::CONTROL) =>
                    {
                        ui_state.deselect_all();
                    }

                    // 切换小文件显示
                    KeyCode::Char('s') | KeyCode::Char('S') => {
                        ui_state.toggle_small_files_display();
                    }

                    // 确认删除
                    KeyCode::Enter => {
                        let selected_entries = ui_state.get_selected_entries();
                        if selected_entries.is_empty() {
                            continue; // 没有选中任何项目
                        }

                        // 显示确认对话框
                        if show_confirmation_dialog(&selected_entries)? {
                            // 执行删除
                            execute_cleanup(&selected_entries)?;
                            break;
                        }
                        // 如果取消删除，继续显示主界面
                    }

                    // 退出程序
                    KeyCode::Esc | KeyCode::Char('q') => break,

                    _ => {} // 忽略其他按键
                }
            }
            Event::Resize(width, height) => {
                // 更新终端大小
                ui_state.terminal_width = width;
                ui_state.terminal_height = height;
            }
            _ => {}
        }
    }

    disable_raw_mode()?;
    Ok(())
}

fn clean_idea_projects(clean_entries: &mut Vec<CleanEntry>, path: &str) -> io::Result<()> {
    let read_dir = std::fs::read_dir(path)?;
    println!("正在清理 IntelliJ IDEA 项目目录: {path}");

    for entry in read_dir {
        if let Ok(entry) = entry {
            // Skip error
            if entry.metadata()?.is_dir() {
                walk_and_delete(
                    clean_entries,
                    [".gradle", "out", "build"],
                    entry.path(),
                    30 * 24 * 60 * 60,
                )
            }
        }
    }
    Ok(())
}

fn walk_and_delete<const N: usize>(
    clean_entries: &mut Vec<CleanEntry>,
    to_delete: [&str; N],
    root: PathBuf,
    // should be 30 days
    time_unused: u64,
) {
    for entry in WalkDir::new(&root).max_depth(2).into_iter().flatten() {
        if let Ok(metadata) = entry.metadata() {
            if metadata.is_dir() {
                let name = entry.file_name().to_str().unwrap_or("");
                if to_delete.contains(&name) {
                    let mut modified_time = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                    let mut access_time = metadata.accessed().unwrap_or(SystemTime::UNIX_EPOCH);
                    let mut created_time = metadata.created().unwrap_or(SystemTime::UNIX_EPOCH);
                    for entry_in_to_del in WalkDir::new(entry.path())
                        .max_depth(5)
                        .into_iter()
                        .flatten()
                    {
                        if let Ok(metadata) = entry_in_to_del.metadata() {
                            if let Ok(modified) = metadata.modified() {
                                if modified > modified_time {
                                    modified_time = modified;
                                }
                            }
                            if let Ok(accessed) = metadata.accessed() {
                                if accessed > access_time {
                                    access_time = accessed;
                                }
                            }
                            if let Ok(created) = metadata.created() {
                                if created > created_time {
                                    created_time = created;
                                }
                            }
                        }
                    }
                    let now = SystemTime::now();
                    let modified_duration = now.duration_since(modified_time).unwrap_or_default();
                    let access_duration = now.duration_since(access_time).unwrap_or_default();
                    let created_duration = now.duration_since(created_time).unwrap_or_default();
                    if modified_duration.as_secs() > time_unused
                        || access_duration.as_secs() > time_unused
                        || created_duration.as_secs() > time_unused
                    {
                        clean_entries.push(CleanEntry {
                            path: entry.path().to_string_lossy().into(),
                            description: format!(
                                "{} 中长时间不用的 {}",
                                entry
                                    .path()
                                    .parent()
                                    .unwrap()
                                    .file_name()
                                    .unwrap()
                                    .to_string_lossy(),
                                name
                            ),
                            score: 0.8,
                            size: None,
                            selected: false,
                        });
                    }
                }
            }
        }
    }
}
