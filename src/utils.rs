pub mod progress_bar {
    use std::time::Duration;

    use indicatif::{ProgressBar, ProgressState, ProgressStyle};

    pub fn style_progress_bar(pb: &ProgressBar) {
        pb.set_style(
            ProgressStyle::with_template(
                "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .with_key(
                "eta",
                |state: &ProgressState, w: &mut dyn std::fmt::Write| {
                    write!(w, "{:.1}s", state.eta().as_secs_f64()).unwrap()
                },
            )
            .progress_chars("#>-"),
        );

        pb.enable_steady_tick(Duration::from_millis(100));
    }
}

pub mod repositories {
    use time::{OffsetDateTime, format_description::well_known::Rfc3339};

    pub fn display_offset_datetime(offset_datetime: &OffsetDateTime) -> String {
        match offset_datetime.format(&Rfc3339) {
            Ok(rfc3339) => rfc3339,
            Err(error) => panic!("Could not format OffsetDateTime: {}", error),
        }
    }
}
