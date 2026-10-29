use crate::model::code_document::{CodeEndian, CodeFormat};
use roead::aamp::ParameterIO;
use roead::byml::Byml;
use roead::Endian;

/// 检查并解压可能被 Yaz0 压缩的二进制字节流
pub fn decompress_if_yaz0(data: &[u8]) -> Vec<u8> {
    if data.len() >= 4 && &data[0..4] == b"Yaz0" {
        roead::yaz0::decompress(data).unwrap_or_else(|_| data.to_vec())
    } else {
        data.to_vec()
    }
}

/// 检查文件名或路径是否代表 Yaz0 压缩格式 (以 .s 开头后缀或 .zs 后缀)
pub fn should_compress_yaz0(file_path: &str) -> bool {
    let lower = file_path.to_lowercase();
    let name = lower.rsplit(['/', '\\']).next().unwrap_or(&lower);
    name.ends_with(".sbactorpack")
        || name.ends_with(".sbeventpack")
        || name.ends_with(".ssarc")
        || name.ends_with(".spack")
        || name.ends_with(".sbyml")
        || name.ends_with(".sbgyml")
        || name.ends_with(".sbaev")
        || name.ends_with(".sbxml")
        || name.ends_with(".smsbt")
        || name.ends_with(".sbfevfl")
        || name.ends_with(".sbaiprog")
        || name.ends_with(".sbquestpack")
        || name.ends_with(".smubin")
        || name.ends_with(".sbgdata")
        || name.ends_with(".sbgsvdata")
        || name.ends_with(".sbdef")
        || name.ends_with(".sbdrop")
        || name.ends_with(".sgenvb")
        || name.ends_with(".sbref")
        || lower.ends_with(".zs")
}

/// 检查文件路径是否属于任天堂二进制 YAML 格式扩展名
pub fn is_byml_extension(file_path: &str) -> bool {
    let lower = file_path.to_lowercase();
    lower.ends_with(".byml")
        || lower.ends_with(".sbyml")
        || lower.ends_with(".sbgyml")
        || lower.ends_with(".bquestpack")
        || lower.ends_with(".sbquestpack")
        || lower.ends_with(".bgdata")
        || lower.ends_with(".sbgdata")
        || lower.ends_with(".bgsvdata")
        || lower.ends_with(".sbgsvdata")
        || lower.ends_with(".mubin")
        || lower.ends_with(".smubin")
        || lower.ends_with(".bdef")
        || lower.ends_with(".sbdef")
        || lower.ends_with(".bdrop")
        || lower.ends_with(".sbdrop")
        || lower.ends_with(".genvb")
        || lower.ends_with(".sgenvb")
        || lower.ends_with(".bref")
        || lower.ends_with(".sbref")
}

/// 根据已读取的二进制数据魔数特征推断文件原生端序
pub fn detect_endianness_from_bytes(data: &[u8]) -> Option<CodeEndian> {
    let decompressed = decompress_if_yaz0(data);
    if decompressed.len() >= 2 {
        if &decompressed[0..2] == b"BY" {
            return Some(CodeEndian::Big);
        } else if &decompressed[0..2] == b"YB" {
            return Some(CodeEndian::Little);
        }
    }
    if decompressed.len() >= 8 && &decompressed[0..4] == b"SARC" {
        if &decompressed[6..8] == b"\xfe\xff" {
            return Some(CodeEndian::Big);
        } else if &decompressed[6..8] == b"\xff\xfe" {
            return Some(CodeEndian::Little);
        }
    }
    if decompressed.len() >= 10 && &decompressed[0..8] == b"MsgStdBn" {
        if &decompressed[8..10] == b"\xfe\xff" {
            return Some(CodeEndian::Big);
        } else {
            return Some(CodeEndian::Little);
        }
    }
    if decompressed.len() >= 6 && &decompressed[0..4] == b"BFEV" {
        if &decompressed[4..6] == b"\xfe\xff" {
            return Some(CodeEndian::Big);
        } else if &decompressed[4..6] == b"\xff\xfe" {
            return Some(CodeEndian::Little);
        }
    }
    if decompressed.len() >= 4 && &decompressed[0..4] == b"AAMP" {
        return Some(CodeEndian::Little);
    }
    None
}

