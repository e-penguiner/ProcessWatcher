# Process Monitoring Tool

This Rust application monitors the start and stop events of a specific process on a Windows system. It uses Windows Management Instrumentation (WMI) to receive notifications for process events and logs relevant information.

## Features

- Monitors process start and stop events.
- Logs process ID, name, start time, and stop time.
- Continuously prints current process information every 5 seconds.

## Requirements

- Rust (latest stable version)
- Windows operating system
- WMI (Windows Management Instrumentation) enabled

## Usage

1. **Clone the repository:**

   ```bash
   git clone https://github.com/yourusername/process-monitoring-tool.git
   cd process-monitoring-tool
   ```

2. **Build the project:**

   ```bash
   cargo build --release
   ```

3. **Run the project:**

   ```bash
   cargo run --release
   ```

4. **Customize the process name:**

   By default, the tool monitors `Notepad.exe`. To monitor a different process, modify the `process_name` variable in the `main` function.

   ```rust
   let process_name = Arc::new("YourProcessName.exe".to_string());
   ```

## Example Output

```shell
Process started: Notepad.exe (PID: 1234)
Process stopped: Notepad.exe (PID: 1234)
Current process info:
PID: 1234, Name: Notepad.exe, Start Time: 2024-06-21T12:34:56Z, Stop Time: 2024-06-21T12:35:10Z
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any improvements or bug fixes.
