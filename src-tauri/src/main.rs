// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod lib;

use crate::lib::central_lib::dependencies_check;
use crate::lib::model_lib::{
    AudioPlayer, GeneralSignal, GeneralVars, MediaControlsInternal, PlaylistKa,
};
use lib::model_lib::AudioPlayerStatus;
use rodio::Sink;
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, PlatformConfig};
use std::ffi::c_void;
use std::ops::Deref;
use std::process::exit;
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::{env, fs, io, thread};
use tauri::State;

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::HWND;
#[cfg(target_os = "windows")]
use windows::Win32::System::Console::GetConsoleWindow;

#[tauri::command]
fn add_audio(
    state: tauri::State<Arc<Mutex<Option<AudioPlayer>>>>,
    title_audio: &str,
) -> (String, bool) {
    let mut return_value = (String::from(""), false);
    let title_audio_clone = title_audio.clone();

    if !(title_audio.is_empty()) {
        let ap_local_clone = state.inner().clone();
        let mut binding_ap = ap_local_clone.lock().unwrap();
        let mut binding_ap_none = AudioPlayer::new_none();
        let ap = binding_ap.as_mut().unwrap_or(&mut binding_ap_none);

        // check update title
        if !(matches!(*ap.status.lock().unwrap(), AudioPlayerStatus::Play)) {
            return_value = (String::from(title_audio_clone), true);
        } else {
            return_value = (String::from(title_audio_clone), false);
        }

        ap.add_audio(&title_audio);
    }

    return return_value;
}

#[tauri::command]
fn resume_pause(state: tauri::State<Arc<Mutex<Option<AudioPlayer>>>>) -> bool {
    let ap_local_clone = state.inner().clone();
    let mut binding_ap = ap_local_clone.lock().unwrap();
    let mut binding_ap_none = AudioPlayer::new_none();
    let ap = binding_ap.as_mut().unwrap_or(&mut binding_ap_none);

    if ap.is_paused() {
        ap.resume();
    } else {
        ap.pause();
    }

    let status = matches!(*ap.status.lock().unwrap(), AudioPlayerStatus::Play);
    status
}

#[tauri::command]
fn get_cta(state: tauri::State<Arc<Mutex<Option<AudioPlayer>>>>) -> String {
    let ap_local_clone = state.inner().clone();
    let mut binding_ap = ap_local_clone.lock().unwrap();
    let mut binding_ap_none = AudioPlayer::new_none();
    let ap = binding_ap.as_mut().unwrap_or(&mut binding_ap_none);

    if (*ap.nb_audios.lock().unwrap()) > 0 {
        let current_nb_audios_clone_u32 = *ap.current_nb_audios.clone().lock().unwrap();
        ap.list_audio
            .get((current_nb_audios_clone_u32 - 1) as usize)
            .map_or_else(
                || String::from("None"),
                |inner_value| {
                    inner_value
                        .audios
                        .get(inner_value.current_audio_index as usize)
                        .unwrap()
                        .title
                        .to_string()
                },
            )
    } else {
        "Audio".to_string()
    }
}

#[tauri::command]
fn next(state: tauri::State<Arc<Mutex<Option<AudioPlayer>>>>) {
    let ap_local_clone = state.inner().clone();
    let mut binding_ap = ap_local_clone.lock().unwrap();
    let mut binding_ap_none = AudioPlayer::new_none();
    let ap = binding_ap.as_mut().unwrap_or(&mut binding_ap_none);

    ap.next_audio();
}

#[tauri::command]
fn previous(state: tauri::State<Arc<Mutex<Option<AudioPlayer>>>>) {
    let ap_local_clone = state.inner().clone();
    let mut binding_ap = ap_local_clone.lock().unwrap();
    let mut binding_ap_none = AudioPlayer::new_none();
    let ap = binding_ap.as_mut().unwrap_or(&mut binding_ap_none);

    ap.prev_audio();
}

#[tauri::command]
fn get_list_audio(state: tauri::State<Arc<Mutex<Option<AudioPlayer>>>>) -> Vec<String> {
    let ap_local_clone = state.inner().clone();
    let mut binding_ap = ap_local_clone.lock().unwrap();
    let mut binding_ap_none = AudioPlayer::new_none();
    let ap = binding_ap.as_mut().unwrap_or(&mut binding_ap_none);

    if (*ap.nb_audios.lock().unwrap()) > 0 {
        ap.print_audio_list()
    } else {
        Vec::new()
    }
}

#[tauri::command]
fn get_volume(state: tauri::State<Arc<Mutex<Option<AudioPlayer>>>>) -> i32 {
    let ap_local_clone = state.inner().clone();
    let mut binding_ap = ap_local_clone.lock().unwrap();
    let mut binding_ap_none = AudioPlayer::new_none();
    let ap = binding_ap.as_mut().unwrap_or(&mut binding_ap_none);
    let current_volume_value = (*ap.volume.lock().unwrap()).clone();
    current_volume_value
}

