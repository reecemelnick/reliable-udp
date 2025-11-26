use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use fs2::FileExt;

#[derive(Debug, Deserialize, Serialize)]
struct PacketStats {
    program: String,
    packets_sent: u32,
    packets_received: u32,
    packets_ignored: u32,
    packets_dropped: u32
}

pub fn reset_csv() -> Result<(), Box<dyn Error>> {
    let lock = File::create("../plotting/data.csv.lock")?;
    lock.lock_exclusive()?;

    // Read the CSV file
    let file = File::open("../plotting/data.csv")?;
    let mut reader = csv::Reader::from_reader(file);

    // Collect all rows into memory
    let mut records: Vec<PacketStats> = reader.deserialize().collect::<Result<_, _>>()?;

    // Increment packets for "Client"
    for record in &mut records {
            record.packets_sent = 0;
            record.packets_received = 0;
            record.packets_ignored = 0;
            record.packets_dropped = 0;
    }

    let tmp_path = "../plotting/data.csv.tmp";
    {
        let tmp = File::create(tmp_path)?;
        let mut writer = csv::Writer::from_writer(&tmp);

        for r in &records {
            writer.serialize(r)?;
        }

        writer.flush()?;
        tmp.sync_all()?;
    }

    std::fs::rename(tmp_path, "../plotting/data.csv")?;

    Ok(())
}

pub fn increment_packet_count(
    program_name: &str,
    sent_inc: u32,
    received_inc: u32,
    ignored_inc: u32,
    dropped_inc: u32,
) -> Result<(), Box<dyn Error>> {
    let lock = File::create("../plotting/data.csv.lock")?;
    lock.lock_exclusive()?;

    // Read the CSV file
    let file = File::open("../plotting/data.csv")?;
    let mut reader = csv::Reader::from_reader(file);

    // Collect all rows into memory
    let mut records: Vec<PacketStats> = reader.deserialize().collect::<Result<_, _>>()?;

    // Increment packets for "Client"
    for record in &mut records {
        if record.program == program_name {
            record.packets_sent += sent_inc;
            record.packets_received += received_inc;
            record.packets_ignored += ignored_inc;
            record.packets_dropped += dropped_inc;
        }
    }

    let tmp_path = "../plotting/data.csv.tmp";
    {
        let tmp = File::create(tmp_path)?;
        let mut writer = csv::Writer::from_writer(&tmp);

        for r in &records {
            writer.serialize(r)?;
        }

        writer.flush()?;
        tmp.sync_all()?;
    }

    std::fs::rename(tmp_path, "../plotting/data.csv")?;
    Ok(())
}
