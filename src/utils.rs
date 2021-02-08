use gtfs_structures::Stop;
use std::sync::Arc;

pub fn is_stop_in_stops(stop: &Stop, stops: &Vec<(String, Arc<Stop>)>) -> bool {
    for (_, s) in stops {
        if s.id == stop.id {
            return true;
        }
    }

    false
}
