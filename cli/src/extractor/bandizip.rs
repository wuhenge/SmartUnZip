use super::{Extractor, ListOutput};

pub struct BandizipExtractor {
    exe_path: String,
}

impl BandizipExtractor {
    pub fn new(exe_path: String) -> Self {
        Self { exe_path }
    }
}

impl Extractor for BandizipExtractor {
    fn name(&self) -> &str {
        "Bandizip"
    }

    fn exe_path(&self) -> &str {
        &self.exe_path
    }

    fn list(&self, archive: &str, password: &str, encoding: &str) -> anyhow::Result<ListOutput> {
        let pwd_flag = format!("-p:{password}");
        let args = vec!["l", "-list:v", "-y", &pwd_flag, archive];
        let output = crate::archive::run_capture(self.exe_path(), &args, encoding)?;

        if output.exit_code != 0 {
            return Ok(ListOutput {
                total_bytes: 0,
                total_files: 0,
                raw_stdout: output.stdout,
            });
        }

        let metrics = crate::archive::parse_listing_metrics(&output.stdout);
        Ok(ListOutput {
            total_bytes: metrics.map(|m| m.total_bytes).unwrap_or(0),
            total_files: metrics.map(|m| m.total_files).unwrap_or(0),
            raw_stdout: output.stdout,
        })
    }

    fn extract(&self, archive: &str, output_dir: &str, password: &str, encoding: &str) -> anyhow::Result<()> {
        let pwd_flag = format!("-p:{password}");
        let out_flag = format!("-o:{output_dir}");
        let args = vec!["x", "-y", &pwd_flag, &out_flag, archive];
        let output = crate::archive::run_capture(self.exe_path(), &args, encoding)?;

        if output.exit_code != 0 {
            anyhow::bail!(
                "Bandizip 解压失败 (exit code {}): {}",
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
        matches!(file_name.as_str(), "bz.exe" | "bz")
    }

    fn extract_args(&self, archive: &str, output_dir: &str, password: &str) -> Vec<String> {
        vec![
            "x".to_string(),
            "-y".to_string(),
            format!("-p:{password}"),
            format!("-o:{output_dir}"),
            archive.to_string(),
        ]
    }
}
