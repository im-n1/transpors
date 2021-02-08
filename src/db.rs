use chrono::NaiveDate;
use gtfs_structures::{Gtfs, Stop};
use serde::{Deserialize, Serialize};

// use crate::utils::is_stop_in_stops;

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomCalendar {
    pub monday: bool,
    pub tuesday: bool,
    pub wednesday: bool,
    pub thursday: bool,
    pub friday: bool,
    pub saturday: bool,
    pub sunday: bool,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

impl From<&gtfs_structures::Calendar> for CustomCalendar {
    fn from(cal: &gtfs_structures::Calendar) -> Self {
        Self {
            monday: cal.monday.clone(),
            tuesday: cal.tuesday.clone(),
            wednesday: cal.wednesday.clone(),
            thursday: cal.thursday.clone(),
            friday: cal.friday.clone(),
            saturday: cal.saturday.clone(),
            sunday: cal.sunday.clone(),
            start_date: cal.start_date.clone(),
            end_date: cal.end_date.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Record {
    pub route: String, // human readable line name
    pub trip: String,
    pub calendar: CustomCalendar,
    pub stop_time: Option<u32>,
    pub stop: String,
}

#[derive(Serialize, Deserialize)]
pub struct Database {
    // TODO: vec -> array
    pub records: Vec<Record>,
}

impl<'a> Database {
    pub fn from(gtfs: &'a Gtfs, stop: &Stop) -> Result<Self, Box<dyn std::error::Error>> {
        let records = Self::fetch(gtfs, stop)?;
        // Self::debug(routes_and_calendars);

        Ok(Self { records })
    }

    /// Walks thru all stops and collects all trips that intersect any
    /// of selected stop.
    fn fetch(gtfs: &'a Gtfs, stop: &Stop) -> Result<Vec<Record>, Box<dyn std::error::Error>> {
        let mut records = vec![];

        // TODO: rayon optimalization
        for (_, route) in &gtfs.routes {
            for (_, trip) in gtfs.trips.iter().filter(|trip| trip.1.route_id == route.id) {
                for time in &trip.stop_times {
                    if time.stop.id == stop.id {
                        // if is_stop_in_stops(&time.stop, stops) {
                        records.push(Record {
                            // route: route.long_name.clone(),
                            route: route.short_name.clone(),
                            trip: trip.service_id.clone(),
                            calendar: CustomCalendar::from(
                                gtfs.get_calendar(trip.service_id.as_str())?,
                            ),
                            stop_time: time.arrival_time.clone(),
                            stop: time.stop.name.clone(),
                        });
                    }
                }
            }
        }

        Ok(records)
    }

    // fn debug(found_routes: Vec<(&Route, &Calendar)>) {
    //     let uniq_routes_with_cals = found_routes
    //         .into_iter()
    //         .unique_by(|r| (&r.0.id, &r.1.id))
    //         .collect::<Vec<(&Route, &Calendar)>>();

    //     println!("Found {} routes", uniq_routes_with_cals.len());

    //     for (route, cal) in uniq_routes_with_cals {
    //         println!(
    //             "{}, {}, {}, {} - {}",
    //             route.id, route.short_name, route.long_name, cal.start_date, cal.end_date
    //         );
    //         println!(
    //             "{}, {}, {}, {}, {}, {}, {}, {}",
    //             cal.id,
    //             cal.monday,
    //             cal.tuesday,
    //             cal.wednesday,
    //             cal.thursday,
    //             cal.friday,
    //             cal.saturday,
    //             cal.sunday
    //         );
    //     }

    //     // for (name, trip) in &gtfs.trips {
    //     //     for time in &trip.stop_times {
    //     //         if is_stop_in_stops(&time.stop, &stops) {
    //     //             println!("{}", trip.route_id);
    //     //             // let dur = ChDuration::from_std(Duration::from_secs(
    //     //             //     time.arrival_time.clone().unwrap().into(),
    //     //             // ))
    //     //             // .unwrap();

    //     //             // println!("{:?}", time.arrival_time);
    //     //             // let arrival_time = time.arrival_time.clone().unwrap()
    //     //             // let time_str = arrival_time.to_string();
    //     //             // let time = NaiveTime::parse_from_str(time_str.as_str(), "%s").unwrap();
    //     //             let date_time =
    //     //                 NaiveDateTime::from_timestamp(time.arrival_time.clone().unwrap().into(), 0);
    //     //             // println!("{}:{}", time.hour(), time.minute());
    //     //             println!("{}", date_time.format("%H:%M"));
    //     //             println!("{:?}", time);
    //     //             println!("");
    //     //         }
    //     //     }
    //     // }
    // }
}
