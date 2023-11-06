use crate::get_audio_devices;

use super::central_lib::{exec_command_yt_dl_tauri, research_on_yt};
use cpal::traits::HostTrait;
use cpal::{default_host, StreamError};
use rodio::{Decoder, Device, DeviceTrait, Sink, Source};
use souvlaki::{MediaControlEvent, MediaControls, MediaMetadata, MediaPlayback, MediaPosition};
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::BufReader;
use std::rc::Rc;
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;
use std::{env, thread, u32};

#[repr(u32)]
pub enum GeneralSignal {
    Exit = 0,
    InvalidInput = 100,
    Reboot = 200,
    Nothing = 300,
    ValidInput(u32),
}

#[derive(Clone)]
pub struct GeneralVars {
    pub ap: Arc<Mutex<Option<AudioPlayer>>>,
    pub c_pk: Arc<Mutex<Vec<PlaylistKa>>>,
    pub mci: Arc<Mutex<Option<MediaControlsInternal>>>,
}

impl GeneralVars {
    pub fn new() -> Self {
        Self {
            ap: Arc::new(Mutex::new(None)),
            c_pk: Arc::new(Mutex::new(Vec::new())),
            mci: Arc::new(Mutex::new(None)),
        }
    }
}

#[derive(Clone)]
pub enum AudioPlayerStatus {
    Pause,
    Play,
    Empty,
    Disabled,
}

#[derive(Clone)]
pub struct AudioPlayer {
    pub status: Arc<Mutex<AudioPlayerStatus>>,
    pub current_nb_audios: Arc<Mutex<u32>>,
    pub nb_audios: Arc<Mutex<u32>>,
    pub current_output_device: Arc<Mutex<Option<String>>>,
    pub list_audio: Vec<PlaylistKa>,
    pub queue_audio: Vec<PlaylistKa>,
    pub thread_status: Arc<Mutex<bool>>,
    pub current_audio_time: Arc<Mutex<Duration>>,
    pub volume: Arc<Mutex<u32>>,
    pub current_audio_sink_sender: Arc<Mutex<Option<Sender<String>>>>,
}

impl AudioPlayer {
    pub fn new() -> Self {
        Self {
            status: Arc::new(Mutex::new(AudioPlayerStatus::Empty)),
            current_nb_audios: Arc::new(Mutex::new(0)),
            nb_audios: Arc::new(Mutex::new(0)),
            current_output_device: Arc::new(Mutex::new(None)),
            list_audio: Vec::new(),
            queue_audio: Vec::new(),
            thread_status: Arc::new(Mutex::new(true)),
            current_audio_time: Arc::new(Mutex::new(Duration::new(0, 0))),
            volume: Arc::new(Mutex::new(100)),
            current_audio_sink_sender: Arc::new(Mutex::new(None)),
        }
    }

    pub fn new_none() -> Self {
        Self {
            status: Arc::new(Mutex::new(AudioPlayerStatus::Disabled)),
            current_nb_audios: Arc::new(Mutex::new(0)),
            nb_audios: Arc::new(Mutex::new(0)),
            current_output_device: Arc::new(Mutex::new(None)),
            list_audio: Vec::new(),
            queue_audio: Vec::new(),
            thread_status: Arc::new(Mutex::new(true)),
            current_audio_time: Arc::new(Mutex::new(Duration::new(0, 0))),
            volume: Arc::new(Mutex::new(100)),
            current_audio_sink_sender: Arc::new(Mutex::new(None)),
        }
    }

