use chrono::Utc;
use std::fmt;
use tracing::{Event, Subscriber};
use tracing_subscriber::{
    fmt::{FmtContext, FormatEvent, FormatFields, format::Writer},
    registry::LookupSpan,
};

/// Custom formatter: `[ CLIENT ] TIMESTAMP LEVEL message`
pub struct ClientFormatter;

impl<S, N> FormatEvent<S, N> for ClientFormatter
where
    S: Subscriber + for<'a> LookupSpan<'a>,
    N: for<'a> FormatFields<'a> + 'static,
{
    fn format_event(
        &self,
        ctx: &FmtContext<'_, S, N>,
        mut writer: Writer<'_>,
        event: &Event<'_>,
    ) -> fmt::Result {
        let now = Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
        let level = *event.metadata().level();
        write!(writer, "[ CLIENT ] {} {} ", now, level)?;
        ctx.field_format().format_fields(writer.by_ref(), event)?;
        writeln!(writer)
    }
}

/// Initialise logging. When `debug_mode` is true uses the custom
/// `[ CLIENT ]` format; otherwise uses the compact default.
pub fn init(debug_mode: bool) {
    if debug_mode {
        tracing_subscriber::fmt()
            .event_format(ClientFormatter)
            .init();
    } else {
        tracing_subscriber::fmt()
            .compact()
            .with_target(false)
            .init();
    }
}
