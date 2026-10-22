use crate::model::{extract_sarc_to_dir, pack_dir_to_sarc_file, should_compress_yaz0};
use crate::ui::components::Toast;
use crate::ui::state::AppState;
use roead::Endian;
use std::path::Path;

/// 针对 SARC 封包物理打包与解包导出的 API 实现
impl AppState {
    /// 将物理文件夹内的全部文件打包为 SARC 归档文件 (支持 Yaz0 压缩与大/小端控制)
    pub fn api_pack_sarc(&mut self, payload: &serde_json::Value) -> Result<serde_json::Value, String> {
        let raw_src = payload
            .get("source_dir")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "缺少 source_dir 参数".to_string())?;
        let raw_dst = payload
            .get("target_file")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "缺少 target_file 参数".to_string())?;

        let resolved_src = self.resolve_target_path(raw_src);
        let resolved_dst = self.resolve_target_path(raw_dst);

        let endian_str = payload.get("endian").and_then(|v| v.as_str()).unwrap_or("big");
        let endian = if endian_str.eq_ignore_ascii_case("little") {
            Endian::Little
        } else {
            Endian::Big
        };

        let compress_yaz0 = payload
            .get("compress_yaz0")
            .and_then(|v| v.as_bool())
            .unwrap_or_else(|| should_compress_yaz0(&resolved_dst));

        let src_path = Path::new(&resolved_src);
        let dst_path = Path::new(&resolved_dst);

        let bytes_written = pack_dir_to_sarc_file(src_path, dst_path, endian, compress_yaz0)?;

        self.reload_project();
        if self.git_is_repo {
            self.refresh_git_status();
        }

        let file_name = dst_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| resolved_dst.clone());
        self.toast = Some(Toast::success(format!("MCP: 封包已打包 「{}」 ({} 字节)", file_name, bytes_written)));

        Ok(serde_json::json!({
            "status": "ok",
            "source_dir": resolved_src,
            "target_file": resolved_dst,
            "bytes_written": bytes_written,
            "compressed_yaz0": compress_yaz0,
            "endian": if endian == Endian::Little { "little" } else { "big" },
        }))
    }

    /// 将 SARC 封包或封包内子目录解包导出到指定物理文件夹中
    pub fn api_unpack_sarc(&mut self, payload: &serde_json::Value) -> Result<serde_json::Value, String> {
        let raw_src = payload
            .get("source_file")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "缺少 source_file 参数".to_string())?;
        let raw_dst = payload
            .get("target_dir")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "缺少 target_dir 参数".to_string())?;

        let resolved_src = self.resolve_target_path(raw_src);
        let resolved_dst = self.resolve_target_path(raw_dst);

        let dst_path = Path::new(&resolved_dst);
        let extracted_count = extract_sarc_to_dir(&resolved_src, dst_path)?;

        self.reload_project();
        if self.git_is_repo {
            self.refresh_git_status();
        }

        let dir_name = dst_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| resolved_dst.clone());
        self.toast = Some(Toast::success(format!("MCP: 已解包 {} 个文件至 「{}」", extracted_count, dir_name)));

        Ok(serde_json::json!({
            "status": "ok",
            "source_file": resolved_src,
            "target_dir": resolved_dst,
            "extracted_count": extracted_count,
        }))
    }
}
