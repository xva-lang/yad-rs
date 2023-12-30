use crate::config::Config;

#[allow(unused_imports)]
use log::{Level, Record};

const DEFAULT_SYSLOG_IDENTIFIER: &str = "yad";

#[cfg(target_os = "linux")]
mod systemd {
    use super::DEFAULT_SYSLOG_IDENTIFIER;
    use crate::config::Config;
    use log::{Level, Record};
    use systemd_journal_logger::JournalLog;

    fn log_journal(config: Option<&Config>, syslog_identifier: &mut String) -> bool {
        match config {
            Some(cfg) => match cfg.logging() {
                Some(l) => match &l.journalctl {
                    Some(jnlcfg) => {
                        if let Some(ident) = &jnlcfg.identifier {
                            *syslog_identifier = ident.to_string();
                        }

                        true
                    }
                    None => false,
                },
                None => false,
            },
            None => false,
        }
    }

    pub(super) fn journal_level(input: String, config: Option<&Config>, level: Level) {
        let mut syslog_identifier = DEFAULT_SYSLOG_IDENTIFIER.to_string();

        if log_journal(config, &mut syslog_identifier) {
            // let formatted_msg = format_args!("{msg}");

            // format_args! seems to have a weird lifetime bug?
            // See https://github.com/rust-lang/rust/issues/92698
            // Using an IIFE that takes ownership of the format_args! result and does what
            // it needs with it seems to be a valid workaround.

            (move |msg: std::fmt::Arguments| {
                let mut builder = Record::builder();
                let record = builder.level(level).args(msg).build();

                JournalLog::new()
                    .unwrap()
                    .with_extra_fields(vec![("VERSION", env!("CARGO_PKG_VERSION"))])
                    .with_syslog_identifier(syslog_identifier.to_string())
                    .journal_send(&record)
                    .unwrap();
            })(format_args!("{}", &input));
        }
    }
}

pub(crate) fn info(input: String, config: Option<&Config>) {
    //#[cfg(target_os = "linux")]
    //systemd::journal_level(input, config, Level::Info);

    if let Some(c) = config {
        if let Some(l) = c.logging() {
            if let Some(_) = &l.stdout {
                println!("{input}")
            }
        }
    }

    #[cfg(target_os = "linux")]
    systemd::journal_level(input, config, Level::Info);
}

pub(crate) fn error(input: String, config: Option<&Config>) {
    //#[cfg(target_os = "linux")]
    //systemd::journal_level(input, config, Level::Error);

    if let Some(c) = config {
        if let Some(l) = c.logging() {
            if let Some(_) = &l.stdout {
                println!("{input}")
            }
        }
    }

    #[cfg(target_os = "linux")]
    systemd::journal_level(input, config, Level::Error);
}
