//--------------------------------------------------------------------
// Description:
// Date: xxxx-xx-xx
//--------------------------------------------------------------------

use anyhow::{anyhow, Result};
use log::{debug, info};
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::thread::JoinHandle;
use std::time::Duration;

const SECONDS_IN_MINUTE: u64 = 60;
const SECONDS_IN_HOUR: u64 = SECONDS_IN_MINUTE * 60;
const SECONDS_IN_DAY: u64 = SECONDS_IN_HOUR * 24;

// A centi-second is 1/100 second.
const CS_IN_MILLI_SECOND: u64 = 10;

// a deci-second is 1/10 second.
const DS_IN_MILLI_SECOND: u64 = 100;

const ERR_NEED_DURATION: &str = "must specify time duration";
const ERR_BAD_DURATION_SUFFIX: &str = "invalid time duration suffix";

// TODO: Add in support for "-1" as Duration::MAX once MAX
// is no longer experimental.
const DURATION_MAX: u32 = 1_000_000_000 - 1;

#[derive(Debug, Default)]
struct Entry {
    url: String,
    browser: Option<String>,
}

fn parse_time(value: &str) -> Result<Duration> {
    if value.is_empty() {
        Err(anyhow!(ERR_NEED_DURATION))
    } else if value == "-1" {
        let max = Duration::new(u64::MAX, DURATION_MAX);

        Ok(max)
    } else if let Some(i) = value.find(char::is_alphabetic) {
        let prefix = &value[..i];

        let numeric = prefix.parse::<u64>()?;

        let suffix = &value[i..];

        let duration = match suffix {
            "d" => Duration::from_secs(numeric * SECONDS_IN_DAY),
            "h" => Duration::from_secs(numeric * SECONDS_IN_HOUR),
            "m" => Duration::from_secs(numeric * SECONDS_IN_MINUTE),
            "s" => Duration::from_secs(numeric),
            "cs" => Duration::from_millis(numeric * CS_IN_MILLI_SECOND),
            "ds" => Duration::from_millis(numeric * DS_IN_MILLI_SECOND),
            "ms" => Duration::from_millis(numeric),
            "ns" => Duration::from_nanos(numeric),
            "us" => Duration::from_micros(numeric),

            _ => return Err(anyhow!(ERR_BAD_DURATION_SUFFIX)),
        };

        Ok(duration)
    } else {
        let secs = value.parse::<u64>()?;

        // "bare" numeric value (no suffix)
        Ok(Duration::from_secs(secs))
    }
}

fn read_url_file(filename: &str) -> Result<Vec<Entry>> {
    let reader: Box<dyn BufRead> = match filename {
        "-" => Box::new(BufReader::new(io::stdin())),
        _ => {
            let f = File::open(filename).map_err(|e| anyhow!(e))?;

            let reader = BufReader::new(f);

            Box::new(reader)
        }
    };

    let mut entries = Vec::<Entry>::new();

    for (i, line) in reader.lines().flatten().enumerate() {
        // Ignore empty lines
        if line.is_empty() {
            continue;
        }

        // Ignore comments
        if line.starts_with('#') {
            continue;
        }

        let fields: Vec<&str> = line.split(';').collect();

        let url = fields
            .first()
            .ok_or(format!("missing URL: file {}, line {}", filename, i + 1))
            .map_err(|e| anyhow!(e))?
            .trim()
            .to_string();

        // Check for an optional browser
        let browser = fields.get(1);

        let browser: Option<String> = browser.map(|browser| browser.to_string());

        let entry = Entry { url, browser };

        entries.push(entry);
    }

    Ok(entries)
}

fn open_urls(
    entries: Vec<Entry>,
    sleep_duration: Option<Duration>,
    cli_browser: Option<String>,
    dry_run: bool,
) -> Result<()> {
    let mut threads: Vec<JoinHandle<Result<(), std::io::Error>>> = vec![];

    let count = entries.len();

    for (i, entry) in entries.iter().enumerate() {
        let url = &entry.url;

        let num = i + 1;

        let thread: JoinHandle<Result<(), std::io::Error>>;

        let browser: &Option<String> = if cli_browser.is_some() {
            &cli_browser
        } else if entry.browser.is_some() {
            &entry.browser
        } else {
            &None
        };

        if dry_run {
            info!(
                dry_run = dry_run,
                thread_num = num,
                thread_count = count,
                url = url,
                browser = browser;
                "Would spawn URL");
        } else {
            if let Some(browser) = browser {
                debug!(
                    thread_num = num,
                    thread_count = count,
                    url = url,
                    browser = browser;
                    "Spawning URL with custom browser");

                thread = open::with_in_background(url, browser);
            } else {
                debug!(
                    thread_num = num,
                    thread_count = count,
                    url = url,
                    browser = browser;
                    "Spawning URL with default browser"
                );

                thread = open::that_in_background(url);
            }

            info!(
                thread_num = num,
                thread_count = count,
                url = url,
                browser = browser;
                "URL spawned");

            threads.push(thread);
        }

        let sleep_duration_str = format!("{:?}", sleep_duration);

        if dry_run {
            info!(
                dry_run = dry_run,
                thread_num = num,
                thread_count = count,
                sleep_duration = &sleep_duration_str;
                "Would sleep");
        } else if let Some(sleep_time) = sleep_duration {
            debug!(
                dry_run = dry_run,
                thread_num = num,
                thread_count = count,
                sleep_duration = &sleep_duration_str;
                "Sleeping");

            std::thread::sleep(sleep_time);

            debug!(
                dry_run = dry_run,
                thread_num = num,
                thread_count = count,
                sleep_duration = &sleep_duration_str;
                "Slept");
        }
    }

    if dry_run {
        info!(
            dry_run = dry_run,
            thread_count = count;
            "Would join threads"
        );
    } else {
        debug!(dry_run = dry_run, thread_count = count; "Joining threads");

        let mut i = 0;

        for thread in threads {
            let num = i + 1;

            debug!(
                dry_run = dry_run,
                thread_num = num,
                thread_count = count;
                "Joining thread"
            );

            thread.join().map_err(|e| anyhow!("{:?}", e))??;

            debug!(
                dry_run = dry_run,
                thread_num = num,
                thread_count = count;
                "Joined thread"
            );

            i += 1;
        }

        info!(thread_count = count; "Joined all threads");
    }

    Ok(())
}

pub fn handle_urls_file(
    file: &str,
    sleep_time: Option<String>,
    browser: Option<String>,
    dry_run: bool,
) -> Result<()> {
    info!(
        dry_run = dry_run,
        browser = browser,
        file = file,
        sleep_time = &sleep_time;
        "Options");

    let sleep_duration = if let Some(sleep) = sleep_time {
        Some(parse_time(&sleep)?)
    } else {
        None
    };

    let entries = read_url_file(file)?;

    info!(entries = entries.len(); "Read entries");

    open_urls(entries, sleep_duration, browser, dry_run)
}
