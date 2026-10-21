/// English (en) File operations and context menu localization dictionary
pub fn get_en_file_ops(key: &str) -> Option<&'static str> {
    let text = match key {
        // ---------------- Context Menu Actions ----------------
        "context_menu.rename" => "Rename",
        "context_menu.delete" => "Delete",
        "context_menu.replace" => "Replace File...",
        "context_menu.add_file" => "Add File...",
        "context_menu.export_file" => "Export File...",
        "context_menu.export_container" => "Export to Folder...",
        "context_menu.reveal_in_explorer" => "Reveal in File Explorer",

        // ---------------- Confirmation Modals ----------------
        "modal.rename_title" => "Rename File",
        "modal.rename_folder_title" => "Rename Folder",
        "modal.current_name" => "Current Name",
        "modal.rename_placeholder" => "Enter new name...",
        "modal.delete_title" => "Confirm Delete",
        "modal.delete_file_confirm_msg" => "Are you sure you want to delete this file from disk? This cannot be undone:",
        "modal.delete_folder_confirm_msg" => "Are you sure you want to delete this folder and its contents? This cannot be undone:",
        "modal.delete_sarc_confirm_msg" => "Are you sure you want to remove this entry from the archive? This cannot be undone:",
        "modal.delete_confirm_btn" => "Confirm Delete",

        // ---------------- Toasts & Notifications ----------------
        "toast.rename_error" => "Rename failed",
        "toast.delete_success" => "Deleted successfully",
        "toast.delete_error" => "Delete failed",
        "toast.replace_success" => "File replaced successfully",
        "toast.replace_error" => "Failed to replace file",
        "toast.add_success" => "File added successfully",
        "toast.add_error" => "Failed to add file",
        "toast.export_success" => "Exported successfully",
        "toast.export_error" => "Export failed",
        "toast.file_not_found" => "Target file or path does not exist",
        "toast.explorer_open_failed" => "Failed to open File Explorer",

        _ => return None,
    };
    Some(text)
}
