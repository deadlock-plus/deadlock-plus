mod features;

use features::ingest::IngestService;
use features::network::{self, NetworkMonitor};
use features::server_picker::{self, ServerPickerState};
use tauri::{Manager, RunEvent};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    features::logging::install_panic_hook();
    let app = tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| features::tray::show_main(app)))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(features::window_state::plugin())
        .manage(ServerPickerState::default())
        .manage(NetworkMonitor::default())
        .manage(IngestService::default())
        .manage(features::tray::CloseToTray::default())
        .manage(features::maintenance::MaintenanceState::default())
        .manage(features::alerts::AlertsState::default())
        .manage(features::demos::metadata::DemoMetaCache::default())
        .manage(features::demos::pin::PinStore::default())
        .manage(features::kv::KvStore::default())
        .setup(|app| {
            let log_dir = app.path().app_log_dir()?;
            app.handle().plugin(features::logging::plugin(&log_dir)?)?;
            features::logging::log_startup(app.handle());
            network::commands::start_monitor(app.handle());
            features::tray::setup(app.handle())?;
            features::maintenance::start(app.handle());
            features::alerts::start(app.handle());
            Ok(())
        })
        .on_window_event(features::tray::on_window_event)
        .invoke_handler(tauri::generate_handler![
            server_picker::commands::get_game_definitions,
            server_picker::commands::fetch_server_groups,
            server_picker::commands::ping_server_groups,
            server_picker::commands::block_server_groups,
            server_picker::commands::unblock_server_groups,
            server_picker::commands::list_blocked_group_ids,
            server_picker::commands::firewall_capability,
            server_picker::commands::detect_external_blocks,
            server_picker::commands::import_external_blocks,
            network::commands::start_network_monitor,
            network::commands::network_snapshot,
            network::commands::network_history,
            features::ingest::commands::set_ingest_enabled,
            features::ingest::commands::ingest_status,
            features::steam_account::commands::current_steam_account,
            features::steam_account::commands::local_steam_account_ids,
            features::voice_bans::commands::read_voice_ban,
            features::voice_bans::commands::write_voice_ban,
            features::demos::commands::list_demos,
            features::demos::commands::reveal_demo,
            features::demos::commands::open_replays_dir,
            features::demos::commands::delete_preview,
            features::demos::commands::delete_demos,
            features::demos::metadata::commands::demo_metadata,
            features::demos::pin::commands::list_pinned,
            features::demos::cleanup::commands::list_cleanup_rules,
            features::demos::cleanup::commands::save_cleanup_rules,
            features::demos::cleanup::commands::cleanup_matches,
            features::demos::pin::commands::set_pinned,
            features::storage::commands::storage_entries,
            features::storage::commands::storage_entry_size,
            features::storage::commands::storage_reveal,
            features::storage::commands::storage_clear,
            features::about::commands::app_info,
            features::about::commands::changelog,
            features::logging::commands::read_logs,
            features::logging::commands::export_logs,
            features::logging::commands::open_log_dir,
            features::autostart::commands::autostart_status,
            features::autostart::commands::set_autostart,
            features::tray::commands::set_close_to_tray,
            features::maintenance::commands::set_maintenance_schedule,
            features::maintenance::commands::next_maintenance,
            features::alerts::commands::set_alerts_enabled,
            features::alerts::commands::list_alerts,
            features::alerts::commands::refresh_alerts,
            features::alerts::commands::mark_alerts_read,
            features::kv::commands::kv_get,
            features::kv::commands::kv_set,
            features::kv::commands::kv_delete,
            features::export::commands::save_text_file,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|handle, event| {
        if let RunEvent::Exit = event {
            log::info!("Deadlock+ exiting");
            handle.state::<NetworkMonitor>().stop();
            handle.state::<IngestService>().stop();
        }
    });
}
