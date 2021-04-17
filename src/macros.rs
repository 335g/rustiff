#[macro_export]
#[allow(missing_docs)]
macro_rules! valid_count {
    ($entry:expr, $conds:expr, $type_name:expr) => {{
        let count = $entry.count();
        if $conds.contains(&count) {
            Ok(())

        } else {
            let err = DecodingError::InvalidCount(vec![(count, $type_name)]);
            
            Err(DecodeError::from(err))
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
