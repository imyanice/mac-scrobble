use crate::{api_key, api_secret, get_config, is_auth_error};
use rustfm_scrobble::{Scrobble, Scrobbler};
use serde_json::Value;
use std::time::Duration;
use std::{process, thread};
use tauri::AppHandle;

pub fn listen(app_handle: AppHandle) {
    loop {
        if get_config().enabled {
            loop {
                let username = get_config().username;
                let password = get_config().password;
                let enabled = get_config().enabled;
                let mut old_data: Value = Value::String("nananananaannan".parse().unwrap());
                let mut listened_time = 0;


                let mut scrobbler = Scrobbler::new(api_key(), api_secret());

                let auth = scrobbler.authenticate_with_password(username.as_str(), password.as_str());
                match auth {
                    Ok(res) => {
                        println!("{:?}", res)
                    }
                    Err(err) => {
                        if is_auth_error(err, &app_handle) { break; };
                    }
                }
                let script = "
                    const music = Application('Music');
                    if (music.running()) {
                        if (music.playerState() == 'stopped') {
                            `{\"running\": false}`;
                        } else {
                            if (music.playerState() !== 'paused') {
                                if (music.currentTrack().properties().class !== 'fileTrack') {
                                    let name = music.currentTrack().name();
                                    let album = music.currentTrack().album();
                                    let duration = music.currentTrack().duration();
                                    let artist = music.currentTrack().artist();
                                    `{\"running\": true, \"data\": {\"properties\": ${JSON.stringify(music.currentTrack().properties())}, \"playerPos\": ${music.playerPosition()}}}`;
                                } else {
                                    `{\"running\": false}`;
                                }
                            } else {
                                `{\"running\": false}`;
                            }
                        }
                    } else {
                        `{\"running\": false}`;
                    }";
                loop {
                    thread::sleep(Duration::from_millis(1000));
                    if !get_config().enabled { break; }
                    if enabled {
                        let output = process::Command::new("osascript")
                            .arg("-l")
                            .arg("JavaScript")
                            .arg("-e")
                            .arg(script)
                            .output()
                            .unwrap()
                            .stdout;
                        let str = String::from_utf8(output).unwrap();
                        println!("{str}");
                        let a: Result<Value, serde_json::Error> = serde_json::from_str(str.as_str());
                        match a {
                            Ok(data) => {
                                if data["running"] == true {
                                    println!("Listened for: {}s", listened_time);

                                    let track = Scrobble::new(
                                        data["data"]["properties"]["artist"].as_str().unwrap(),
                                        data["data"]["properties"]["name"].as_str().unwrap(),
                                        data["data"]["properties"]["album"].as_str().unwrap(),
                                    );
                                    if old_data["data"]["properties"] == data["data"]["properties"] {
                                        let mut song_duration: i64;
                                        if data["data"]["properties"]["duration"].as_i64().is_none() {
                                            song_duration = data["data"]["properties"]["duration"].as_f64().unwrap() as i64;
                                        } else {
                                            song_duration = data["data"]["properties"]["duration"].as_i64().unwrap()
                                        }
                                        if song_duration > 30 {
                                            listened_time += 1;
                                            if listened_time
                                                == (song_duration
                                                / 2)
                                                || (listened_time == 4 * 60)
                                            {
                                                let response = scrobbler.scrobble(&track);
                                                match response {
                                                    Ok(res) => {
                                                        println!("Sent scrobble! {:#?}", res);
                                                    }
                                                    Err(err) => {
                                                        if is_auth_error(err, &app_handle) { break; };
                                                    }
                                                }
                                            }
                                        }
                                    } else {
                                        old_data = data.clone();
                                        listened_time = 1;
                                        let response = scrobbler.now_playing(&track);
                                        match response {
                                            Ok(res) => {
                                                println!("Sent now playing! {:#?}", res);
                                            }
                                            Err(err) => {
                                                if is_auth_error(err, &app_handle) { break; };
                                            }
                                        }
                                    }
                                }
                            }
                            Err(err) => {
                                println!("{}", err)
                            }
                        }
                    }
                }
                break;
            }
        }
        thread::sleep(Duration::from_millis(2000));
    }
}
