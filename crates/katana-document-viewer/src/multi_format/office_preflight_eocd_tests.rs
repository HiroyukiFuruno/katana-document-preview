use super::EndOfCentralDirectory;

fn record(disk: u16, central_disk: u16, disk_entries: u16, total_entries: u16) -> Vec<u8> {
    let mut bytes = vec![0; 22];
    bytes[..4].copy_from_slice(b"PK\x05\x06");
    bytes[4..6].copy_from_slice(&disk.to_le_bytes());
    bytes[6..8].copy_from_slice(&central_disk.to_le_bytes());
    bytes[8..10].copy_from_slice(&disk_entries.to_le_bytes());
    bytes[10..12].copy_from_slice(&total_entries.to_le_bytes());
    bytes
}

#[test]
fn rejects_missing_truncated_and_multi_disk_records() {
    assert!(EndOfCentralDirectory::entry_count(b"missing").is_err());
    assert!(EndOfCentralDirectory::entry_count(b"PK\x05\x06").is_err());
    assert!(EndOfCentralDirectory::entry_count(&record(1, 0, 1, 1)).is_err());
}

#[test]
fn accepts_zip64_sentinel_and_regular_counts() {
    assert_eq!(
        Ok(None),
        EndOfCentralDirectory::entry_count(&record(0, 0, u16::MAX, u16::MAX))
    );
    assert_eq!(
        Ok(Some(2)),
        EndOfCentralDirectory::entry_count(&record(0, 0, 2, 2))
    );
}
