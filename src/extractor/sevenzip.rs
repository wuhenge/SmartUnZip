use super::{Extractor, ListOutput};

pub struct SevenZipExtractor {
    exe_path: String,
}

impl SevenZipExtractor {
    pub fn new(exe_path: String) -> Self {
        Self { exe_path }
    }
}

impl Extractor for SevenZipExtractor {
    fn name(&self) -> &str {
        "7-Zip"
    }

    fn exe_path(&self) -> &str {
        &self.exe_path
    }

    fn list(&self, archive: &str, password: &str, encoding: &str) -> anyhow::Result<ListOutput> {
        let pwd_flag = if password.is_empty() {
            String::new()
        } else {
            format!("-p{password}")
        };

        let mut args = vec!["l", "-slt", "-y"];
        if !pwd_flag.is_empty() {
            args.push(&pwd_flag);
        }
        args.push(archive);

        let output = crate::archive::run_capture(self.exe_path(), &args, encoding)?;

        if output.exit_code != 0 {
            return Ok(ListOutput {
                total_bytes: 0,
                total_files: 0,
                raw_stdout: output.stdout,
            });
        }

        let metrics = parse_7zip_slt_listing(&output.stdout);
        Ok(ListOutput {
            total_bytes: metrics.0,
            total_files: metrics.1,
            raw_stdout: output.stdout,
        })
    }

    fn extract(&self, archive: &str, output_dir: &str, password: &str, encoding: &str) -> anyhow::Result<()> {
        let pwd_flag = if password.is_empty() {
            String::new()
        } else {
            format!("-p{password}")
        };
        let out_flag = format!("-o{output_dir}");

        let mut args = vec!["x", "-y", &out_flag];
        if !pwd_flag.is_empty() {
            args.push(&pwd_flag);
        }
        args.push(archive);

        let output = crate::archive::run_capture(self.exe_path(), &args, encoding)?;

        if output.exit_code != 0 {
            anyhow::bail!(
                "7-Zip 解压失败 (exit code {}): {}",
                output.exit_code,
                output.stderr.trim()
            );
        }
        Ok(())
    }

    fn validate(&self, path: &str) -> bool {
        if !std::path::Path::new(path).exists() {
            return false;
        }
        let file_name = std::path::Path::new(path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_lowercase();
        matches!(file_name.as_str(), "7z.exe" | "7z" | "7zz")
    }

    fn extract_args(&self, archive: &str, output_dir: &str, password: &str) -> Vec<String> {
        let mut args = vec!["x".to_string(), "-y".to_string(), format!("-o{output_dir}")];
        if !password.is_empty() {
            args.push(format!("-p{password}"));
        }
        args.push(archive.to_string());
        args
    }
}

/// 解析 7-Zip -slt 模式的列表输出
fn parse_7zip_slt_listing(output: &str) -> (u64, u32) {
    let mut total_bytes: u64 = 0;
    let mut total_files: u32 = 0;

    // 逐行解析，遇到新的 Path = 表示新条目开始
    // 7-Zip -slt 输出中第一个 Path = 是压缩包自身（带有 Type = xxx），
    // 需要跳过；后续的 Path = 才是压缩包内的条目
    let mut has_path = false;
    let mut is_folder = false;
    let mut is_archive_header = false;
    let mut size: u64 = 0;

    for raw_line in output.lines() {
        let line = raw_line.trim();

        if let Some(path_val) = line.strip_prefix("Path =") {
            // 新条目开始，先提交上一个条目
            if has_path && !is_folder && !is_archive_header {
                total_bytes += size;
                total_files += 1;
            }
            // 重置状态
            has_path = !path_val.trim().is_empty();
            is_folder = false;
            is_archive_header = false;
            size = 0;
        } else if has_path {
            if line == "Folder = +" {
                is_folder = true;
            } else if let Some(size_val) = line.strip_prefix("Size =") {
                size = size_val.trim().parse().unwrap_or(0);
            } else if line.starts_with("Type =") || line.starts_with("Physical Size =") {
                // Type / Physical Size 只出现在压缩包头部，标记跳过
                is_archive_header = true;
            }
        }
    }

    // 提交最后一个条目
    if has_path && !is_folder && !is_archive_header {
        total_bytes += size;
        total_files += 1;
    }

    (total_bytes, total_files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_slt_listing() {
        // 标准 7-Zip -slt 输出（带 ---------- 分隔符）
        let output = r#"
7-Zip 26.00 (x64) : Copyright (c) 1999-2024 Igor Pavlov : 2024-12-20

Scanning the drive for archives:
1 file, 1024 bytes (1 KiB)

Listing archive: test.zip

--
Path = test.zip
Type = zip
Physical Size = 1024

----------
Path = folder1
Folder = +
Size = 0

----------
Path = file1.txt
Size = 100
Folder = -

----------
Path = file2.txt
Size = 200
Folder = -

----------
"#;
        let (total_bytes, total_files) = parse_7zip_slt_listing(output);
        assert_eq!(total_bytes, 300);
        assert_eq!(total_files, 2);
    }

    #[test]
    fn test_parse_slt_listing_no_separator() {
        // 某些格式没有 ---------- 分隔符（如用户遇到的 .7z1 格式）
        let output = r#"
7-Zip 26.00 (x64) : Copyright (c) 1999-2024 Igor Pavlov : 2024-12-20

Scanning the drive for archives:
1 file, 113082465 bytes (108 MiB)

Listing archive: test.7z1

--
Path = test.7z1
Type = 7z
Physical Size = 113082465

Path = file1.txt
Size = 1014259
Folder = -

Path = folder1
Folder = +
Size = 0

Path = file2.txt
Size = 1556774
Folder = -
"#;
        let (total_bytes, total_files) = parse_7zip_slt_listing(output);
        assert_eq!(total_bytes, 1014259 + 1556774);
        assert_eq!(total_files, 2);
    }

    #[test]
    fn test_parse_slt_listing_without_folder_field() {
        // 某些格式不输出 Folder = 字段，应视为文件
        let output = r#"
Path = archive.7z
Type = 7z
Physical Size = 500

Path = single_file.txt
Size = 500
"#;
        let (total_bytes, total_files) = parse_7zip_slt_listing(output);
        assert_eq!(total_bytes, 500);
        assert_eq!(total_files, 1);
    }
}
