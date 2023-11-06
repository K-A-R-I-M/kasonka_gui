use regex::Regex;
use reqwest;
use reqwest::get;
use scraper::{Html, Selector};
use std::{env, fs};
use tauri::api::process::{Command as Com, CommandEvent};
use std::fs::File;

fn get_yt_page(req: &str) -> Result<String, Box<dyn std::error::Error>> {
    let yt_page_txt = reqwest::blocking::get(
        String::from("https://www.youtube.com/results?search_query=") + req,
    )?
    .text()?;
    return Ok(yt_page_txt);
}

pub fn research_on_yt(research_txt: &str) -> (Vec<String>, String) {
    let mut research_results: Vec<String> = Vec::new();
    let mut title = String::new();
    let yt_page_txt = get_yt_page(research_txt).unwrap();
    let document = Html::parse_document(&yt_page_txt);
    let scripts = Selector::parse("script").unwrap();

    for script in document.select(&scripts) {
        let script_txt = script.text().collect::<Vec<_>>();
        if script_txt.len() > 0 {
            match script_txt[0].find("watch?v=") {
                Some(_) => {
                    let re = Regex::new(r#"watch\?v=.*?""#).unwrap();
                    for matching_pat in re.find_iter(script_txt[0]) {
                        let url = matching_pat.as_str().split("\\").collect::<Vec<&str>>()[0];
                        research_results.push(String::from(url));
                    }
                }
                None => {}
            }
            match script_txt[0].find("\"title\":") {
                Some(_) => {
                    let re = Regex::new(r#"\"title\":\{.*?\}"#).unwrap();
                    title = re.find_iter(script_txt[0]).collect::<Vec<_>>()[0]
                        .as_str()
                        .replace("\"title\":{\"runs\":[{\"text\":\"", "");
                    // removing 2 last chars = "]}
                    title.pop();
                    title.pop();
                    // println!("{}", title.as_str());
                }
                None => {}
            }
        }
    }
    return (research_results, title.to_string());
}

pub fn exec_command_yt_dl_tauri(url: &str, file_name: &str) {
    // Linux
    let mut command = "utils/yt-dlp";
    let mut ffmpeg = "ffmpeg";
    // Windows
    if cfg!(target_os = "windows") {
        command = "utils\\yt-dlp-x86_64-pc-windows-msvc";
        ffmpeg = "ffmpeg-x86_64-pc-windows-msvc.exe";
    }

    let mut data_path = env::current_exe().expect("Failed to get executable path");
    data_path.pop(); // Remove the executable name from the path
    data_path.push("data");
    data_path.push(format!("{}.%(ext)s", file_name));

    let mut path_dir_bin = env::current_exe().expect("Failed to get executable path");
    path_dir_bin.pop();
    path_dir_bin.push("utils");
    path_dir_bin.push(ffmpeg);

    // `new_sidecar()` attend juste le nom du fichier, PAS le chemin complet comme en JavaScript
    let (mut rx, mut child) = Com::new_sidecar(command)
        .expect("échec de la création de la commande binaire `yt-dlp`")
        .args([
            "-vU",
            "-x",
            "--audio-format",
            "wav",
            "-q",
            "--progress",
            "--ffmpeg-location",
            &path_dir_bin.to_string_lossy().to_string(),
            "-o",
            &data_path.to_string_lossy().to_string(),
            &format!("https://www.youtube.com/{}", url),
        ])
        .spawn()
        .expect("Échec de l'exécution de l'application");
    // println!("{:?}", child);
}

pub fn dependencies_check() -> bool {
    let mut file_names = vec!["data"];

    #[cfg(not(target_os = "windows"))]
    let mut file_names_os = vec!["utils/yt-dlp", "utils/ffmpeg"];
    #[cfg(target_os = "windows")]
    let mut file_names_os = vec![
        "utils/yt-dlp-x86_64-pc-windows-msvc.exe",
        "utils/ffmpeg-x86_64-pc-windows-msvc.exe",
    ];

    file_names.append(&mut file_names_os);

    for file_name in file_names {
        if fs::metadata(&file_name).is_ok() {
            println!("The file {} exists", file_name);
        } else {
            println!("The file {} does not exist", file_name);

            //------- data folder creation
            if file_name == "data" {
                match fs::create_dir("data") {
                    Err(e) => {
                        println!("ERROR: couldn't create data folder : {}", e);
                    }
                    _ => {}
                }
            }

            return false;
        }
    }

    return true;
}
