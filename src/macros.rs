#[macro_export]
#[allow(missing_docs)]
macro_rules! valid_count {
    ($entry:expr, $conds:expr) => {{
        let count = $entry.count();
        if $conds.contains(&count) {
            Ok(())
        } else {
            Err(DecodeError::from(DecodeValueError::InvalidCount(count)))
        }
    }};
}

#[macro_export]
#[allow(missing_docs)]
macro_rules! field_is_data_pointer {
    ($reader:expr, $endian:expr, $entry:expr) => {{
        if $entry.overflow() {
            let next = $entry.field().read_u32($endian)?;
            let next = std::io::SeekFrom::Start(next as u64);

            $reader.seek(next)?;

            true
        } else {
            false
        }
    }};
}