    pub fn start_auto_next(
        ap_raw: Arc<Mutex<Option<AudioPlayer>>>,
        mci_raw: Arc<Mutex<Option<MediaControlsInternal>>>,
    ) {
        let ap_clone = ap_raw.clone();
        let mut binding_ap = ap_clone.lock().unwrap();
        let mut binding_ap_none = AudioPlayer::new_none();
        let ap = binding_ap.as_mut().unwrap_or(&mut binding_ap_none);

        let mci_clone = mci_raw.clone();
        let mut binding_mci = mci_clone.lock().unwrap();

        if !(matches!(*ap.status.lock().unwrap(), AudioPlayerStatus::Disabled)) {
            let ap_clone_thread = ap.clone();
            let ap_clone_thread_2 = ap.clone();

            drop(binding_ap);

            thread::spawn(move || {
                let mut ap = ap_clone_thread;

                loop {
                    if *ap.thread_status.lock().unwrap() {
                        if ap.is_nextable() {
                            while !(matches!(*ap.status.lock().unwrap(), AudioPlayerStatus::Empty))
                            {
                                sleep(Duration::new(1, 0));
                                let current_sender_option_clone = ap.current_audio_sink_sender.clone();
                                let current_sender_option = current_sender_option_clone.lock().unwrap().clone();
                                //destoy old one if existing
                                match current_sender_option {
                                    Some(tx) => {
                                        match tx.send("status".to_string()) {
                                            Ok(_) => {}
                                            Err(_) => {}
                                        };
                                    }
                                    None => {}
                                }
                            }
                            println!("neeeeeeeeeext");
                            *ap.status.lock().unwrap() = AudioPlayerStatus::Pause;
                            ap.next_audio();
                        }
                        sleep(Duration::new(1, 0));
                    } else {
                        break;
                    }
                }
            });

            // OS media system
            if !(binding_mci.as_mut().is_none()) {
                thread::spawn(move || {
                    let mci_local_clone = mci_raw.clone();
                    let mut mci_local = mci_local_clone.lock().unwrap();
                    let mut ap = ap_clone_thread_2;

                    loop {
                        if *ap.thread_status.lock().unwrap() {
                            match *ap.status.lock().unwrap() {
                                AudioPlayerStatus::Play => {
                                    mci_local
                                        .as_mut()
                                        .unwrap()
                                        .media_controls
                                        .set_playback(MediaPlayback::Playing { progress: None })
                                        .expect("set playing : panic message");
                                }
                                AudioPlayerStatus::Pause => {
                                    mci_local
                                        .as_mut()
                                        .unwrap()
                                        .media_controls
                                        .set_playback(MediaPlayback::Paused { progress: None })
                                        .expect("set pause : panic message");
                                }
                                _ => {}
                            }
                            sleep(Duration::new(1, 0));
                        } else {
                            break;
                        }
                    }
                });
            }
        }
    }

    pub fn add_audio(&mut self, title: &str) {
        let string_title = title.clone();
        let nb_audio_clone = self.nb_audios.clone();
        let current_nb_audios_clone = self.current_nb_audios.clone();

        let nb_audio_clone_u32 = *nb_audio_clone.lock().unwrap();
        *nb_audio_clone.lock().unwrap() = nb_audio_clone_u32 + 1;

        let (file_name, title) = self.research_download(Some(string_title.to_string()));
        println!("Recherche finis!!!");

        let pk_current_audio =
            PlaylistKa::new(string_title.to_string(), vec![AudioKa::new_simple(title)]);
        self.list_audio.push(pk_current_audio.clone());
        self.queue_audio.push(pk_current_audio.clone());

        // init
        if *nb_audio_clone.lock().unwrap() == 1 {
            let current_nb_audios_clone_u32 = *current_nb_audios_clone.lock().unwrap();
            *current_nb_audios_clone.lock().unwrap() = current_nb_audios_clone_u32 + 1;

            self.play_audio();

            *self.status.lock().unwrap() = AudioPlayerStatus::Play;
        }
    }

    pub fn pause(&mut self) {
        if matches!(*self.status.lock().unwrap(), AudioPlayerStatus::Play) {
            let current_sender_option_clone = self.current_audio_sink_sender.clone();
            let current_sender_option = current_sender_option_clone.lock().unwrap().clone();
            match current_sender_option {
                Some(tx) => {
                    tx.send("pause".to_string()).unwrap();
                }
                None => {}
            }
            *self.status.lock().unwrap() = AudioPlayerStatus::Pause;
        }
    }

