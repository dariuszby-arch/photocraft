//! Read-only memory diagnostics for a machine where native import admission fails.

fn main() {
    println!("PhotoCraft DW PSD FIX3 — memory diagnostics");
    let measurement = photocraft_raster::memory::system::current();
    println!("detector={measurement:?}");
    if measurement.constraint_unreadable && measurement.available_ram.is_none() {
        println!("Limit niezamontowanego kontrolera pamięci jest nieznany — to nie oznacza 0 RAM. Import używa standardowej polityki zastępczej.");
    }
    #[cfg(target_os = "linux")]
    {
        for path in ["/proc/meminfo", "/proc/self/cgroup", "/proc/self/mountinfo"] {
            println!("\n{path}:");
            match std::fs::read_to_string(path) {
                Ok(text) => {
                    if text.is_empty() {
                        println!("(empty)");
                    }
                    for line in text.lines().filter(|line| match path {
                        "/proc/meminfo" => ["MemTotal:", "MemAvailable:", "SwapTotal:", "SwapFree:"].iter().any(|key| line.starts_with(key)),
                        "/proc/self/mountinfo" => line.split_once(" - ").is_some_and(|(_, tail)| tail.starts_with("cgroup ") || tail.starts_with("cgroup2 ")),
                        _ => true,
                    }) {
                        println!("{line}");
                    }
                }
                Err(error) => println!("{error}"),
            }
        }
    }
}
