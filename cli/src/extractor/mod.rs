pub mod bandizip;
pub mod sevenzip;

/// 列表输出结果
#[derive(Debug, Clone)]
pub struct ListOutput {
    pub total_bytes: u64,
    pub total_files: u32,
    pub raw_stdout: String,
}

/// 解压引擎统一接口
pub trait Extractor: Send + Sync {
    /// 引擎名称
    fn name(&self) -> &str;

    /// 可执行文件路径
    fn exe_path(&self) -> &str;

    /// 列出压缩包内容（带密码和编码）
    fn list(&self, archive: &str, password: &str, encoding: &str) -> anyhow::Result<ListOutput>;

    /// 解压压缩包到指定目录（带密码和编码）
    fn extract(&self, archive: &str, output_dir: &str, password: &str, encoding: &str) -> anyhow::Result<()>;

    /// 构建解压命令参数（供进度跟踪使用）
    fn extract_args(&self, archive: &str, output_dir: &str, password: &str) -> Vec<String>;

    /// 验证可执行文件是否可用
    fn validate(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
}

/// 根据配置创建对应的解压引擎
pub fn create_extractor(extractor_type: &str, path: &str) -> Option<Box<dyn Extractor>> {
    match extractor_type {
        "bandizip" => Some(Box::new(bandizip::BandizipExtractor::new(path.to_string()))),
        "7zip" => Some(Box::new(sevenzip::SevenZipExtractor::new(path.to_string()))),
        _ => None,
    }
}
