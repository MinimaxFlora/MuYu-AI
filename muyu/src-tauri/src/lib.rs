#![recursion_limit = "512"]

mod commands;
mod jianying;
mod llm;
mod media;
mod model;
mod pipeline;
mod render;
mod settings;
mod store;
mod subtitle;
mod tts;
mod updater;

use commands::{
    analyze_plot, attach_subtitle, cancel_render, cancel_tts, check_environment, create_project,
    delete_project, export_items, export_jianying, export_srt, generate_copy, generate_items,
    get_project, get_settings, list_projects, list_tts_voices, open_log_dir, open_path,
    preview_voice, probe_media, remove_subtitle, render_video, reveal_project, save_project,
    save_settings, synthesize_items, test_llm,
};
use updater::{check_update, install_update};
use tauri::{
    menu::{Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, WindowEvent,
};
use tauri_plugin_autostart::MacosLauncher;

fn show_main(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tts::ensure_crypto();
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            show_main(app);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some("muyu".into()),
                    }),
                ])
                .level(tauri_plugin_log::log::LevelFilter::Info)
                .max_file_size(2_000_000)
                .build(),
        )
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, None))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            std::panic::set_hook(Box::new(|info| {
                tauri_plugin_log::log::error!("panic: {info}");
            }));
            tauri_plugin_log::log::info!("MuYu started");
            let update_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                tokio::time::sleep(std::time::Duration::from_secs(8)).await;
                match updater::check_auto(&update_handle).await {
                    Ok(Some(info)) => {
                        let _ = update_handle.emit("updater://available", info);
                    }
                    Ok(None) => {}
                    Err(error) => {
                        tauri_plugin_log::log::warn!("auto update check failed: {error}");
                    }
                }
            });
            let show = MenuItem::with_id(app, "show", "显示主窗口", true, None::<&str>)?;
            let projects = MenuItem::with_id(app, "projects", "打开项目库", true, None::<&str>)?;
            let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出 MuYu", true, None::<&str>)?;
            let separator = PredefinedMenuItem::separator(app)?;
            let menu = Menu::with_items(app, &[&show, &projects, &settings, &separator, &quit])?;

            let mut tray = TrayIconBuilder::with_id("main-tray")
                .menu(&menu)
                .tooltip("MuYu · 幕语")
                .show_menu_on_left_click(false)
                .on_menu_event(|app, event| match event.id.as_ref() {
                    "show" => show_main(app),
                    "projects" => {
                        show_main(app);
                        let _ = app.emit("tray://navigate", "projects");
                    }
                    "settings" => {
                        show_main(app);
                        let _ = app.emit("tray://navigate", "settings");
                    }
                    "quit" => app.exit(0),
                    _ => {}
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        show_main(tray.app_handle());
                    }
                });
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.build(app)?;
            Ok(())
        })
        .on_window_event(|window, event| {
            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                tauri_plugin_log::log::info!("main window hidden to tray");
                let _ = window.hide();
            }
        })
        .invoke_handler(tauri::generate_handler![
            create_project,
            list_projects,
            get_project,
            save_project,
            attach_subtitle,
            remove_subtitle,
            delete_project,
            probe_media,
            check_environment,
            reveal_project,
            get_settings,
            save_settings,
            test_llm,
            analyze_plot,
            generate_copy,
            generate_items,
            list_tts_voices,
            preview_voice,
            synthesize_items,
            cancel_tts,
            render_video,
            cancel_render,
            export_srt,
            export_items,
            export_jianying,
            open_path,
            open_log_dir,
            check_update,
            install_update
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
