#[macro_export]
#[allow(missing_docs)]
macro_rules! valid_count {
    ($entry:expr, $conds:expr) => {{
        let count = $entry.count();
        if $conds.contains(&count) {
            Ok(())
        } else {
            Err(DecodeError::from(DecodeValueErrorDetail::InvalidCount(
                count,
            )))
        }
    }};
}

#[macro_export]
#[allow(missing_docs)]
macro_rules! field_is_data_pointer {
    ($decoder:expr, $entry:expr) => {{
        let endian = $decoder.endian();

        if $entry.overflow() {
            let next = $entry.field().read_u32(endian)?;
            let next = std::io::SeekFrom::Start(next as u64);
            $decoder.seek(next)?;
            true
        } else {
            false
        }
    }};
}
