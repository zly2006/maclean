use crossterm::event::{Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{
    cursor, execute,
    style::{self, Color, Stylize},
    terminal::{Clear, ClearType},
};
use std::collections::HashMap;
use std::env;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use walkdir::WalkDir;

macro_rules! add_clean_entry {
    ($clean_entries:expr, $path:expr, $desc:expr) => {
        $clean_entries.push(CleanEntry {
            path: format!("/Users/{}/{}", whoami::username(), $path),
            description: $desc.into(),
            score: 1.0,
        });
    };
    ($clean_entries:expr, $path:expr, $desc:expr, $score:expr) => {
        $clean_entries.push(CleanEntry {
            path: format!("/Users/{}/{}", whoami::username(), $path),
            description: $desc.into(),
            score: $score,
        });
    };
    ($clean_entries:expr, $username:expr, $path:expr, $desc:expr, $score:expr) => {
        $clean_entries.push(CleanEntry {
            path: format!("/Users/{}/{}", $username, $path),
            description: $desc.into(),
            score: $score,
        });
    };
}

// 定义一个函数来格式化文件大小
fn format_size(size: u64) -> String {
    const KIB: u64 = 1024;
    const MIB: u64 = KIB * 1024;
    const GIB: u64 = MIB * 1024;
    const TIB: u64 = GIB * 1024;

    if size < KIB {
        format!("{} B", size)
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
                eprintln!("警告: 遍历目录时出错: {}", e);
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
    });
    clean_entries.push(CleanEntry {
        path: format!("{root}/Code Cache"),
        description: format!("{app} 缓存"),
        score: 1.0,
    });
    clean_entries.push(CleanEntry {
        path: format!("{root}/GPUCache"),
        description: format!("{app} 缓存"),
        score: 1.0,
    });
    clean_entries.push(CleanEntry {
        path: format!("{root}/page_cache"),
        description: format!("{app} 缓存"),
        score: 1.0,
    });
    clean_entries.push(CleanEntry {
        path: format!("{root}/logs"),
        description: format!("{app} 日志"),
        score: 1.0,
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
                    if version.len() > 0 && version.contains('.') {
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
                });
                keep_dirs.insert(app_name, version);
            } else {
                clean_entries.push(CleanEntry {
                    path: format!("{root}/{app_name}{version}"),
                    description: format!("{app_name} 的旧版本 {version}"),
                    score: 0.8,
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
            });
            clean_entries.push(CleanEntry {
                path: format!("{path}/intellij-rust/macros"),
                description: format!("{app_name} 的 Rust 插件缓存"),
                score: 0.8,
            });
            clean_entries.push(CleanEntry {
                path: format!("{path}/caches"),
                description: format!("{app_name} 的 IDE 缓存"),
                score: 0.8,
            });
        }
    }
}

#[allow(unused)]
struct CleanEntry {
    path: String,
    description: String,
    score: f32,
}

#[allow(unused)]
trait ToClean {
    fn path(&self) -> &String;
    fn description(&self) -> &String;
}

impl ToClean for CleanEntry {
    fn path(&self) -> &String {
        &self.path
    }

    fn description(&self) -> &String {
        &self.description
    }
}