/// 将任意二进制文件转换为可供代码编辑器编辑的文本表示
pub fn parse_file_to_code(
    data: &[u8],
    file_path: &str,
) -> Result<(String, CodeFormat, CodeEndian), String> {
    let decompressed = decompress_if_yaz0(data);
    let lower_path = file_path.to_lowercase();

    // 1. AAMP / BAIPROG 二进制格式识别 (Magic: "AAMP")
    if decompressed.len() >= 4 && &decompressed[0..4] == b"AAMP" {
        let pio = ParameterIO::from_binary(&decompressed)
            .map_err(|e| format!("解析 AAMP / BAIPROG 失败: {:?}", e))?;
        return Ok((pio.to_text(), CodeFormat::Aamp, CodeEndian::Little));
    }

    // 1.1 AAMP / BAIPROG 文本格式识别 (如以 !io 开头，或扩展名为 .baiprog / .sbaiprog / .aamp)
    if lower_path.ends_with(".baiprog")
        || lower_path.ends_with(".sbaiprog")
        || lower_path.ends_with(".aamp")
        || lower_path.ends_with(".bxml")
        || lower_path.ends_with(".bactparam")
    {
        if let Ok(text) = String::from_utf8(decompressed.clone()) {
            if text.trim_start().starts_with("!io") || ParameterIO::from_text(&text).is_ok() {
                return Ok((text, CodeFormat::Aamp, CodeEndian::Little));
            }
        }
    }

    // 2. BYML / BGYML 格式识别 (Magic: "BY" 或 "YB")
    if decompressed.len() >= 2 && (&decompressed[0..2] == b"BY" || &decompressed[0..2] == b"YB") {
        let endian = if &decompressed[0..2] == b"BY" {
            CodeEndian::Big
        } else {
            CodeEndian::Little
        };
        let byml = Byml::from_binary(&decompressed)
            .map_err(|e| format!("解析 BYML 失败: {:?}", e))?;
        return Ok((byml.to_text(), CodeFormat::Byml, endian));
    }

    // 3. MSBT 本地化文本格式识别 (Magic: "MsgStdBn")
    if decompressed.len() >= 8 && &decompressed[0..8] == b"MsgStdBn" {
        let endian = if decompressed.len() >= 10 && &decompressed[8..10] == b"\xfe\xff" {
            CodeEndian::Big
        } else {
            CodeEndian::Little
        };
        let msyt_doc = msyt::Msyt::from_msbt_bytes(&decompressed)
            .map_err(|e| format!("解析 MSBT 对话数据失败: {:?}", e))?;
        let yaml_text = serde_yaml::to_string(&msyt_doc)
            .map_err(|e| format!("序列化 MSYT 文本失败: {:?}", e))?;
        return Ok((yaml_text, CodeFormat::Msbt, endian));
    }

    // 4. BFEVFL / BFEVTM 事件流与时间线格式识别 (Magic: "BFEVFL" 或扩展名 .bfevfl / .bfevtm)
    if (decompressed.len() >= 6 && &decompressed[0..6] == b"BFEVFL")
        || lower_path.ends_with(".bfevfl")
        || lower_path.ends_with(".bfevtm")
    {
        match crate::model::evfl::load_eventflow_from_bytes(&decompressed) {
            Ok(evfl) => {
                let is_timeline = evfl.timeline.is_some() || lower_path.ends_with(".bfevtm");
                let format = if is_timeline { CodeFormat::Bfevtm } else { CodeFormat::Bfevfl };
                let json_text = serde_json::to_string_pretty(&evfl)
                    .map_err(|e| format!("格式化 EventFlow JSON 失败: {:?}", e))?;
                return Ok((json_text, format, CodeEndian::Little));
            }
            Err(e) => return Err(format!("解析 BFEVFL/BFEVTM 失败: {}", e)),
        }
    }

    // 5. JSON 格式识别
    if lower_path.ends_with(".json") {
        if let Ok(text) = String::from_utf8(decompressed.clone()) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&text) {
                let formatted = serde_json::to_string_pretty(&val).unwrap_or(text);
                return Ok((formatted, CodeFormat::Json, CodeEndian::Little));
            }
            return Ok((text, CodeFormat::Json, CodeEndian::Little));
        }
    }

    // 6. YAML 格式识别
    if lower_path.ends_with(".yml") || lower_path.ends_with(".yaml") {
        if let Ok(text) = String::from_utf8(decompressed.clone()) {
            return Ok((text, CodeFormat::Yaml, CodeEndian::Little));
        }
    }

    // 7. 通用文本与日志识别 (UTF-8 字符流)
    if let Ok(text) = String::from_utf8(decompressed.clone()) {
        return Ok((text, CodeFormat::Text, CodeEndian::Little));
    }

    // 8. 二进制降级处理
    Err(format!(
        "暂不支持直接以文本编辑此二进制文件格式 (大小: {} 字节)",
        data.len()
    ))
}

