use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveDateTime, Weekday};
use std::rc::Rc;

use crate::config::Config;
use crate::config::Stop;
use crate::db::Record;

pub struct Departure<'a> {
    pub stop: &'a Stop,
    pub departures: Vec<&'a Record>,
}

pub struct Timetables {
    config: Rc<Config>,
}

impl<'a> Timetables {
    pub async fn from(config: Rc<Config>) -> Result<Self, Box<dyn std::error::Error>> {
        let timetables = Timetables { config };

        Ok(timetables)
    }

    pub fn get_departures(&self) -> Vec<Departure> {
        let mut departures = vec![];

        for stop in self.config.stops.iter() {
            departures.push(Departure {
                stop,
                departures: self.get_next_departures(&stop),
            });
        }

        departures
    }

    // TODO: async
    fn get_next_departures(&self, stop: &'a Stop) -> Vec<&'a Record> {
        // let now = Local::now();
        // let date = now.date().naive_utc();

        let now = NaiveDate::from_ymd(2020, 12, 6).and_hms(16, 0, 0);
        let date = now.date();

        println!("{}", &now);

        let mut filtered_and_sorted = stop
            .database
            .records
            .iter()
            // Filter for date.
            .filter(|r| {
                if r.calendar.start_date <= date && r.calendar.end_date >= date {
                    return true;
                }

                false
            })
            // Filter for week day.
            .filter(|r| match now.weekday() {
                Weekday::Mon => r.calendar.monday,
                Weekday::Tue => r.calendar.tuesday,
                Weekday::Wed => r.calendar.wednesday,
                Weekday::Thu => r.calendar.thursday,
                Weekday::Fri => r.calendar.friday,
                Weekday::Sat => r.calendar.saturday,
                Weekday::Sun => r.calendar.sunday,
                _ => false,
            })
            .collect::<Vec<&Record>>();

        // Sort by stop time (arrival time).
        filtered_and_sorted.sort_by_key(|r| r.stop_time);

        filtered_and_sorted

        // for record in filtered_and_sorted.iter() {
        //     if record.stop_time.is_some() {
        //         println!(
        //             "{}",
        //             NaiveDateTime::from_timestamp(record.stop_time.unwrap().into(), 0)
        //                 .format("%H:%M")
        //         );
        //     }
        // }

        // println!("times: {}", filtered_and_sorted.len());
    }
}
