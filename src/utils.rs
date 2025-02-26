use time::{OffsetDateTime, format_description::well_known::Rfc3339};

pub fn display_offset_datetime(offset_datetime: &OffsetDateTime) -> String {
    match offset_datetime.format(&Rfc3339) {
        Ok(rfc3339) => rfc3339,
        Err(error) => panic!("Could not format OffsetDateTime: {}", error),
    }
}
