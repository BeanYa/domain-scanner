mod models;
mod db;
mod scanner;
mod llm;
mod embedding;
mod proxy;
mod export;
mod commands;

use commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            task_cmds::create_tasks,
            task_cmds::start_task,
            task_cmds::pause_task,
            task_cmds::resume_task,
            task_cmds::delete_task,
            task_cmds::list_tasks,
            task_cmds::get_task_detail,
            batch_cmds::list_batches,
            batch_cmds::batch_pause,
            batch_cmds::batch_resume,
            scan_cmds::scan_preview,
            export_cmds::export_results,
            filter_cmds::filter_exact,
            filter_cmds::filter_fuzzy,
            filter_cmds::filter_regex,
            filter_cmds::filter_semantic,
            proxy_cmds::list_proxies,
            proxy_cmds::create_proxy,
            proxy_cmds::test_proxy,
            llm_cmds::list_llm_configs,
            llm_cmds::save_llm_config,
            llm_cmds::test_llm_config,
            log_cmds::get_logs,
            vector_cmds::start_vectorize,
            vector_cmds::get_vectorize_progress,
            gpu_cmds::get_gpu_status,
            gpu_cmds::update_gpu_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
