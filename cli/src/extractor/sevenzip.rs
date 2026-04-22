use super::{Extractor, ListOutput};

pub struct SevenZipExtractor {
    exe_path: String,
}

impl SevenZipExtractor {
    pub fn new(exe_path: String) -> Self {
        Self { exe_path }
    }

    fn password_flag(password: &str) -> Option<String> {
        if password.is_empty() {
            None
        } else {
            Some(format!("-p{password}"))
        }
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
        let mut args = vec!["l".to_string(), "-slt".to_string(), "-y".to_string()];
        if let Some(password_flag) = Self::password_flag(password) {
            args.push(password_flag);
        }
        args.push(archive.to_string());

        let args_ref: Vec<&str> = args.iter().map(|s| s.as_str()).collect();
        let output = crate::archive::run_capture(self.exe_path(), &args_ref, encoding)?;

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

    fn extract(
        &self,
        archive: &str,
        output_dir: &str,
        password: &str,
        encoding: &str,
    ) -> anyhow::Result<()> {
        let extract_args = self.extract_args(archive, output_dir, password);
        let args_ref: Vec<&str> = extract_args.iter().map(|s| s.as_str()).collect();
        let output = crate::archive::run_capture(self.exe_path(), &args_ref, encoding)?;

        if output.exit_code != 0 {
            anyhow::bail!(
                "7-Zip 瑙ｅ帇澶辫触 (exit code {}): {}",
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
        let mut args = vec![
            "x".to_string(),
            "-y".to_string(),
            "-bb0".to_string(),
            "-bso0".to_string(),
            "-bse2".to_string(),
            "-bsp1".to_string(),
            format!("-o{output_dir}"),
        ];
        if let Some(password_flag) = Self::password_flag(password) {
            args.push(password_flag);
        }
        args.push(archive.to_string());
        args
    }
}

fn parse_7zip_slt_listing(output: &str) -> (u64, u32) {
    let mut total_bytes: u64 = 0;
    let mut total_files: u32 = 0;
    let mut has_path = false;
    let mut is_folder = false;
    let mut is_archive_header = false;
    let mut size: u64 = 0;

    for raw_line in output.lines() {
        let line = raw_line.trim();

        if let Some(path_val) = line.strip_prefix("Path =") {
            if has_path && !is_folder && !is_archive_header {
                total_bytes += size;
                total_files += 1;
            }
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
                is_archive_header = true;
            }
        }
    }

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
