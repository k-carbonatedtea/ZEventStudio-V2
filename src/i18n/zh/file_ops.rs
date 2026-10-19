/// 中文 (Simplified Chinese) 文件资源管理器与文件操作本地化词典
pub fn get_zh_file_ops(key: &str) -> Option<&'static str> {
    let text = match key {
        // ---------------- 资源管理器右键上下文菜单与操作 ----------------
        "context_menu.rename" => "重命名",
        "context_menu.delete" => "删除",
        "context_menu.replace" => "替换文件...",
        "context_menu.add_file" => "添加文件...",
        "context_menu.export_file" => "导出文件...",
        "context_menu.export_container" => "导出到文件夹...",
        "context_menu.reveal_in_explorer" => "在文件资源管理器中显示",

        // ---------------- 模态确认弹窗 ----------------
        "modal.rename_title" => "重命名文件",
        "modal.rename_folder_title" => "重命名文件夹",
        "modal.current_name" => "当前名称",
        "modal.rename_placeholder" => "请输入新名称...",
        "modal.delete_title" => "确认删除",
        "modal.delete_file_confirm_msg" => "确定要从本地磁盘删除以下文件吗？此操作无法撤销：",
        "modal.delete_folder_confirm_msg" => "确定要从本地磁盘删除以下文件夹及其所有内容吗？此操作无法撤销：",
        "modal.delete_sarc_confirm_msg" => "确定要从封包中移除以下条目吗？此操作无法撤销：",
        "modal.delete_confirm_btn" => "确认删除",

        // ---------------- 操作提示通知 (Toast) ----------------
        "toast.rename_error" => "重命名失败",
        "toast.delete_success" => "已成功删除",
        "toast.delete_error" => "删除失败",
        "toast.replace_success" => "已成功替换文件",
        "toast.replace_error" => "替换文件失败",
        "toast.add_success" => "已成功添加文件",
        "toast.add_error" => "添加文件失败",
        "toast.export_success" => "导出成功",
        "toast.export_error" => "导出失败",

        _ => return None,
    };
    Some(text)
}
