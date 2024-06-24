use chrono::Duration as ChronoDuration;
use chrono::{DateTime, TimeZone, Utc};
use serde::Deserialize;
use serde_yaml::from_reader;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use wmi::{COMLibrary, FilterValue, WMIConnection};

#[derive(Deserialize, Debug)]
struct Config {
    watch_list: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_ProcessStartTrace")]
#[allow(non_snake_case)]
struct ProcessStartTrace {
    ProcessID: u32,
    ProcessName: String,
    TIME_CREATED: u64,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_ProcessStopTrace")]
#[allow(non_snake_case)]
struct ProcessStopTrace {
    ProcessID: u32,
    ProcessName: String,
    TIME_CREATED: u64,
}

#[derive(Debug)]
struct ProcessInfo {
    process_name: String,
    start_time: Option<DateTime<Utc>>,
    stop_time: Option<DateTime<Utc>>,
}

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let file = File::open("config.yaml")?;
    let config: Config = from_reader(file)?;

    let map_process_info: Arc<Mutex<HashMap<u32, ProcessInfo>>> =
        Arc::new(Mutex::new(HashMap::new()));
    let watch_list = Arc::new(config.watch_list);

    let start_handles: Vec<_> = watch_list
        .iter()
        .map(|process_name| {
            spawn_start_thread(
                Arc::clone(&map_process_info),
                Arc::new(process_name.clone()),
            )
        })
        .collect();

    let stop_handles: Vec<_> = watch_list
        .iter()
        .map(|process_name| {
            spawn_stop_thread(
                Arc::clone(&map_process_info),
                Arc::new(process_name.clone()),
            )
        })
        .collect();
    let print_handle = spawn_print_thread(Arc::clone(&map_process_info));

    for handle in start_handles {
        handle.join().unwrap()?;
    }
    for handle in stop_handles {
        handle.join().unwrap()?;
    }
    print_handle.join().unwrap();

    Ok(())
}

fn convert_wmi_time_to_utc(time_created: u64) -> DateTime<Utc> {
    let seconds = (time_created / 10_000_000) as i64;
    let nanoseconds = ((time_created % 10_000_000) * 100) as u32;
    let base_time = Utc.with_ymd_and_hms(1601, 1, 1, 0, 0, 0).unwrap();
    let duration =
        ChronoDuration::seconds(seconds) + ChronoDuration::nanoseconds(nanoseconds as i64);
    base_time + duration
}

fn spawn_start_thread(
    map_process_info: Arc<Mutex<HashMap<u32, ProcessInfo>>>,
    process_name: Arc<String>,
) -> thread::JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> {
    thread::spawn(move || {
        let mut start_filters = HashMap::new();
        start_filters.insert(
            "ProcessName".to_string(),
            FilterValue::String((*process_name).clone()),
        );

        let com_con = COMLibrary::new()?;
        let wmi_con = WMIConnection::new(com_con)?;
        let start_iterator = wmi_con.filtered_notification::<ProcessStartTrace>(
            &start_filters,
            Some(Duration::from_millis(500)),
        )?;

        for result in start_iterator {
            match result {
                Ok(start_trace) => {
                    println!(
                        "Process started: {} (PID: {})",
                        start_trace.ProcessName, start_trace.ProcessID
                    );
                    let mut map = map_process_info.lock().unwrap();
                    map.insert(
                        start_trace.ProcessID,
                        ProcessInfo {
                            process_name: start_trace.ProcessName,
                            start_time: Some(convert_wmi_time_to_utc(start_trace.TIME_CREATED)),
                            stop_time: None,
                        },
                    );
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }
        Ok(())
    })
}

fn spawn_stop_thread(
    map_process_info: Arc<Mutex<HashMap<u32, ProcessInfo>>>,
    process_name: Arc<String>,
) -> thread::JoinHandle<Result<(), Box<dyn Error + Send + Sync>>> {
    thread::spawn(move || {
        let mut stop_filters = HashMap::new();
        stop_filters.insert(
            "ProcessName".to_string(),
            FilterValue::String((*process_name).clone()),
        );

        let com_con = COMLibrary::new()?;
        let wmi_con = WMIConnection::new(com_con)?;
        let stop_iterator = wmi_con.filtered_notification::<ProcessStopTrace>(
            &stop_filters,
            Some(Duration::from_millis(500)),
        )?;

        for result in stop_iterator {
            match result {
                Ok(stop_trace) => {
                    println!(
                        "Process stopped: {} (PID: {})",
                        stop_trace.ProcessName, stop_trace.ProcessID
                    );
                    let mut map = map_process_info.lock().unwrap();
                    if let Some(map_process_info) = map.get_mut(&stop_trace.ProcessID) {
                        map_process_info.stop_time =
                            Some(convert_wmi_time_to_utc(stop_trace.TIME_CREATED));
                    }
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                    break;
                }
            }
        }
        Ok(())
    })
}

fn spawn_print_thread(
    map_process_info: Arc<Mutex<HashMap<u32, ProcessInfo>>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(5));
        let info = map_process_info.lock().unwrap();
        println!("Current process info:");
        for (pid, map_process_info) in info.iter() {
            println!(
                "PID: {}, Name: {}, Start Time: {:}, Stop Time: {:}",
                pid,
                map_process_info.process_name,
                map_process_info
                    .start_time
                    .map_or("N/A".to_string(), |dt| dt.to_string()),
                map_process_info
                    .stop_time
                    .map_or("N/A".to_string(), |dt| dt.to_string()),
            );
        }
    })
}
