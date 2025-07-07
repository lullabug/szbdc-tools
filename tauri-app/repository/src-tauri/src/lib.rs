mod fumo;

#[tauri::command]
async fn scan_barcode(luma: Vec<u8>, width: u32, height: u32) -> Result<Option<String>, String> {
    use tools_core::infra::barcode::decode_text_from_luma;

    let rs = decode_text_from_luma(luma, width, height).await
        .map_err(|e| e.to_string())?;
    Ok(rs)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            scan_barcode,
            fumo::fumo_load, fumo::fumo_get_by_uid, fumo::fumo_get_by_sku, fumo::fumo_add, fumo::fumo_remove
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
