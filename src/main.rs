mod config;
mod db;
mod timetables;
mod ui;
mod utils;

use std::rc::Rc;

use config::Config;
use timetables::Timetables;
use ui::Ui;

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create/get config.
    let config = Rc::new(Config::new().await?);

    let timetables = Timetables::from(config.clone()).await?;
    let departures = timetables.get_departures();

    let ui = Ui::new(&config);
    ui.output(departures);

    Ok(())
}