fn main() -> io::Result<()> {
    let _args: Vec<String> = env::args().collect();
    let started = std::time::Instant::now();

    let username = whoami::username();
    println!("当前用户：{}", username);

    let mut clean_entries: Vec<CleanEntry> = Vec::with_capacity(100);
    #[cfg(target_os = "macos")]
    {
        add_clean_entry!(clean_entries, "Library/Application Support/Microsoft/EdgeUpdater", "Microsoft Edge 自动更新");
        add_clean_entry!(clean_entries, "Library/Containers/com.tencent.qq/Data/Library/Record", "QQ 录屏文件");
        add_clean_entry!(clean_entries, "Library/Group Containers/UBF8T346G9.OneDriveStandaloneSuite/FileProviderLogs", "OneDrive 日志");
        add_clean_entry!(clean_entries, "Library/Logs/OneDrive", "OneDrive 日志");
        add_clean_entry!(clean_entries, "Library/Containers/com.apple.mediaanalysisd/Data/Library/Caches", "mediaanalysisd 缓存");
        add_clean_entry!(clean_entries, "Library/Containers/com.tencent.meeting/Data/Library/Global/Data/DynamicResourcePackage", "腾讯会议下载缓存");
        add_clean_entry!(clean_entries, "Library/Application Support/Caches", "应用程序支持缓存");
        add_clean_entry!(clean_entries, "Library/Containers/com.tencent.meeting/Data/Library/Global", "腾讯会议全局数据");
        add_clean_entry!(clean_entries, "Library/Application Support/Adobe/Common/Media Cache Files", "Adobe Media Cache");
        add_clean_entry!(clean_entries, "Library/Application Support/Adobe/Common/Media Cache", "Adobe Media Cache");
        add_clean_entry!(clean_entries, "Library/Application Support/zoom.us/AutoUpdater", "Zoom 自动更新");
        add_clean_entry!(clean_entries, "Library/Logs/JetBrains", "JetBrains 日志");
        add_clean_entry!(clean_entries, "Library/Caches/Microsoft Edge", "Microsoft Edge 缓存");
        add_clean_entry!(clean_entries, "Library/Caches/Google/Chrome", "Google Chrome 缓存");
        add_clean_entry!(clean_entries, "Library/Caches/Google/Jib", "Google Jib 缓存");
        add_clean_entry!(clean_entries, "Library/Caches/com.hnc.Discord.ShipIt", "Discord 自动更新缓存");
        add_clean_entry!(clean_entries, "Library/Caches/ms-playwright", "Playwright 缓存");
        add_clean_entry!(clean_entries, "Library/Caches/Homebrew/downloads", "Homebrew 下载缓存");
        #[cfg(feature = "experimental")]
        add_clean_entry!(clean_entries, "Library/Caches/typescript", "typescript 缓存");
        add_clean_entry!(clean_entries, "Library/Caches/Yarn", "Yarn (yarnpkg) 缓存");
        add_clean_entry!(clean_entries, "Library/Caches/electron", "未知来源 electron 二进制缓存");
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
        clean_idea_projects(&mut clean_entries, &format!("/Users/{username}/IdeaProjects"))?;
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
                    if entry.metadata()?.is_dir() {
                        if entry.file_name().to_string_lossy().starts_with("nt_qq") {
                            let mut nt_data = entry.path().clone();
                            nt_data.push("nt_data");
                            if nt_data.exists() {
                                let mut log = nt_data.clone();
                                log.push("log");
                                clean_entries.push(CleanEntry {
                                    path: log.to_string_lossy().into(),
                                    description: "QQ 日志".into(),
                                    score: 1.0,
                                });
                            }
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

    println!("time: {:?}", started.elapsed());
    println!("开始扫描磁盘空间占用情况...\n");

    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, Clear(ClearType::All), cursor::MoveTo(0, 0))?;

    let mut entries_with_size: Vec<(&CleanEntry, u64)> = Vec::new();
    let total_entries = clean_entries.len();

    for (index, entry) in clean_entries.iter().enumerate() {
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
                    &entry.path
                )
                .with(Color::Green)
            ),
            cursor::RestorePosition
        )?;
        io::stdout().flush()?;

        // 计算目录大小
        let path = Path::new(&entry.path);
        let size_result = check_size(path);

        if let Ok(size) = size_result {
            if size > 0 {
                entries_with_size.push((entry, size));
            }
        }
    }

    // 计算总大小
    let total_size: u64 = entries_with_size.iter().map(|(_, size)| size).sum();

    execute!(stdout, cursor::MoveTo(0, 0), Clear(ClearType::All),)?;
    disable_raw_mode()?;
    stdout.flush()?;
    for (entry, size) in &entries_with_size {
        println!(
            "{}: {} ({})",
            &entry.path,
            style::Print(format_size(*size).with(Color::Yellow).bold()),
            style::Print(entry.description.clone().with(Color::Green).bold()),
        )
    }

    execute!(
        stdout,
        style::Print("\n总计空间占用: ".to_string()),
        style::Print(format_size(total_size).with(Color::Green).bold())
    )?;

    // 恢复终端状态
    println!();
    println!("清理全部？(Y/Enter)");
    stdout.flush()?;
    enable_raw_mode()?;

    let mut clean_all = false;

    while let Event::Key(key_event) = crossterm::event::read()? {
        if let KeyCode::Char('y') | KeyCode::Char('Y') = key_event.code {
            clean_all = true;
            break;
        } else if let KeyCode::Enter = key_event.code {
            clean_all = true;
            break;
        } else if let KeyCode::Char(_) = key_event.code {
            disable_raw_mode()?;
            return Ok(());
        }
    }
    disable_raw_mode()?;
    if clean_all {
        for (entry, _) in entries_with_size {
            println!("{} {}", "正在清理".green(), entry.path);
            let result = std::fs::remove_dir_all(&entry.path);
            if let Err(e) = result {
                if e.kind() != io::ErrorKind::NotFound {
                    println!("清理 {} 失败: {}", entry.path, e.to_string().red().bold());
                }
            }
        }
    }
    Ok(())
}

fn clean_idea_projects(mut clean_entries: &mut Vec<CleanEntry>, path: &str) -> io::Result<()> {
    let read_dir = std::fs::read_dir(path)?;
    println!("正在清理 IntelliJ IDEA 项目目录: {}", path);

    for entry in read_dir {
        if let Ok(entry) = entry { // Skip error
            if entry.metadata()?.is_dir() {
                walk_and_delete(
                    &mut clean_entries,
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
    for entry in WalkDir::new(&root).max_depth(2) {
        if let Ok(entry) = entry {
            if let Ok(metadata) = entry.metadata() {
                if metadata.is_dir() {
                    let name = entry.file_name().to_str().unwrap_or("");
                    if to_delete.contains(&name) {
                        let mut modified_time =
                            metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
                        let mut access_time = metadata.accessed().unwrap_or(SystemTime::UNIX_EPOCH);
                        let mut created_time = metadata.created().unwrap_or(SystemTime::UNIX_EPOCH);
                        for entry_in_to_del in WalkDir::new(entry.path()).max_depth(5) {
                            if let Ok(entry_in_to_del) = entry_in_to_del {
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
                        }
                        let now = SystemTime::now();
                        let modified_duration =
                            now.duration_since(modified_time).unwrap_or_default();
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
                            });
                        }
                    }
                }
            }
        }
    }
}
