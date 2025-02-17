#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // to suppress console with debug output for release builds
///
/// swyh-rs
///
/// Basic SWYH (https://www.streamwhatyouhear.com/, source repo https://github.com/StreamWhatYouHear/SWYH) clone entirely written in rust.
///
/// I wrote this because I a) wanted to learn Rust and b) SWYH did not work on Linux and did not work well with Volumio (push streaming does not work).
///
/// For the moment all music is streamed in wav-format (audio/l16) with the sample rate of the music source (the default audio device, I use HiFi Cable Input).
///
/// Tested on Windows 10 and on Ubuntu 20.04 with Raspberry Pi based Volumio DLNA renderers and with a Harman-Kardon AVR DLNA device.
/// I don't have access to a Mac, so I don't know if this would also work.
///
///
/*
MIT License

Copyright (c) 2020 dheijl

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

#[macro_use]
extern crate bitflags;

mod openhome;
mod server;
mod ui;
mod utils;

use crate::openhome::avmedia::{discover, Renderer, WavData};
use crate::server::streaming_server::run_server;
use crate::ui::mainform::MainForm;
use crate::utils::audiodevices::{
    capture_output_audio, get_default_audio_output_device, get_output_audio_devices,
};
use crate::utils::configuration::Configuration;
use crate::utils::local_ip_address::*;
use crate::utils::priority::raise_priority;
use crate::utils::rwstream::ChannelStream;
use cpal::traits::{DeviceTrait, StreamTrait};
use cpal::Sample;
use crossbeam_channel::{unbounded, Receiver, Sender};
use fltk::{
    app, dialog,
    misc::Progress,
    prelude::{ButtonExt, WidgetExt},
};
use lazy_static::lazy_static;
use log::{debug, error, info, warn, LevelFilter};
use parking_lot::RwLock;
use simplelog::{ColorChoice, CombinedLogger, Config, TermLogger, WriteLogger};
use std::cell::Cell;
use std::collections::HashMap;
use std::fs::File;
use std::net::IpAddr;
use std::path::Path;
use std::rc::Rc;
use std::time::Duration;

/// app version
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// the HTTP server port
pub const SERVER_PORT: u16 = 5901;

/// streaming state
#[derive(Debug, Clone, Copy)]
pub enum StreamingState {
    Started,
    Ended,
}

impl PartialEq for StreamingState {
    fn eq(&self, other: &Self) -> bool {
        self == other
    }
}

/// streaming state feedback for a client
#[derive(Debug, Clone, PartialEq)]
pub struct StreamerFeedBack {
    remote_ip: String,
    streaming_state: StreamingState,
}

lazy_static! {
    // streaming clients of the webserver
    static ref CLIENTS: RwLock<HashMap<String, ChannelStream>> = RwLock::new(HashMap::new());
    // the global GUI logger textbox channel used by all threads
    static ref LOGCHANNEL: RwLock<(Sender<String>, Receiver<String>)> = RwLock::new(unbounded());
    // the global configuration state
    static ref CONFIG: RwLock<Configuration> = RwLock::new(Configuration::read_config());
}

/// swyh-rs
///
/// - set up the fltk GUI
/// - setup and start audio capture
/// - start the streaming webserver
/// - start ssdp discovery of media renderers thread
/// - run the GUI, and show any renderers found in the GUI as buttons (to start/stop playing)
fn main() {
    // first initialize cpal audio to prevent COM reinitialize panic on Windows
    let mut audio_output_device =
        get_default_audio_output_device().expect("No default audio device");

    // initialize config
    let mut config = {
        let mut conf = CONFIG.write();
        if conf.sound_source == "None" {
            conf.sound_source = audio_output_device.name().unwrap();
            let _ = conf.update_config();
        }
        conf.clone()
    };
    ui_log(format!("{config:?}"));
    if cfg!(debug_assertions) {
        config.log_level = LevelFilter::Debug;
    }

    let config_changed: Rc<Cell<bool>> = Rc::new(Cell::new(false));

    // configure simplelogger
    let loglevel = config.log_level;
    let logfile = Path::new(&config.log_dir()).join("log.txt");
    // disable TermLogger on susbsystem Windows because it panics now with Rust edition 2021
    if cfg!(debug_assertions) || cfg!(target_os = "linux") {
        let _ = CombinedLogger::init(vec![
            TermLogger::new(
                loglevel,
                Config::default(),
                simplelog::TerminalMode::Stderr,
                ColorChoice::Auto,
            ),
            WriteLogger::new(loglevel, Config::default(), File::create(logfile).unwrap()),
        ]);
    } else {
        let _ = CombinedLogger::init(vec![WriteLogger::new(
            loglevel,
            Config::default(),
            File::create(logfile).unwrap(),
        )]);
    }
    info!("swyh-rs V {} - Logging started.", APP_VERSION.to_string());
    if cfg!(debug_assertions) {
        ui_log("*W*W*>Running DEBUG build => log level set to DEBUG!".to_string());
    }
    info!("Config: {:?}", config);

    // get the output device from the config and get all available audio source names
    let audio_devices = get_output_audio_devices().unwrap();
    let mut source_names: Vec<String> = Vec::new();
    for adev in audio_devices {
        let devname = adev.name().unwrap();
        if devname == config.sound_source {
            audio_output_device = adev;
            info!("Selected audio source: {}", devname);
        }
        source_names.push(devname);
    }

    // get the default network that connects to the internet
    let local_addr: IpAddr = {
        if config.last_network == "None" {
            let addr = get_local_addr().expect("Could not obtain local address.");
            let mut conf = CONFIG.write();
            conf.last_network = addr.to_string();
            let _ = conf.update_config();
            addr
        } else {
            config.last_network.parse().unwrap()
        }
    };

    // get the list of available networks
    let networks = get_interfaces();

    // we need to pass some audio config data to the play function
    let audio_cfg = &audio_output_device
        .default_output_config()
        .expect("No default output config found");
    let wd = WavData {
        sample_format: audio_cfg.sample_format(),
        sample_rate: audio_cfg.sample_rate(),
        channels: audio_cfg.channels(),
    };

    // we now have enough information to create the GUI with meaningful data
    let mut mf = MainForm::create(
        &config,
        config_changed.clone(),
        &source_names,
        &networks,
        local_addr,
        &wd,
        APP_VERSION.to_string(),
    );

    // raise process priority a bit to prevent audio stuttering under cpu load
    raise_priority();

    // the rms monitor channel
    let rms_channel: (Sender<Vec<f32>>, Receiver<Vec<f32>>) = unbounded();

    // capture system audio
    debug!("Try capturing system audio");
    let stream: cpal::Stream;
    match capture_output_audio(&audio_output_device, rms_channel.0) {
        Some(s) => {
            stream = s;
            stream.play().unwrap();
        }
        None => {
            ui_log("*E*E*> Could not capture audio ...Please check configuration.".to_string());
        }
    }

    // now start the SSDP discovery update thread with a Crossbeam channel for renderer updates
    // the discovered renderers will be kept in this list
    ui_log("Discover networks".to_string());
    let mut renderers: Vec<Renderer> = Vec::new();
    let (ssdp_tx, ssdp_rx): (Sender<Renderer>, Receiver<Renderer>) = unbounded();
    ui_log("Starting SSDP discovery".to_string());
    let ssdp_int = CONFIG.read().ssdp_interval_mins;
    let _ = std::thread::Builder::new()
        .name("ssdp_updater".into())
        .stack_size(4 * 1024 * 1024)
        .spawn(move || run_ssdp_updater(ssdp_tx, ssdp_int))
        .unwrap();

    // also start the "monitor_rms" thread
    let rms_receiver = rms_channel.1;
    let mon_l = mf.rms_mon_l.clone();
    let mon_r = mf.rms_mon_r.clone();
    let _ = std::thread::Builder::new()
        .name("rms_monitor".into())
        .stack_size(4 * 1024 * 1024)
        .spawn(move || run_rms_monitor(&wd.clone(), rms_receiver, mon_l, mon_r))
        .unwrap();

    // finally start a webserver on the local address,
    // with a Crossbeam feedback channel for connection accept/drop
    let (feedback_tx, feedback_rx): (Sender<StreamerFeedBack>, Receiver<StreamerFeedBack>) =
        unbounded();
    let server_port = CONFIG.read().server_port;
    let _ = std::thread::Builder::new()
        .name("swyh_rs_webserver".into())
        .stack_size(4 * 1024 * 1024)
        .spawn(move || {
            run_server(
                &local_addr,
                server_port.unwrap_or_default(),
                wd,
                feedback_tx,
            )
        })
        .unwrap();

    // get the logreader channel
    let logreader = &LOGCHANNEL.read().1;

    // and now we can run the GUI event loop, app::awake() is used by the various threads to
    // trigger updates when something has changed, some threads use Crossbeam channels
    // to signal what has changed
    while app::wait() {
        if app::should_program_quit() {
            break;
        }
        // test for a configuration change that needs an app restart to take effect
        if config_changed.get() && app_restart(&mf) != 0 {
            config_changed.set(false);
        }
        // check if the streaming webserver has closed a connection not caused by
        // pushing a renderer button
        // in that case we turn the button off as a visual feedback for the user
        // but if auto_resume is set, we restart playing instead
        while let Ok(streamer_feedback) = feedback_rx.try_recv() {
            if let Some(button) = mf.buttons.get_mut(&streamer_feedback.remote_ip) {
                match streamer_feedback.streaming_state {
                    StreamingState::Started => {
                        if !button.is_set() {
                            button.set(true);
                        }
                    }
                    StreamingState::Ended => {
                        // first check if the renderer has actually not started streaming again
                        // as this can happen with Bubble/Nest Audio Openhome
                        let still_streaming = CLIENTS
                            .read()
                            .values()
                            .any(|chanstrm| chanstrm.remote_ip == streamer_feedback.remote_ip);
                        if !still_streaming {
                            if mf.auto_resume.is_set() && button.is_set() {
                                if let Some(r) = renderers
                                    .iter()
                                    .find(|r| r.remote_addr == streamer_feedback.remote_ip)
                                {
                                    let config = CONFIG.read().clone();
                                    let _ = r.play(
                                        &local_addr,
                                        server_port.unwrap_or_default(),
                                        &wd,
                                        &dummy_log,
                                        config.use_wave_format,
                                        config.bits_per_sample.unwrap(),
                                    );
                                }
                            } else if button.is_set() {
                                button.set(false);
                            }
                        }
                    }
                }
            }
        }
        // check the ssdp discovery thread channel for newly discovered renderers
        // add a new button below the last one for each discovered renderer
        while let Ok(newr) = ssdp_rx.try_recv() {
            mf.add_renderer_button(&newr);
            renderers.push(newr.clone());
        }
        // check the logchannel for new log messages to show in the logger textbox
        while let Ok(msg) = logreader.try_recv() {
            mf.add_log_msg(msg);
        }
    } // while app::wait()
}

fn app_restart(mf: &MainForm) -> i32 {
    let c = dialog::choice(
        mf.wind.width() as i32 / 2 - 100,
        mf.wind.height() as i32 / 2 - 50,
        "Configuration value changed!",
        "Restart",
        "Cancel",
        "",
    );
    if c == 0 {
        std::process::Command::new(std::env::current_exe().unwrap().into_os_string())
            .spawn()
            .expect("Unable to spawn myself!");
        std::process::exit(0);
    } else {
        c
    }
}

/// ui_log - send a logmessage to the textbox on the Crossbeam LOGCHANNEL
fn ui_log(s: String) {
    let cat: &str = &s[..2];
    match cat {
        "*W" => warn!("tb_log: {}", s),
        "*E" => error!("tb_log: {}", s),
        _ => info!("tb_log: {}", s),
    };
    let logger = &LOGCHANNEL.read().0;
    logger.send(s).unwrap();
    app::awake();
}

/// a dummy_log is used during AV transport autoresume
fn dummy_log(s: String) {
    debug!("Autoresume: {}", s);
}

/// run the ssdp_updater - thread that periodically run ssdp discovery
/// and detect new renderers
/// send any new renderers to te main thread on the Crossbeam ssdp channel
fn run_ssdp_updater(ssdp_tx: Sender<Renderer>, ssdp_interval_mins: f64) {
    // the hashmap used to detect new renderers
    let mut rmap: HashMap<String, Renderer> = HashMap::new();
    loop {
        let renderers = discover(&rmap, &ui_log).unwrap_or_default();
        for r in renderers.iter() {
            if !rmap.contains_key(&r.remote_addr) {
                let _ = ssdp_tx.send(r.clone());
                app::awake();
                std::thread::yield_now();
                info!(
                    "Found new renderer {} {}  at {}",
                    r.dev_name, r.dev_model, r.remote_addr
                );
                rmap.insert(r.remote_addr.clone(), r.clone());
            }
        }
        std::thread::sleep(Duration::from_millis(
            (ssdp_interval_mins * 60.0 * 1000.0) as u64,
        ));
    }
}

fn run_rms_monitor(
    wd: &WavData,
    rms_receiver: Receiver<Vec<f32>>,
    mut rms_frame_l: Progress,
    mut rms_frame_r: Progress,
) {
    // compute # of samples needed to get a 10 Hz refresh rate
    let samples_per_update = ((wd.sample_rate.0 * wd.channels as u32) / 10) as i64;
    let mut nsamples = 0i64;
    let mut sum_l = 0i64;
    let mut sum_r = 0i64;
    while let Ok(samples) = rms_receiver.recv() {
        for (n, sample) in samples.iter().enumerate() {
            nsamples += 1;
            let i64sample = (*sample).to_i16() as i64;
            if n & 1 == 0 {
                sum_l += i64sample * i64sample;
            } else {
                sum_r += i64sample * i64sample;
            }
            if nsamples >= samples_per_update {
                // compute rms value
                let rms_l = ((sum_l / nsamples) as f64).sqrt();
                rms_frame_l.set_value(rms_l);
                let rms_r = ((sum_r / nsamples) as f64).sqrt();
                rms_frame_r.set_value(rms_r);
                app::awake();
                //reset counters
                nsamples = 0;
                sum_l = 0;
                sum_r = 0;
            }
        }
    }
}
