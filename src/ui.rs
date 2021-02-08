use std::io::{self, prelude::*, BufRead};
use std::path::PathBuf;
use std::sync::Arc;

use chrono::NaiveDateTime;
use gtfs_structures::{Gtfs, Stop};
use rayon::prelude::*;
use tokio::fs;

use crate::config::Config;
// use crate::db::Record;
use crate::timetables::Departure;

pub struct WizardOutput {
    pub gtfs: Gtfs,
    pub stops: Vec<(String, Arc<Stop>)>,
}

/// Wizard for user that ask a few questions.
/// The result is used by Config struct.
pub struct Wizard<'a> {
    pub data_file_url: Option<String>,
    conf_dir: &'a PathBuf,
    pub data_file_path: Option<PathBuf>,
}

impl<'a> Wizard<'a> {
    pub async fn new(conf_dir: &'a PathBuf) -> Wizard<'a> {
        Wizard {
            data_file_url: None,
            conf_dir,
            data_file_path: None,
        }
    }

    pub async fn run_wizard(&mut self) -> Result<WizardOutput, Box<dyn std::error::Error>> {
        let gtfs = self.retrieve_data_file().await?;
        let stops = self.read_stop_names(&gtfs)?;
        // let times = self.read_stop_times(&gtfs, &stops)?;

        Ok(WizardOutput { gtfs, stops })
        // Ok(WizardOutput { gtfs, stops, times })
    }

    /// Downloads or copies (depends on the origin location) the datafile
    /// to project config location (see Config.path).
    async fn retrieve_data_file(&mut self) -> Result<Gtfs, Box<dyn std::error::Error>> {
        // Read data file path/URL.
        println!("Enter data file path/URL: ");
        let mut data_file = String::new();
        io::stdin().lock().read_line(&mut data_file)?;
        data_file = data_file.trim().to_owned();
        self.data_file_url = Some(data_file.clone());

        // Download or copy data file.
        self.data_file_path = Some(
            self.download_or_copy_data_file(self.conf_dir, data_file)
                .await?,
        );

        print!("Parsing ...");
        io::stdout().flush().unwrap();
        let gtfs = Gtfs::new(self.data_file_path.clone().unwrap().to_str().unwrap())?;
        println!(" done!");

        Ok(gtfs)
    }

    /// Downloads or copies the data file into config folder.
    async fn download_or_copy_data_file(
        &self,
        conf_dir: &PathBuf,
        path_or_url: String,
    ) -> Result<PathBuf, Box<dyn std::error::Error>> {
        // Prepare data file path.
        let mut data_file_path = conf_dir.clone();
        data_file_path.push("data_file.gtfs");

        // Download or copy.
        if path_or_url.starts_with("http") {
            print!("Downloading ...");
            io::stdout().flush()?;
            fs::write(
                &data_file_path,
                reqwest::get(&path_or_url).await?.bytes().await?,
            )
            .await?;
            println!(" done!");
        } else {
            fs::copy(&path_or_url, &data_file_path).await?;
        }

        Ok(data_file_path)
    }

    /// Triggers the loop for reading stop names. User can
    /// enter as many stops as he likes.
    fn read_stop_names(
        &self,
        gtfs: &'a Gtfs,
    ) -> Result<Vec<(String, Arc<Stop>)>, Box<dyn std::error::Error>> {
        let mut chosen_stops = vec![];

        loop {
            // Read stop name.
            chosen_stops.push(self.read_stop_name(gtfs)?);

            // Ask for more stops.
            println!(
                "Currently {} stop(s) has been chosen. Do you want to add another one? (y/n)",
                chosen_stops.len()
            );
            let mut answer = String::new();
            io::stdin().lock().read_line(&mut answer).unwrap();
            answer = answer.trim().to_owned();

            if "y" != answer.to_lowercase() {
                break;
            }
        }

        Ok(chosen_stops)
    }

    /// Tries to collect one stop based on user input.
    fn read_stop_name(
        &self,
        gtfs: &'a Gtfs,
    ) -> Result<(String, Arc<Stop>), Box<dyn std::error::Error>> {
        loop {
            let mut found_stops = self.seek_stops(gtfs)?;

            println!("Found {} stops:", found_stops.len());

            // Sort found stops by stop name.
            found_stops.sort_by_key(|i| i.1.name.clone());

            // Paralelly iterate thru stops and fetch terminating station for
            // each station (based on trip direction).
            let found_stops_with_terminating_stop: Vec<(String, Arc<Stop>, Arc<Stop>)> =
                found_stops
                    .par_iter()
                    .map(|item| {
                        (
                            item.0.clone(),
                            item.1.clone(),
                            self.get_last_trip_stop_for_stop(gtfs, item.1.clone()),
                        )
                    })
                    .collect();

            // Print stops.
            for (i, (_, stop, terminating_stop)) in
                found_stops_with_terminating_stop.iter().enumerate()
            {
                println!("{}) {} -> {}", i, stop, terminating_stop);
            }

            // Read stop number.
            let stop_number: usize;

            loop {
                println!("Please enter the number of stop you want to choose:");
                let mut stop_number_input = String::new();
                io::stdin().lock().read_line(&mut stop_number_input)?;

                // Validate stop number.
                match stop_number_input.trim().parse::<usize>() {
                    Ok(number) => {
                        stop_number = number;
                        break;
                    }
                    Err(_) => {
                        println!("Wrong number! Try again.");
                    }
                }
            }

            if let Some(stop) = found_stops.get(stop_number) {
                return Ok(stop.clone());
            }

            println!("Wrong number! Try again.");
        }
    }

    /// Asks user for input and then finds similar stops in datafile.
    /// All similar stops are then returned.
    /// If no similar stop are found user is asked for the input again.
    fn seek_stops(
        &self,
        gtfs: &'a Gtfs,
    ) -> Result<Vec<(String, Arc<Stop>)>, Box<dyn std::error::Error>> {
        let mut found_stops: Vec<(String, Arc<Stop>)>;

        loop {
            println!("Enter stop name: ");
            let mut stop = String::new();
            io::stdin().lock().read_line(&mut stop)?;
            stop = stop.trim().to_owned();

            // Validate stop name against data file.
            found_stops = gtfs
                .stops
                .iter()
                .filter(|x| x.1.name.contains(stop.as_str()))
                .map(|x| (x.0.clone(), x.1.clone()))
                .collect();

            // We did found at least one stop.
            if !found_stops.is_empty() {
                break;
            }

            println!("No stop with such name (or similar) was found. Please try again.");
        }

        Ok(found_stops)
    }

    /// Seeks last stop (terminating station) for the given stop (based on
    /// associated trip and stop times.
    fn get_last_trip_stop_for_stop(&self, gtfs: &'a Gtfs, stop: Arc<Stop>) -> Arc<Stop> {
        let mut found_stop: Option<Arc<Stop>> = None;

        // Closes thing to stops we have are trips.
        for (_, trip) in gtfs.trips.iter() {
            for time in trip.stop_times.iter() {
                if time.stop.id == stop.id {
                    found_stop = Some(
                        trip.stop_times
                            .last()
                            .expect("Trip ha no stop - that's very weird!")
                            .stop
                            .clone(),
                    );
                }
            }
        }

        if found_stop.is_none() {
            return stop;
        }

        found_stop.unwrap()
    }
}

pub struct Ui<'a> {
    config: &'a Config,
}

impl<'a> Ui<'a> {
    pub fn new(config: &'a Config) -> Self {
        Self { config }
    }

    pub fn output(&self, departures: Vec<Departure>) {
        for departure in departures.iter() {
            // Heading.
            let stop_name = &departures.first().unwrap().stop.name;
            println!("{}", stop_name);
            println!("{}", "-".repeat(stop_name.chars().count()));

            // Timetable.
            for departure_record in departure.departures.iter() {
                if departure_record.stop_time.is_some() {
                    println!(
                        "{} {} {}",
                        departure_record.route,
                        departure_record.trip,
                        NaiveDateTime::from_timestamp(
                            departure_record.stop_time.unwrap().into(),
                            0
                        )
                        .format("%H:%M")
                    );
                }
            }
        }
    }
}
