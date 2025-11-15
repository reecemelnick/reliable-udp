use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs::File;
use fs2::FileExt;

#[derive(Debug, Deserialize, Serialize)]
struct PacketStats {
    Program: String,
    Packets_Sent: u32,
    Packets_Received: u32,
    Packets_Ignored: u32,
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
            record.Packets_Sent = 0;
            record.Packets_Received = 0;
            record.Packets_Ignored = 0;
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

    println!("CSV updated successfully!");
    Ok(())
}

pub fn increment_packet_count(
    program_name: &str,
    sent_inc: u32,
    received_inc: u32,
    ignored_inc: u32,
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
        if record.Program == program_name {
            record.Packets_Sent += sent_inc;
            record.Packets_Received += received_inc;
            record.Packets_Ignored += ignored_inc;
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

    println!("CSV updated successfully!");
    Ok(())
}
