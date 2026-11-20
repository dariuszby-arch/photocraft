//! Read-only diagnostic: compare PSD admission with real and controlled memory snapshots.

#[cfg(feature = "testgen")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use photocraft_io::{ImportOptions, memory};
    use photocraft_raster::Interrupt;

    let path = std::env::args_os().nth(1).ok_or("pass a PSD path")?;
    let path = std::path::Path::new(&path);
    let bytes = std::fs::read(path)?;
    let psd = photocraft_psd::PsdFile::from_bytes(&bytes)?;
    println!("header={:?}; layers={}", psd.header, psd.layers().len());
    let estimate = photocraft_io::estimate_psd_memory(&psd)?;
    println!("estimate={estimate:?}");
    let real = memory::MemorySnapshot::current();
    let constrained = memory::MemorySnapshot { available_ram: Some(0), swap_free: Some(0), commit_headroom: None };
    let mut real_opened = false;
    let mut zero_refused = false;
    for (label, snapshot) in [("real", real), ("zero-ram-with-scratch", constrained)] {
        memory::with_mock_memory(snapshot, || {
            memory::with_mock_scratch(Some(4 << 30), || {
                println!("{label}: snapshot={snapshot:?}; budget={:?}", memory::resolve_budget(snapshot, None));
                match photocraft_io::import_file("probe.psd", path, &ImportOptions::default(), &Interrupt::NONE) {
                    Ok(result) => {
                        if label == "real" {
                            real_opened = true;
                        }
                        println!("{label}: opened; size={:?}; layers={}; warnings={:?}", result.document.size, result.document.layers.len(), result.warnings);
                    }
                    Err(error) => {
                        if label == "zero-ram-with-scratch" {
                            zero_refused = matches!(error, photocraft_io::IoError::PsdImportBudget { required, limit } if required == estimate.total_bytes && limit == estimate.tile_bytes.min(4 << 30));
                        }
                        println!("{label}: ERROR: {error}");
                    }
                }
            });
        });
    }
    if !real_opened {
        return Err("PSD import failed with the real memory snapshot".into());
    }
    if !zero_refused {
        return Err("expected a genuine zero-RAM snapshot to retain the hard refusal".into());
    }
    Ok(())
}

#[cfg(not(feature = "testgen"))]
fn main() {
    eprintln!("Build this diagnostic with --features testgen.");
}
