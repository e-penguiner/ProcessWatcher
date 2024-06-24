# Process-Watcher

This Rust application monitors the start and stop events of specific processes on a Windows system. It uses Windows Management Instrumentation (WMI) to receive notifications for process events and logs relevant information.

## Features

- Monitors process start and stop events.
- Logs process ID, name, start time, and stop time.
- Continuously prints current process information every 5 seconds.
- Supports monitoring multiple processes specified in a configuration file.

## Requirements

- Rust (latest stable version)
- Windows operating system
- WMI (Windows Management Instrumentation) enabled

## Usage

1. **Clone the repository:**

   ```bash
   git clone https://github.com/yourusername/process-watcher.git
   cd process-watcher
   ```

2. **Build the project:**

   ```bash
   cargo build --release
   ```

3. **Prepare the configuration file:**

   Create a `config.yaml` file in the root directory of the project with the following structure:

   ```yaml
   watch_list:
     - "paraview.exe"
     - "Code.exe"
   ```

   Modify the `watch_list` to include the processes you want to monitor.

4. **Run the project:**

   ```bash
   cargo run --release
   ```

## Example Output

```shell
Process started: paraview.exe (PID: 1234)
Process stopped: paraview.exe (PID: 1234)
Process started: Code.exe (PID: 5678)
Current process info:
PID: 1234, Name: paraview.exe, Start Time: 2024-06-21T12:34:56Z, Stop Time: 2024-06-21T12:35:10Z
PID: 5678, Name: Code.exe, Start Time: 2024-06-21T12:40:00Z, Stop Time: N/A
```

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.

## Contributing

Contributions are welcome! Please open an issue or submit a pull request for any improvements or bug fixes.