/// 将代码编辑器中的文本重新编译为目标二进制字节流
pub fn compile_code_to_bytes(
    text: &str,
    format: CodeFormat,
    endian: CodeEndian,
    file_path: &str,
) -> Result<Vec<u8>, String> {
    let raw_bytes = match format {
        CodeFormat::Aamp => {
            let pio = ParameterIO::from_text(text)
                .map_err(|e| format!("AAMP YAML 语法解析错误: {:?}", e))?;
            pio.to_binary().to_vec()
        }
        CodeFormat::Byml => {
            let byml = Byml::from_text(text)
                .map_err(|e| format!("BYML YAML 语法解析错误: {:?}", e))?;
            let roead_endian = match endian {
                CodeEndian::Big => Endian::Big,
                CodeEndian::Little => Endian::Little,
            };
            byml.to_binary(roead_endian).to_vec()
        }
        CodeFormat::Msbt => {
            let msyt_doc: msyt::Msyt = serde_yaml::from_str(text)
                .map_err(|e| format!("MSBT (MSYT) YAML 语法错误: {:?}", e))?;
            let msyt_endian = match endian {
                CodeEndian::Big => msyt::Endianness::Big,
                CodeEndian::Little => msyt::Endianness::Little,
            };
            msyt_doc
                .into_msbt_bytes(msyt_endian)
                .map_err(|e| format!("编译 MSBT 字节流失败: {:?}", e))?
        }
        CodeFormat::Bfevfl | CodeFormat::Bfevtm => {
            // 优先解析原生 EventFlow 结构并编译为官方 BFEVFL/BFEVTM 二进制字节流；若为直接 Timeline 或旧版 Graph 则兼容降级
            if let Ok(evfl) = serde_json::from_str::<revfl::evfl::EventFlow>(text) {
                crate::model::evfl::save_eventflow_to_bytes(&evfl)
                    .map_err(|e| format!("编译 BFEVFL/BFEVTM 失败: {}", e))?
            } else if let Ok(tl) = serde_json::from_str::<revfl::timeline::Timeline>(text) {
                let mut evfl = revfl::evfl::EventFlow::new();
                evfl.name = tl.name.clone();
                evfl.timeline = Some(tl);
                crate::model::evfl::save_eventflow_to_bytes(&evfl)
                    .map_err(|e| format!("编译 BFEVTM 失败: {}", e))?
            } else if let Ok(graph) = serde_json::from_str::<crate::model::Graph>(text) {
                crate::model::evfl::save_bfevfl_to_bytes(&graph)
                    .map_err(|e| format!("编译 BFEVFL 失败: {}", e))?
            } else {
                return Err("BFEVFL/BFEVTM JSON 语法错误: 无法反序列化为 EventFlow 或 Timeline 结构".to_string());
            }
        }
        CodeFormat::Json | CodeFormat::Yaml | CodeFormat::Text | CodeFormat::Generic => {
            text.as_bytes().to_vec()
        }
    };

    // 如果目标路径要求 Yaz0 压缩，自动执行压缩打包
    if should_compress_yaz0(file_path) {
        Ok(roead::yaz0::compress(&raw_bytes))
    } else {
        Ok(raw_bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_byml_roundtrip() {
        let yaml_src = "Actor:\n  Health: 100\n  Name: Link\n";
        let compiled = compile_code_to_bytes(yaml_src, CodeFormat::Byml, CodeEndian::Little, "test.byml").unwrap();
        assert!(!compiled.is_empty());
        assert_eq!(&compiled[0..2], b"YB");

        let (parsed_yaml, fmt, end) = parse_file_to_code(&compiled, "test.byml").unwrap();
        assert_eq!(fmt, CodeFormat::Byml);
        assert_eq!(end, CodeEndian::Little);
        assert!(parsed_yaml.contains("Health"));
    }

    #[test]
    fn test_aamp_roundtrip() {
        let pio = ParameterIO::new();
        let aamp_src = pio.to_text();
        let compiled = compile_code_to_bytes(&aamp_src, CodeFormat::Aamp, CodeEndian::Little, "test.bxml").unwrap();
        assert!(!compiled.is_empty());
        assert_eq!(&compiled[0..4], b"AAMP");

        let (parsed_yaml, fmt, _) = parse_file_to_code(&compiled, "test.bxml").unwrap();
        assert_eq!(fmt, CodeFormat::Aamp);
        assert!(parsed_yaml.contains("param_root"));
    }

    #[test]
    fn test_msbt_roundtrip() {
        let msyt_src = "group_count: 0\nentries: {}\n";
        let compiled = compile_code_to_bytes(msyt_src, CodeFormat::Msbt, CodeEndian::Little, "Message.msbt").unwrap();
        assert!(!compiled.is_empty());
        assert_eq!(&compiled[0..8], b"MsgStdBn");

        let (parsed_yaml, fmt, end) = parse_file_to_code(&compiled, "Message.msbt").unwrap();
        assert_eq!(fmt, CodeFormat::Msbt);
        assert_eq!(end, CodeEndian::Little);
        assert!(parsed_yaml.contains("entries"));
    }

    #[test]
    fn test_baiprog_roundtrip() {
        let pio = ParameterIO::new();
        let aamp_src = pio.to_text();
        let compiled = compile_code_to_bytes(&aamp_src, CodeFormat::Aamp, CodeEndian::Little, "Lynel.baiprog").unwrap();
        assert!(!compiled.is_empty());
        assert_eq!(&compiled[0..4], b"AAMP");

        let (parsed_yaml, fmt, _) = parse_file_to_code(&compiled, "Lynel.baiprog").unwrap();
        assert_eq!(fmt, CodeFormat::Aamp);
        assert!(parsed_yaml.contains("param_root"));
    }

    #[test]
    fn test_bfevfl_native_eventflow_roundtrip() {
        let bfevfl_raw = include_bytes!("../../100enemy.bfevfl");
        let (json_code, fmt, end) = parse_file_to_code(bfevfl_raw, "100enemy.bfevfl").unwrap();
        assert_eq!(fmt, CodeFormat::Bfevfl);
        assert_eq!(end, CodeEndian::Little);

        // 验证不再包含编辑器 UI 画布结构，而是任天堂原生 EventFlow 反序列化格式
        assert!(json_code.contains("\"flowchart\""));
        assert!(json_code.contains("\"actors\""));
        assert!(json_code.contains("\"events\""));
        assert!(!json_code.contains("\"kind\": \"EntryPoint\""));
        assert!(!json_code.contains("\"color_override\""));

        // 重新编译为 BFEVFL 字节流
        let compiled = compile_code_to_bytes(&json_code, CodeFormat::Bfevfl, CodeEndian::Little, "100enemy.bfevfl").unwrap();
        assert!(compiled.len() >= 8);
        assert_eq!(&compiled[0..6], b"BFEVFL");

        // 再次解析编译出的二进制，确认往返完全一致且可正常反序列化
        let (reparsed_code, fmt2, _) = parse_file_to_code(&compiled, "100enemy.bfevfl").unwrap();
        assert_eq!(fmt2, CodeFormat::Bfevfl);
        assert!(reparsed_code.contains("\"flowchart\""));
    }
}
