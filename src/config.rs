use serde::{Deserialize, Serialize};
use serde_yaml;
use std::path::PathBuf;
use tokio::fs::{self, File};
use tokio::prelude::*;

use crate::db::Database;
use crate::ui::{Wizard, WizardOutput};

const CONF_DIR: &str = "transpors";
const CONF_FILE: &str = "config.yaml";

#[derive(Serialize, Deserialize)]
pub struct Stop {
    id: String,
    pub name: String,
    pub database: Database,
}

#[derive(Serialize, Deserialize)]
pub struct Config {
    data_file_url: String,
    data_file_path: PathBuf,
    pub stops: Vec<Stop>,
}

impl Config {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let config: Self;

        // Determine config path.
        let mut dir = dirs::config_dir().expect("Config directory is not available.");
        dir.push(CONF_DIR);

        let mut conf_file_path = dir.clone();
        conf_file_path.push(CONF_FILE);

        // Check if config exists.
        if !dir.exists() {
            Self::create_conf_dir(&dir).await?;
            let mut wiz = Wizard::new(&dir).await;
            let output = wiz.run_wizard().await.expect("Wizard failed. Exiting.");

            let stops = Self::build_stops_database(&output).await;
            // println!("{:?}", stops);

            config = Self {
                data_file_url: wiz.data_file_url.unwrap(),
                data_file_path: wiz.data_file_path.unwrap().clone(),
                stops,
            };

            config.save(&conf_file_path).await?;
        } else {
            // Load config file.
            let mut config_file = File::open(&conf_file_path).await?;
            let mut file_content = String::new();
            config_file.read_to_string(&mut file_content).await?;

            // Construct Self.
            config = serde_yaml::from_str(&file_content)?;
        }

        // Debug output
        for stop in &config.stops {
            println!(
                "Stop {} has {} records in database",
                &stop.name,
                stop.database.records.len()
            );
        }

        Ok(config)
    }

    /// Creates config directory and returns path to that directory.
    async fn create_conf_dir(path: &PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        if !path.exists() {
            fs::create_dir_all(&path).await?;
        }

        Ok(())
    }

    /// Saves config (serialize) to config YAML file.
    async fn save(&self, path: &PathBuf) -> Result<(), std::io::Error> {
        fs::write(
            path,
            serde_yaml::to_string(self).expect("Couldn't serialize config."),
        )
        .await
    }

    /// Builds up stop database for each stop from config.
    async fn build_stops_database(output: &WizardOutput) -> Vec<Stop> {
        let mut stops = vec![];

        // TODO: implement rayon
        for (id, stop) in &output.stops {
            // TODO: remove unwrap set up error.
            let database = Database::from(&output.gtfs, stop).unwrap();
            stops.push(Stop {
                id: id.clone(),
                name: stop.name.clone(),
                database,
            });
        }

        stops
    }
}