    pub fn resume(&mut self) {
        if matches!(*self.status.lock().unwrap(), AudioPlayerStatus::Pause) {
            let current_sender_option_clone = self.current_audio_sink_sender.clone();
            let current_sender_option = current_sender_option_clone.lock().unwrap().clone();
            match current_sender_option {
                Some(tx) => {
                    tx.send("play".to_string()).unwrap();
                }
                None => {}
            }
            *self.status.lock().unwrap() = AudioPlayerStatus::Play;
        }
    }

    pub fn is_paused(&mut self) -> bool {
        return matches!(*self.status.lock().unwrap(), AudioPlayerStatus::Pause);
    }

    pub fn is_nextable(&mut self) -> bool {
        let nb_audio_clone = self.nb_audios.clone();
        let current_nb_audios_clone = self.current_nb_audios.clone();
        let nb_audio_clone_u32 = *nb_audio_clone.lock().unwrap();
        let current_nb_audios_clone_u32 = *current_nb_audios_clone.lock().unwrap();

        return current_nb_audios_clone_u32 < nb_audio_clone_u32 && nb_audio_clone_u32 > 1;
    }

    pub fn is_previousable(&mut self) -> bool {
        let nb_audio_clone = self.nb_audios.clone();
        let current_nb_audios_clone = self.current_nb_audios.clone();
        let nb_audio_clone_u32 = *nb_audio_clone.lock().unwrap();
        let current_nb_audios_clone_u32 = *current_nb_audios_clone.lock().unwrap();

        return current_nb_audios_clone_u32 > 1 && nb_audio_clone_u32 > 1;
    }

    pub fn print_audio_list(&mut self) -> Vec<String> {
        let mut list_a = vec![];
        for playlistka in &self.list_audio {
            for audioka in &playlistka.audios {
                list_a.push(audioka.title.clone());
                // println!("{}", audioka.title);
            }
        }
        list_a
    }

    pub fn next_audio(&mut self) {
        if !(matches!(*self.status.lock().unwrap(), AudioPlayerStatus::Empty)) {
            if self.is_nextable() {
                self.destroy_old_audio();

                let current_nb_audios_clone = self.current_nb_audios.clone();

                let current_nb_audios_clone_u32 = *current_nb_audios_clone.lock().unwrap();
                *self.current_nb_audios.lock().unwrap() = current_nb_audios_clone_u32 + 1;

                self.play_audio();

                *self.status.lock().unwrap() = AudioPlayerStatus::Play;
            }
        }
    }

    pub fn prev_audio(&mut self) {
        if !(matches!(*self.status.lock().unwrap(), AudioPlayerStatus::Empty)) {
            if self.is_previousable() {
                self.destroy_old_audio();

                let current_nb_audios_clone = self.current_nb_audios.clone();

                let current_nb_audios_clone_u32 = *current_nb_audios_clone.lock().unwrap();
                *self.current_nb_audios.lock().unwrap() = current_nb_audios_clone_u32 - 1;

                self.play_audio();

                *self.status.lock().unwrap() = AudioPlayerStatus::Play;
            }
        }
    }

    fn research_download(&mut self, input_research: Option<String>) -> (String, String) {
        // Research on youtube
        // tui_print(format!("Researching : {}", input_research.clone().unwrap()).as_str());
        let (results, title) = research_on_yt(&input_research.clone().unwrap());

        // Download and create file
        // println!("Executing command !! ");
        // tui_print(format!("Downloading : {}", input_research.clone().unwrap()).as_str());
        let file_name = format!("audio{}", *self.nb_audios.lock().unwrap());
        exec_command_yt_dl_tauri(&results[0], &file_name);

        return (file_name, title);
    }

    fn destroy_old_audio(&mut self) {
        let current_sender_option_clone = self.current_audio_sink_sender.clone();
        let current_sender_option = current_sender_option_clone.lock().unwrap().clone();
        //destoy old one if existing
        match current_sender_option {
            Some(tx) => {
                match tx.send("destroy".to_string()){
                    Ok(_) => {}
                    Err(_) => {}
                };
            }
            None => {}
        }
    }

