
// alarm.rs

use crate::models::errors::MyError;
use crate::config::BitcoinRpcConfig;
use crossterm::{event, terminal, ExecutableCommand};
use rodio::{Decoder, OutputStream, Sink, Source};
use std::{fs::File, io::{self, BufReader}, sync::{Arc, Mutex}, thread, time, env};
use crate::rpc::fetch_blockchain_info;
use colored::*;
use std::path::Path;
use num_format::{Locale, ToFormattedString};

pub async fn check_and_activate_alarm(init_blocks: u64, config: &BitcoinRpcConfig) -> Result<(), MyError> {
    // Parse command-line arguments.
    let args: Vec<String> = env::args().collect();
    let mut alarm_blocks_ahead = None;
    let mut mp3_filename: Option<String> = None;

    if args.len() > 1 && args[1].starts_with("-a") {
        if let Some(blocks_str) = args[1].strip_prefix("-a") {
            if let Ok(blocks) = blocks_str.parse::<u64>() {
                alarm_blocks_ahead = Some(blocks);
            }
        }

        // Ensure there is a third argument for the mp3 file path and check if exists.
        if args.len() > 2 {
            mp3_filename = Some(args[2].clone());
        }

        // Plan to add embedded melody using include_bytes! as default.
        if let Some(filepath) = mp3_filename.as_deref() {
            if !Path::new(filepath).exists() {
                eprintln!("Error: MP3 file does not exist at path: {}", filepath);
                std::process::exit(1); // Exit the program with a non-zero status code.
            }
        }
    }

    // Start polling mode if alarm is enabled.
    if let Some(alarm_blocks) = alarm_blocks_ahead {
        let initial_blocks = init_blocks; 
        let threshold = initial_blocks + alarm_blocks;
        let mp3_file = &mp3_filename.clone().unwrap_or("default string".to_string());

        println!("Alarm set to activate at block {} or greater.", 
            threshold.to_formatted_string(&Locale::en).green());
        println!("Set to play: {:?}", mp3_file);

        loop {
            thread::sleep(time::Duration::from_secs(60));

            // Fetch the latest blockchain info.
            let blockchain_info = fetch_blockchain_info(&config.bitcoin_rpc).await?;
            if blockchain_info.blocks >= threshold {
                // println!("Alarm triggered! {} blocks have passed.", alarm_blocks);

                // Play the alarm sound.  Use the mp3_filename if it is available.
                if let Some(filepath) = mp3_filename.as_deref() {
                    if let Err(e) = play_alarm_sound(filepath) {
                        eprintln!("Error playing alarm sound: {}", e);
                }
                } else {
                    eprintln!("Error: No MP3 file specified. Please provide a file path as the third argument.");
                }

                println!("Hello!, Block {} activated your alarm.", blockchain_info.blocks
                    .to_formatted_string(&Locale::en).green());

                break;
            }
        }
    }

    Ok(())

}

// Plays an alarm sound until stopped by any key.
fn play_alarm_sound(file_path: &str) -> Result<(), MyError> {
    let file_path = file_path.to_string();

    // Set up audio output.
    let (_stream, stream_handle) = OutputStream::try_default()
        .map_err(|_| MyError::Audio("Failed to initialize audio output stream".to_string()))?;
    let sink = Sink::try_new(&stream_handle)
        .map_err(|_| MyError::Audio("Failed to create audio sink".to_string()))?;
    let sink = Arc::new(Mutex::new(sink));
    let sink_clone = Arc::clone(&sink);

    // Spawn a thread to play the alarm sound.
    let alarm_thread = thread::spawn(move || {
        let file = File::open(&file_path).map_err(|_| {
            MyError::Audio(format!("Failed to open audio file: {}", file_path))
        })?;
        let source = Decoder::new(BufReader::new(file)).map_err(|_| {
            MyError::Audio(format!("Failed to decode audio file: {}", file_path))
        })?;

        let sink = sink_clone.lock().map_err(|_| {
            MyError::Audio("Failed to lock the audio sink".to_string())
        })?;

        sink.append(source.repeat_infinite());
        sink.set_volume(0.5);

        Ok::<(), MyError>(())
    });

    if let Err(e) = alarm_thread.join()
        .unwrap_or_else(|_| Err(MyError::Audio("Thread panicked while playing alarm".to_string()))) {
        return Err(e);
    }

    // Enable raw mode for key detection.
    io::stdout()
        .execute(terminal::EnterAlternateScreen)
        .map_err(|_| MyError::Terminal("Failed to enter alternate screen mode".to_string()))?;
    terminal::enable_raw_mode()
        .map_err(|_| MyError::Terminal("Failed to enable raw mode".to_string()))?;

    println!("Press any key to stop the alarm.");

    let start_time = time::Instant::now(); // Track the start time
    loop {
        // Check if 5 minutes have passed, and stop the alarm to avoid indefinite playback 
        // and potential system resource exhaustion.
        if start_time.elapsed() >= time::Duration::from_secs(300) {
            println!("Timeout reached: 5 minutes elapsed. Stopping the alarm.");
            let sink = sink.lock().map_err(|_| {
                MyError::Audio("Failed to lock the audio sink for stopping".to_string())
            })?;
            sink.stop();
            break;
        }

        // Check for key press.
        if event::poll(std::time::Duration::from_millis(500))
            .map_err(|_| MyError::Terminal("Failed to poll for events".to_string()))?
        {
            if let event::Event::Key(_) = event::read()
                .map_err(|_| MyError::Terminal("Failed to read key event".to_string()))?
            {
                let sink = sink.lock().map_err(|_| {
                    MyError::Audio("Failed to lock the audio sink for stopping".to_string())
                })?;
                sink.stop();
                break;
            }
        }
    }

    // Disable raw mode and return back to original terminal state.
    terminal::disable_raw_mode()
        .map_err(|_| MyError::Terminal("Failed to disable raw mode".to_string()))?;
    io::stdout()
        .execute(terminal::LeaveAlternateScreen)
        .map_err(|_| MyError::Terminal("Failed to leave alternate screen mode".to_string()))?;

    Ok(())
}