#[tauri::command]
fn set_volume(state: tauri::State<Arc<Mutex<Option<AudioPlayer>>>>, volume_value: i32) {
    let ap_local_clone = state.inner().clone();
    let mut binding_ap = ap_local_clone.lock().unwrap();
    let mut binding_ap_none = AudioPlayer::new_none();
    let ap = binding_ap.as_mut().unwrap_or(&mut binding_ap_none);
    *ap.volume.lock().unwrap() = volume_value;
    ap.update_volume();
}

fn init() -> io::Result<()> {
    // Directory where the files are located
    let mut dir_path = env::current_exe().expect("Failed to get executable path");
    dir_path.pop();
    dir_path.push("data");

    let entries = fs::read_dir(dir_path.to_string_lossy().to_string())?;

    for entry in entries {
        let entry = entry?;

        if let Some(file_name) = entry.file_name().to_str() {
            if file_name.starts_with("audio") && file_name.ends_with(".wav") {
                let file_path = entry.path();

                if file_path.is_file() {
                    fs::remove_file(file_path.clone())?;
                    println!("Removed file: {:?}", file_path);
                }
            }
        }
    }

    Ok(())
}

fn main() {
    //--------------------------------- AUDIO INIT ---------------------------------
    let deps_status = dependencies_check();

    if deps_status {
        let mut gv = GeneralVars::new();

        let mut controls = Arc::new(Mutex::new(None));

        //--------------------------------- AUDIO INIT ---------------------------------
        let (_stream, stream_handle) = rodio::OutputStream::try_default().unwrap();
        let sink_raw = Sink::try_new(&stream_handle);

        let sink = match sink_raw {
            Ok(sink_result) => sink_result,
            Err(error) => panic!("ERROR : audio unavailable !!! : {}", error),
        };

        let audio_player = Arc::new(Mutex::new(sink));
        //--------------------------------- AUDIO PLAYER INIT ---------------------------------
        let ap = Arc::new(Mutex::new(Some(AudioPlayer::new(audio_player))));

        //--------------------------------- CONT PlaylistKa INIT ---------------------------------
        let cont_pk: Arc<Mutex<Vec<PlaylistKa>>> = Arc::new(Mutex::new(Vec::new()));

        //--------------------------------- MediaControlInternal INIT ---------------------------------
        #[cfg(not(target_os = "windows"))]
        let hwnd = None;

        #[cfg(target_os = "windows")]
        let mut hwnd = {
            let mut re_hwnd = None;
            let mut raw_hwnd = unsafe { GetConsoleWindow() };
            match raw_hwnd.0 {
                0 => println!("Error getting console window handle"),
                pre_hwnd => {
                    println!("Console window handle: {:?}", pre_hwnd);
                    re_hwnd = Some(pre_hwnd as *mut c_void);
                }
            }
            re_hwnd
        };
        if (cfg!(target_os = "windows") && !(matches!(hwnd, None)))
            || !(cfg!(target_os = "windows"))
        {
            let config = PlatformConfig {
                dbus_name: "my_player",
                display_name: "My Player",
                hwnd: hwnd,
            };
            match MediaControls::new(config) {
                Ok(mc) => {
                    controls = Arc::new(Mutex::new(Some(MediaControlsInternal::new(mc))));
                }
                Err(error) => {
                    println!("no media available {:?}", error);
                    println!("{:?}", hwnd);
                }
            }
        }

        gv.ap = ap;
        gv.c_pk = cont_pk;
        gv.mci = controls;

        let gv_thread_main = gv.clone();

        let gv_thread_player_check = gv.clone();
        let mut gv_thread_media_controls = gv.clone();

        let _thread_player_check = thread::spawn(move || {
            AudioPlayer::start_auto_next(
                gv_thread_player_check.ap.clone(),
                gv_thread_player_check.mci.clone(),
            );
        });

        let _thread_media_controls = thread::spawn(move || {
            let local_mci = gv_thread_media_controls.mci.lock().unwrap();
            if !(local_mci.is_none()) {
                drop(local_mci);
                MediaControlsInternal::attach_os_notify(
                    &mut gv_thread_media_controls.ap.clone(),
                    &mut gv_thread_media_controls.mci.clone(),
                );
            }
        });

        let mut cont_pk: Arc<Mutex<Vec<PlaylistKa>>> = gv_thread_main.c_pk;
        let mut ap = gv_thread_main.ap;
        let _ = init();
        tauri::Builder::default()
            .manage(ap)
            .manage(cont_pk)
            .invoke_handler(tauri::generate_handler![
                add_audio,
                resume_pause,
                next,
                previous,
                get_cta,
                get_list_audio,
                get_volume,
                set_volume
            ])
            .run(tauri::generate_context!())
            .expect("error while running tauri application");
        GeneralSignal::Nothing;
    } else {
        println!("error with dependencies");
    }
}