    pub fn play_audio(&mut self) {
        let current_audio_time_clone = self.current_audio_time.clone();
        let volume = self.volume.clone();
        let device_name_option_raw = self.current_output_device.clone();
        let device_name_option = device_name_option_raw.lock().unwrap().clone();
        let current_audio_statut_clone = self.status.clone();

        let file_name = format!("audio{}.wav", *self.current_nb_audios.lock().unwrap());

        let mut path = env::current_exe().expect("Failed to get executable path");
        path.pop();
        path.push("data");
        path.push(file_name);

        let (tx, rx) = mpsc::channel();

        *self.current_audio_sink_sender.lock().unwrap() = Some(tx);

        thread::spawn(move || {
            let mut current_audio_time = current_audio_time_clone.lock().unwrap();

            AudioPlayer::audio_thread(
                rx,
                device_name_option,
                path.to_string_lossy().to_string().clone(),
                volume,
                current_audio_statut_clone,
                &mut current_audio_time,
            );
        });
    }

    pub fn update_volume(&mut self) {
        if !(matches!(*self.status.lock().unwrap(), AudioPlayerStatus::Disabled)) {
            let current_sender_option_clone = self.current_audio_sink_sender.clone();
            let current_sender_option = current_sender_option_clone.lock().unwrap().clone();
            match current_sender_option {
                Some(tx) => {
                    tx.send("volume".to_string()).unwrap();
                }
                None => {}
            }
        }
    }

    fn get_output_device(device_name_option: Option<String>) -> Option<Device> {
        let mut option_device = None;
        match device_name_option {
            Some(dev_name) => {
                let devices_raw = default_host().output_devices();
                match devices_raw {
                    Ok(devices_raw_success) => {
                        for device in devices_raw_success {
                            match device.name() {
                                Ok(device_name) => {
                                    if dev_name.clone() == device_name {
                                        option_device = Some(device);
                                    }
                                }
                                Err(_) => {}
                            }
                        }
                    }
                    Err(_) => {}
                }
            }
            None => match default_host().default_output_device() {
                Some(device) => {
                    option_device = Some(device);
                }
                None => {}
            },
        }
        option_device
    }
    fn audio_thread(
        rx: mpsc::Receiver<String>,
        device_name_option: Option<String>,
        path: String,
        volume: Arc<Mutex<u32>>,
        current_audio_statut: Arc<Mutex<AudioPlayerStatus>>,
        current_audio_time: &mut Duration,
    ) {
        match Self::get_output_device(device_name_option) {
            Some(device) => match rodio::OutputStream::try_from_device(&device) {
                Ok((_stream, stream_handle)) => {
                    let mut iter = 0;

                    while (iter <= 10) {
                        let file_raw = File::open(&path);
                        match file_raw {
                            Ok(file) => {
                                match rodio::Decoder::new(BufReader::new(file)) {
                                    Ok(audio_source) => {
                                        let mut sink_n = Sink::try_new(&stream_handle);

                                        match sink_n {
                                            Ok(sink) => {
                                                *current_audio_time = audio_source
                                                    .total_duration()
                                                    .unwrap_or(Duration::new(0, 0));
                                                println!("around {:?} seconds", current_audio_time);
                                                println!("---------------------------------------");

                                                sink.append(audio_source);
                                                sink.play();
                                                let mut destroy = false;
                                                while !(destroy) {
                                                    match rx.recv() {
                                                        Ok(command) => {
                                                            match command.as_str() {
                                                                "play" => {
                                                                    // Play audio
                                                                    sink.play();
                                                                }
                                                                "pause" => {
                                                                    // Pause audio
                                                                    sink.pause();
                                                                }
                                                                "volume" => {
                                                                    // drop audio
                                                                    sink.set_volume(
                                                                        volume
                                                                            .lock()
                                                                            .unwrap()
                                                                            .clone()
                                                                            as f32
                                                                            / 100.0,
                                                                    );
                                                                }
                                                                "stop" => {
                                                                    // Stop audio
                                                                    sink.stop();
                                                                }
                                                                "destroy" => {
                                                                    // drop audio
                                                                    destroy = true;
                                                                }
                                                                "status" => {
                                                                    if sink.empty(){
                                                                        *current_audio_statut.lock().unwrap() = AudioPlayerStatus::Empty;
                                                                        destroy = true;
                                                                    }
                                                                }
                                                                _ => {}
                                                            }
                                                        }
                                                        Err(_) => {} // Exit the thread on channel disconnect
                                                    }
                                                }
                                                println!("out");
                                                break;
                                            }
                                            Err(_) => {}
                                        }
                                    }
                                    Err(_) => {}
                                }
                            }
                            Err(error) => {
                                iter = iter + 1;

                                println!("Error: audio file problem : {}", error);
                                println!("{:?}", path.clone());
                                sleep(Duration::new(2, 0))
                            }
                        }
                    }
                }
                Err(_) => {}
            },
            None => {}
        }
    }
}

