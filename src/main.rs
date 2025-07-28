use chrono::{DateTime, Duration};
use cli_table::{print_stdout, Cell, Style, Table};
use geoutils::Location;
use gpx::read;
use once_cell::sync::Lazy;
use std::{collections::HashMap, io::BufReader};

fn main() {
    let paths = std::fs::read_dir("./gpx").expect("Could not find gpx files");
    let mut stage_durations = HashMap::new();

    for path in paths {
        let file = std::fs::File::open(path.expect("Could not load path").path())
            .expect("Could not open file");
        let reader = BufReader::new(file);
        let gpx = read(reader).expect("Could not parse gpx file");
        let track = gpx
            .tracks
            .into_iter()
            .next()
            .expect("Gpx file does not contain a track");
        let segment = track
            .segments
            .into_iter()
            .next()
            .expect("Gpx track does not contain a segment");

        let mut prev_datetime: Option<DateTime<_>> = None;
        for point in segment.points {
            let coord = point.point().0;
            let stage = closest_stage(coord.y, coord.x);

            let time = point.time.expect("No time");
            let datetime =
                DateTime::parse_from_rfc3339(&time.format().expect("Could not format time"))
                    .expect("Could not parse date");

            if let Some(prev_datetime) = prev_datetime.as_ref() {
                let duration = datetime - *prev_datetime;
                let stage_duration = stage_durations
                    .entry(stage)
                    .or_insert_with(|| Duration::seconds(0));
                *stage_duration = *stage_duration + duration;
            }

            prev_datetime = Some(datetime);
        }
    }

    let mut stage_durations: Vec<_> = stage_durations.into_iter().collect();
    stage_durations.sort_by_key(|stage_duration| stage_duration.1);
    stage_durations.reverse();
    let stage_durations_table = stage_durations
        .into_iter()
        .map(|(stage, duration)| {
            vec![
                stage,
                format!(
                    "{}:{:02}",
                    duration.num_hours(),
                    duration.num_minutes() % 60
                ),
            ]
        })
        .collect::<Vec<_>>()
        .table()
        .title(vec![
            "Stage".cell().bold(true),
            "Duration".cell().bold(true),
        ]);
    print_stdout(stage_durations_table).expect("Could not print table");
}

fn closest_stage(lat: f64, lon: f64) -> String {
    let location = Location::new(lat, lon);
    let mut distances: Vec<(String, f64)> = STAGES
        .iter()
        .map(|stage| (stage.name.clone(), stage.distance_meters(&location)))
        .collect();
    distances.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
    distances.into_iter().next().unwrap().0
}

static STAGES: Lazy<Vec<Stage>> = Lazy::new(|| {
    vec![
        Stage::new("HPTTRSN", 53.30959, 12.73694),
        Stage::new("Turmbühne", 53.30864, 12.73732),
        Stage::new("Dubstation", 53.30766, 12.73471),
        Stage::new("Casino", 53.30659, 12.73436),
        Stage::new("FreiKörperKüste", 53.30399, 12.73192),
        Stage::new("Tanzwüste", 53.31186, 12.73839),
        Stage::new("Stoners Garden", 53.31189, 12.74076),
        Stage::new("Trancefloor", 53.31224, 12.74221),
        Stage::new("Rootsbase", 53.31198, 12.74359),
        Stage::new("Extravaganza", 53.31059, 12.74393),
        Stage::new("Sonnendeck", 53.31089, 12.74322),
        Stage::new("Palapa", 53.30974, 12.74381),
        Stage::new("Panne Eichel", 53.31049, 12.75295),
    ]
});

#[derive(Debug)]
struct Stage {
    name: String,
    location: Location,
}

impl Stage {
    fn new(name: &str, lat: f64, lon: f64) -> Self {
        Self {
            name: name.to_owned(),
            location: Location::new(lat, lon),
        }
    }

    fn distance_meters(&self, other: &Location) -> f64 {
        self.location
            .distance_to(other)
            .expect("Could not calculate distance")
            .meters()
    }
}
