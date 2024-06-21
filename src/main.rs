use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::collections::HashMap;
use std::error::Error;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use wmi::{COMLibrary, FilterValue, WMIConnection};

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_ProcessStartTrace")]
#[serde(rename_all = "PascalCase")]
struct ProcessStartTrace {
    process_id: u32,
    process_name: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename = "Win32_ProcessStopTrace")]
#[serde(rename_all = "PascalCase")]
struct ProcessStopTrace {
    process_id: u32,
    process_name: String,
}

#[derive(Debug)]
struct ProcessInfo {
    process_name: String,
    start_time: Option<DateTime<Utc>>,
    stop_time: Option<DateTime<Utc>>,
}

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let process_info: Arc<Mutex<HashMap<u32, ProcessInfo>>> = Arc::new(Mutex::new(HashMap::new()));
    let process_name = Arc::new("Notepad.exe".to_string());

    let start_handle = spawn_start_thread(Arc::clone(&process_info), Arc::clone(&process_name));
    let stop_handle = spawn_stop_thread(Arc::clone(&process_info), Arc::clone(&process_name));
    let print_handle = spawn_print_thread(Arc::clone(&process_info));

    start_handle.join().unwrap()?;
    stop_handle.join().unwrap()?;
    print_handle.join().unwrap();

    Ok(())
}

fn spawn_start_thread(
    process_info: Arc<Mutex<HashMap<u32, ProcessInfo>>>,
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
            Some(Duration::from_secs(1)),
        )?;

        for result in start_iterator {
            match result {
                Ok(start_trace) => {
                    println!(
                        "Process started: {} (PID: {})",
                        start_trace.process_name, start_trace.process_id
                    );
                    let mut info = process_info.lock().unwrap();
                    info.insert(
                        start_trace.process_id,
                        ProcessInfo {
                            process_name: start_trace.process_name,
                            start_time: Some(Utc::now()),
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
    process_info: Arc<Mutex<HashMap<u32, ProcessInfo>>>,
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
            Some(Duration::from_secs(1)),
        )?;

        for result in stop_iterator {
            match result {
                Ok(stop_trace) => {
                    println!(
                        "Process stopped: {} (PID: {})",
                        stop_trace.process_name, stop_trace.process_id
                    );
                    let mut info = process_info.lock().unwrap();
                    if let Some(process_info) = info.get_mut(&stop_trace.process_id) {
                        process_info.stop_time = Some(Utc::now());
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
    process_info: Arc<Mutex<HashMap<u32, ProcessInfo>>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || loop {
        thread::sleep(Duration::from_secs(5));
        let info = process_info.lock().unwrap();
        println!("Current process info:");
        for (pid, process_info) in info.iter() {
            println!(
                "PID: {}, Name: {}, Start Time: {:}, Stop Time: {:}",
                pid,
                process_info.process_name,
                process_info
                    .start_time
                    .map_or("N/A".to_string(), |dt| dt.to_string()),
                process_info
                    .start_time
                    .map_or("N/A".to_string(), |dt| dt.to_string()),
            );
        }
    })
}
