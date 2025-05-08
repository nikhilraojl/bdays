mod error;

use chrono::Datelike;
use chrono::Local;
use chrono::TimeDelta;
use chrono::TimeZone;
use chrono::Weekday;
use error::{Error, Result};
use std::fmt::Write;
use std::{env::consts::OS, path::PathBuf};
use std::{fs::File, io::BufReader};

#[derive(Debug)]
enum Rrule {
    Yearly(YearlyRule),
    // TODO: add Weekly Weekly(WeeklyRule),
}

impl Rrule {
    fn from_outlook_event(rule: Option<String>) -> Option<Self> {
        let rule = rule?;
        let rule_split = rule.split(';').collect::<Vec<&str>>();
        let freq_value = rule_split.first()?.split('=').collect::<Vec<&str>>();
        match *freq_value.get(1)? {
            "YEARLY" => {
                let by_month_day = rule_split
                    .get(1)?
                    .split('=')
                    .collect::<Vec<&str>>()
                    .get(1)?
                    .parse()
                    .ok()?;
                let by_month = rule_split
                    .get(2)?
                    .split('=')
                    .collect::<Vec<&str>>()
                    .get(1)?
                    .parse()
                    .ok()?;
                Some(Self::Yearly(YearlyRule {
                    by_month_day,
                    by_month,
                }))
            }
            // TODO: add Weekly
            // "WEEKLY" => {
            //     let count = rule_split[1].split('=').collect::<Vec<&str>>()[1]
            //         .parse()
            //         .unwrap();
            //     let by_day = rule_split[2].split('=').collect::<Vec<&str>>()[1]
            //         .split(',')
            //         .collect::<Vec<_>>();
            //     Some(Self::Weekly(WeeklyRule {
            //         count,
            //         weekdays: by_day.iter().map(|s| s.to_owned().to_owned()).collect(),
            //     }))
            // }
            _ => None,
        }
    }
}

#[derive(Debug)]
struct YearlyRule {
    by_month_day: u32,
    by_month: u32,
}

// #[derive(Debug)]
// struct WeeklyRule {
//     count: u32,
//     weekdays: Vec<String>,
// }

#[derive(Debug, Default)]
struct Event {
    date_start: Option<String>,
    summary: Option<String>,
    rrule: Option<Rrule>,
}

fn main() {
    // let ins_now = std::time::Instant::now();
    if let Err(err) = run() {
        eprintln!("{err}");
    }
    // println!("{}", ins_now.elapsed().as_millis());
}

fn run() -> Result<()> {
    let mut args = std::env::args();
    args.next();

    if let Some(_arg) = args.next() {
        eprintln!("No arguments accepted!");
    } else {
        let home_dir = match OS {
            "windows" => Ok(PathBuf::from(std::env::var("userprofile")?)),
            "linux" => Ok(PathBuf::from(std::env::var("HOME")?)),
            "macos" => Ok(PathBuf::from(std::env::var("HOME")?)),
            _ => Err(Error::UnSupportedOs),
        }?;
        let cal_path = home_dir.join("./cal.ics");
        find_birthdays(&parse_calendar(cal_path)?)?;
    }
    Ok(())
}

fn parse_calendar(cal_path: PathBuf) -> Result<Vec<Event>> {
    let mut events: Vec<Event> = Vec::new();

    let buf = BufReader::new(File::open(cal_path)?);
    let mut reader = ical::IcalParser::new(buf);
    let line = reader.next().ok_or(Error::IterError)??;

    for ical_event in line.events {
        let mut event = Event::default();
        for prop in ical_event.properties {
            match prop.name.as_str() {
                "RRULE" => {
                    event.rrule = Rrule::from_outlook_event(prop.value);
                }
                "SUMMARY" => event.summary = prop.value,
                "DTSTART" => event.date_start = prop.value,
                _ => {}
            }
        }
        events.push(event);
    }
    Ok(events)
}

fn find_birthdays(events: &Vec<Event>) -> Result<()> {
    let now = Local::now();
    let curr_day = now.day();
    let curr_month = now.month();
    let until_tomorrow = now + TimeDelta::days(1);
    let until_date = now + TimeDelta::days(7);

    let mut output_today: Vec<&str> = Vec::new();
    let mut output_tomorrow: Vec<&str> = Vec::new();
    let mut output_7days: Vec<(i64, Weekday, String, &str)> = Vec::new();

    for event in events {
        if let Some(Rrule::Yearly(y_rrule)) = &event.rrule {
            let e_name = event
                .summary
                .as_ref()
                .ok_or(Error::IncorrectRrule)?;
            let e_date = Local
                .with_ymd_and_hms(now.year(), y_rrule.by_month, y_rrule.by_month_day, 0, 0, 0)
                .single()
                .ok_or(Error::IncorrectRrule)?;

            if y_rrule.by_month == curr_month && y_rrule.by_month_day == curr_day {
                output_today.push(e_name);
            }

            if now < e_date && e_date < until_tomorrow {
                output_tomorrow.push(e_name);
            }

            if until_tomorrow < e_date && e_date < until_date {
                let formatted_e_date = format!("{}", e_date.format("%d/%m"));
                let in_days = e_date - now;
                output_7days.push((
                    in_days.num_days(),
                    e_date.weekday(),
                    formatted_e_date,
                    e_name,
                ));
            }
        }
    }

    let mut output_buf = String::new();

    let mut body_buf_today = String::new();
    for event in &output_today {
        body_buf_today.push_str(&format!("> {event}"));
        body_buf_today.push('\n');
    }

    let mut body_buf_tomorrow = String::new();
    for event in &output_tomorrow {
        body_buf_tomorrow.push_str(&format!("> {event}"));
        body_buf_tomorrow.push('\n');
    }

    let mut body_buf_7days = String::new();
    output_7days.sort_by(|a, b| a.0.cmp(&b.0));
    for event in output_7days {
        if event.0 == 1 {
            body_buf_7days.push_str(&format!(
                "{:>2} day  | {} {} | {}",
                event.0, event.1, event.2, event.3
            ));
        } else {
            body_buf_7days.push_str(&format!(
                "{:>2} days | {} {} | {}",
                event.0, event.1, event.2, event.3
            ));
        }
        body_buf_7days.push('\n');
    }

    //////////////////////////////////////////////////////////////////
    // output for `today`
    //////////////////////////////////////////////////////////////////
    writeln!(output_buf, "-----")?;
    writeln!(output_buf, "TODAY")?;
    writeln!(output_buf, "-----")?;
    if body_buf_today.is_empty() {
        writeln!(output_buf, "No birthdays today\n")?;
    } else {
        writeln!(output_buf, "{body_buf_today}")?;
    }

    //////////////////////////////////////////////////////////////////
    // output for `tomorrow`
    //////////////////////////////////////////////////////////////////
    writeln!(output_buf, "--------")?;
    writeln!(output_buf, "TOMORROW")?;
    writeln!(output_buf, "--------")?;
    if body_buf_tomorrow.is_empty() {
        writeln!(output_buf, "No birthdays tomorrow\n")?;
    } else {
        writeln!(output_buf, "{body_buf_tomorrow}")?;
    }

    //////////////////////////////////////////////////////////////////
    // output for `in 7 days`
    //////////////////////////////////////////////////////////////////
    if body_buf_7days.is_empty() {
        writeln!(output_buf, "No birthdays in 7 days")?;
    } else {
        writeln!(output_buf, "--------")?;
        writeln!(output_buf, "UPCOMING")?;
        writeln!(output_buf, "--------")?;
        write!(output_buf, "{body_buf_7days}")?;
    }
    println!("{output_buf}");
    Ok(())
}