#[derive(Clone)]
pub struct PlaylistKa {
    pub title: String,
    pub audios: Vec<AudioKa>,
    pub current_audio_index: u32,
}

impl PlaylistKa {
    pub fn new(title: String, audios: Vec<AudioKa>) -> Self {
        Self {
            title,
            audios,
            current_audio_index: 0,
        }
    }
    pub fn add(&mut self, ak: AudioKa) {
        self.audios.push(ak);
    }
}

// Implement the Display trait for PlaylistKa
impl Display for PlaylistKa {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let formatted_playlist = format!(
            "PlaylistKa {{\n    title: {},\n    audios: {:?},\n    current_audio_index: {}\n}}",
            self.title, self.audios, self.current_audio_index
        );

        write!(f, "{}", formatted_playlist)?;

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct AudioKa {
    pub title: String,
    pub url: String,
    // time: f32,
}

impl AudioKa {
    pub fn new(title: String, url: String) -> Self {
        Self { title, url }
    }
    pub fn new_simple(title: String) -> Self {
        Self {
            title,
            url: String::from(""),
        }
    }
}

// Implement the Display trait for AudioKa
impl Display for AudioKa {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
        let formatted_playlist = format!(
            "AudioKa {{\n    title: {},\n    url: {:?},\n}}",
            self.title, self.url,
        );

        write!(f, "{}", formatted_playlist)?;

        Ok(())
    }
}

pub struct MediaControlsInternal {
    pub media_controls: MediaControls,
    pub media_controls_status: bool,
}

impl MediaControlsInternal {
    pub fn new(mc: MediaControls) -> Self {
        Self {
            media_controls: mc,
            media_controls_status: true,
        }
    }

    pub fn attach_os_notify(
        ap_raw: &mut Arc<Mutex<Option<AudioPlayer>>>,
        mci_raw: &mut Arc<Mutex<Option<MediaControlsInternal>>>,
    ) {
        let ap_clone = ap_raw.clone();
        let mut binding_ap = ap_clone.lock().unwrap();
        let mut binding_ap_none = AudioPlayer::new_none();
        let ap = binding_ap.as_mut().unwrap_or(&mut binding_ap_none);

        if !(matches!(*ap.status.lock().unwrap(), AudioPlayerStatus::Disabled)) {
            let ap_clone_thread = ap.clone();

            if let Some(mci) = mci_raw.lock().unwrap().as_mut() {
                mci.media_controls
                    .attach(move |event: MediaControlEvent| {
                        let mut ap = ap_clone_thread.clone();
                        match event {
                            MediaControlEvent::Pause => {
                                ap.pause();
                            }
                            MediaControlEvent::Play => {
                                ap.resume();
                            }
                            MediaControlEvent::Next => {
                                ap.next_audio();
                            }
                            MediaControlEvent::Previous => {
                                ap.prev_audio();
                            }
                            _ => {}
                        }
                    })
                    .expect("Media Control panic");
            }
        }
    }
}
