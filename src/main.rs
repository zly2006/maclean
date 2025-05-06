use crossterm::event::{Event, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use crossterm::{
    cursor, execute,
    style::{self, Color, Stylize},
    terminal::{self, Clear, ClearType},
};
use std::io::{self, Write};
use std::path::Path;
use walkdir::WalkDir;

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
    // 检查路径是否存在且是目录
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

    // 使用 WalkDir 遍历目录树
    for entry in WalkDir::new(path) {
        match entry {
            Ok(entry) => {
                let metadata = entry.metadata();
                match metadata {
                    Ok(metadata) => {
                        // 只计算文件的大小
                        if metadata.is_file() {
                            total_size += metadata.len();
                        }
                    }
                    Err(e) => {
                        // 处理获取文件元数据时的错误 (例如权限问题)
                        eprintln!("警告: 无法获取文件元数据 '{:?}': {}", entry.path(), e);
                    }
                }
            }
            Err(e) => {
                // 处理遍历目录时的错误 (例如没有读取目录权限)
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
    let username = whoami::username();

    let mut clean_entries: Vec<CleanEntry> = vec![];
    #[cfg(target_os = "macos")]
    {
        clean_entries.push(CleanEntry {
            path: format!("/Users/{username}/Library/Application Support/Microsoft/EdgeUpdater"),
            description: "Microsoft Edge 自动更新".into(),
            score: 1.0,
        });
        clean_entries.push(CleanEntry {
            path: format!(
                "/Users/{username}/Library/Containers/com.tencent.qq/Data/Library/Record"
            ),
            description: "QQ 录屏文件".into(),
            score: 1.0,
        });
        clean_entries.push(CleanEntry {
            path: format!("/Users/{username}/Library/Group Containers/UBF8T346G9.OneDriveStandaloneSuite/FileProviderLogs"),
            description: "OneDrive 日志".into(),
            score: 1.0,
        });
        clean_entries.push(CleanEntry {
            path: format!("/Users/{username}/Library/Logs/OneDrive"),
            description: "OneDrive 日志".into(),
            score: 1.0,
        });
        clean_entries.push(CleanEntry {
            path: format!(
                "/Users/{username}/Library/Containers/com.apple.mediaanalysisd/Data/Library/Caches"
            ),
            description: "mediaanalysisd 缓存".into(),
            score: 1.0,
        });
        clean_entries.push(CleanEntry {
            path: format!("/Users/{username}/Library/Containers/com.tencent.meeting/Data/Library/Global/Data/DynamicResourcePackage"),
            description: "腾讯会议下载缓存".into(),
            score: 1.0,
        });
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
        clean_entries.push(CleanEntry {
            path: format!(
                "/Users/{username}/Library/Application Support/Adobe/Common/Media Cache Files"
            ),
            description: "Adobe Media Cache".into(),
            score: 1.0,
        });
        clean_entries.push(CleanEntry {
            path: format!("/Users/{username}/Library/Application Support/Adobe/Common/Media Cache"),
            description: "Adobe Media Cache".into(),
            score: 1.0,
        });
        clean_entries.push(CleanEntry {
            path: format!("/Users/{username}/Library/Application Support/zoom.us/AutoUpdater"),
            description: "Zoom 自动更新".into(),
            score: 1.0,
        });
        clean_entries.push(CleanEntry {
            path: format!("/Users/{username}/Library/Logs/JetBrains"),
            description: "JetBrains 日志".into(),
            score: 1.0,
        });
        //todo: Library/Caches, IdeaProjects(target/build/.gradle), WebstormProjects(node_modules)
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
    }

    println!("当前用户：{}", username);
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
        execute!(
            io::stdout(),
            cursor::MoveTo(0, index as u16 + 3),
            Clear(ClearType::CurrentLine),
            style::Print(format!("正在扫描: {}", entry.path))
        )?;
        io::stdout().flush()?;

        // 计算目录大小
        let path = Path::new(&entry.path);
        let size_result = check_size(path);

        if let Ok(size) = size_result {
            entries_with_size.push((entry, size));
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
            println!("正在清理 {}", entry.path);
            std::fs::remove_dir_all(&entry.path)?;
        }
    }
    Ok(())
}
