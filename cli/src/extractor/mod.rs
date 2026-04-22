pub mod sevenzip;

#[derive(Debug, Clone)]
pub struct ListOutput {
    pub total_bytes: u64,
    pub total_files: u32,
    pub raw_stdout: String,
}

pub trait Extractor: Send + Sync {
    fn name(&self) -> &str;

    fn exe_path(&self) -> &str;

    fn list(&self, archive: &str, password: &str, encoding: &str) -> anyhow::Result<ListOutput>;

    fn extract(
        &self,
        archive: &str,
        output_dir: &str,
        password: &str,
        encoding: &str,
    ) -> anyhow::Result<()>;

    fn extract_args(&self, archive: &str, output_dir: &str, password: &str) -> Vec<String>;

    fn validate(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
}

pub fn create_extractor(extractor_type: &str, path: &str) -> Option<Box<dyn Extractor>> {
    match extractor_type {
        "7zip" => Some(Box::new(sevenzip::SevenZipExtractor::new(path.to_string()))),
        _ => None,
    }
}
