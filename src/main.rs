mod home_dir;

use chrono::Datelike;
use chrono::Local;
use chrono::TimeDelta;
use chrono::TimeZone;
use home_dir::get;
use std::fmt::Write;
use std::path::PathBuf;
use std::{fs::File, io::BufReader};

#[derive(Debug)]
enum Rrule {
    Yearly(YearlyRule),
    // TODO: add Weekly
    // Weekly(WeeklyRule),
}

impl Rrule {
    fn from_outlook_event(rule: &str) -> Option<Self> {
        let rule_split = rule.split(';').collect::<Vec<&str>>();
        let freq_value = rule_split[0].split('=').collect::<Vec<&str>>();
        match *freq_value.get(1).unwrap() {
            "YEARLY" => {
                let by_month_day = rule_split[1].split('=').collect::<Vec<&str>>()[1]
                    .parse()
                    .unwrap();
                let by_month = rule_split[2].split('=').collect::<Vec<&str>>()[1]
                    .parse()
                    .unwrap();
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
    let mut args = std::env::args();
    // skip current program path
    args.next();
    let ins_now = std::time::Instant::now();

    if let Some(_arg) = args.next() {
        eprintln!("No arguments accepted!");
    } else {
        let home_dir = get().unwrap();
        let cal_path = home_dir.join("./cal.ics");
        find_birthdays(&parse_calendar(cal_path));
    }
    println!("{}", ins_now.elapsed().as_millis());
}

fn parse_calendar(cal_path: PathBuf) -> Vec<Event> {
    let buf = BufReader::new(File::open(cal_path).unwrap());

    let mut reader = ical::IcalParser::new(buf);

    let line = reader.next().unwrap().unwrap();
    let mut events: Vec<Event> = Vec::new();
    for event in line.events {
        let mut e = Event::default();
        for prop in event.properties {
            // println!("{:?}", prop);
            match prop.name.as_str() {
                "RRULE" => {
                    e.rrule = Rrule::from_outlook_event(&prop.value.unwrap());
                }
                "SUMMARY" => e.summary = prop.value,
                "DTSTART" => e.date_start = prop.value,
                _ => {}
            }
        }
        events.push(e);
    }
    return events;
}

fn find_birthdays(events: &Vec<Event>) {
    let now = Local::now();
    let curr_day = now.day();
    let curr_month = now.month();

    let until_date = now + TimeDelta::days(7);
    let mut output_today: Vec<&str> = Vec::new();
    let mut output_7days: Vec<(i64, &str)> = Vec::new();

    for event in events {
        if let Some(Rrule::Yearly(y_rrule)) = &event.rrule {
            if y_rrule.by_month == curr_month && y_rrule.by_month_day == curr_day {
                let e_name = event.summary.as_ref().unwrap();
                output_today.push(e_name);
            }
            let e_date = Local
                .with_ymd_and_hms(now.year(), y_rrule.by_month, y_rrule.by_month_day, 0, 0, 0)
                .unwrap();
            if now < e_date && e_date < until_date {
                let e_name = event.summary.as_ref().unwrap();
                let in_days = e_date - now;
                output_7days.push((in_days.num_days(), e_name));
            }
        }
    }

    let mut max_event_length = 5;
    let mut output_buf = String::new();

    let mut body_buf_today = String::new();
    for event in &output_today {
        if event.len() > max_event_length {
            max_event_length = event.len();
        }
        body_buf_today.push_str(&format!("> {event}"));
        body_buf_today.push('\n');
    }

    output_7days.sort_by(|a, b| a.0.cmp(&b.0));
    let mut body_buf_7days = String::new();
    for event in output_7days {
        if event.1.len() > max_event_length {
            max_event_length = event.1.len();
        }
        body_buf_7days.push_str(&format!("{:>2} days | {} ", event.0, event.1));
        body_buf_7days.push('\n');
    }

    //////////////////////////////////////////////////////////////////
    // output for `today`
    //////////////////////////////////////////////////////////////////
    writeln!(output_buf, "-----").unwrap();
    writeln!(output_buf, "TODAY").unwrap();
    writeln!(output_buf, "-----").unwrap();
    if body_buf_today.is_empty() {
        writeln!(output_buf, "No birthdays today\n").unwrap();
    } else {
        writeln!(output_buf, "{body_buf_today}").unwrap();
    }

    //////////////////////////////////////////////////////////////////
    // output for `in 7 days`
    //////////////////////////////////////////////////////////////////
    if body_buf_7days.is_empty() {
        writeln!(output_buf, "No birthdays in 7 days").unwrap();
    } else {
        writeln!(
            output_buf,
            "{}-+-{}",
            "-".repeat(7),
            "-".repeat(max_event_length)
        )
        .unwrap();
        writeln!(
            output_buf,
            "{:<7} | {}",
            " In",
            " ".repeat(max_event_length)
        )
        .unwrap();
        writeln!(
            output_buf,
            "{}-+-{}",
            "-".repeat(7),
            "-".repeat(max_event_length)
        )
        .unwrap();
        write!(output_buf, "{body_buf_7days}").unwrap();
        writeln!(
            output_buf,
            "{}-+-{}",
            "-".repeat(7),
            "-".repeat(max_event_length)
        )
        .unwrap();
    }
    println!("{output_buf}");
}
