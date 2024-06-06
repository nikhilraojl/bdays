mod home_dir;

use chrono::Datelike;
use chrono::Local;
use chrono::TimeDelta;
use chrono::TimeZone;
use home_dir::get_home_dir;
use std::fmt::Write;
use std::{fs::File, io::BufReader};

#[derive(Debug)]
enum Rrule {
    Yearly(YearlyRule),
    // Weekly(WeeklyRule),
}

impl Rrule {
    fn from_outlook_event(rule: &str) -> Option<Self> {
        // TODO: directly indexing into vec may blow up
        let rule_split = rule.split(';').collect::<Vec<&str>>();
        let freq_value = rule_split[0].split('=').collect::<Vec<&str>>();
        match freq_value[1] {
            "YEARLY" => {
                let by_month_day = rule_split[1].split('=').collect::<Vec<&str>>()[1]
                    .parse()
                    .unwrap();
                let by_month = rule_split[2].split('=').collect::<Vec<&str>>()[1]
                    .parse()
                    .unwrap();
                // println!("{by_month_day:?}, {by_month:?}");
                Some(Self::Yearly(YearlyRule {
                    by_month_day,
                    by_month,
                }))
            }
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

    // let ins_now = std::time::Instant::now();

    let home_dir = get_home_dir().unwrap();
    let cal_path = home_dir.join("./cal.ics");
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
    if let Some(_arg) = args.next() {
        eprintln!("No arguments needed!");
        return;
    } else {
        find_birthdays(&events)
    }

    // println!("{}", ins_now.elapsed().as_millis());
}

fn find_birthdays(events: &Vec<Event>) {
    let now = Local::now();
    let curr_day = now.day();
    let curr_month = now.month();
    // let curr_day = 2;
    // let curr_month = 6;

    // let mut output_today = String::new();
    let mut output_today: Vec<&str> = Vec::new();

    let until_date = now + TimeDelta::days(7);
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
                // let output_str = format!("{} {}", in_days.num_days(), e_name);
                // output_buf.push_str(&output_str);
                // output_buf.push('\n');
            }
        }
    }

    if output_today.is_empty() {
        println!("No brithdays today");
    } else {
        let mut max_event_length = 5;
        let mut output_buf = String::new();

        let mut body_buf_today = String::new();
        // writeln!(output_buf, "TODAY");
        // writeln!(output_buf, "-----");
        for e in &output_today {
            if e.len() > max_event_length {
                max_event_length = e.len();
            }
            body_buf_today.push_str(&format!("> {}", e));
            body_buf_today.push('\n');
        }

        output_7days.sort_by(|a, b| a.0.cmp(&b.0));
        let mut body_buf_7days = String::new();
        for e in output_7days {
            if e.1.len() > max_event_length {
                max_event_length = e.1.len();
            }
            body_buf_7days.push_str(&format!("{:>2} days | {} ", e.0, e.1));
            body_buf_7days.push('\n');
        }

        writeln!(output_buf, "-----").unwrap();
        writeln!(output_buf, "TODAY").unwrap();
        writeln!(output_buf, "-----").unwrap();
        writeln!(output_buf, "{}", body_buf_today).unwrap();

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
        write!(output_buf, "{}", body_buf_7days).unwrap();
        writeln!(
            output_buf,
            "{}-+-{}",
            "-".repeat(7),
            "-".repeat(max_event_length)
        )
        .unwrap();
        println!("{}", output_buf);
    }
}
