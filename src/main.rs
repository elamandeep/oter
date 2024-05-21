use clap::{error, Parser, Subcommand};
use dbase::{DateTime, FieldValue, ReadableRecord, Record};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{self, Deserialize, Serialize};
use serde_json::{json, to_string_pretty, Map, Value};
use uuid::Uuid;

use std::{
    any::Any,
    collections::HashMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::SystemTime,
    vec,
};

use shapefile::{
    self,
    record::{self, point, polygon},
    ReadableShape, Reader, Shape, ShapeReader,
};
use walkdir::{DirEntry, WalkDir};

enum Target {
    Files(Vec<String>),
    Folder(String),
}

#[derive(Serialize, Deserialize, Debug)]
struct Geojson {
    #[serde(rename = "type")]
    geo_type: String,
    features: Vec<Value>,
}

struct AppState {
    index: usize,
    id: u32,
    line_index: usize,
}
impl AppState {
    fn new() -> Self {
        AppState {
            index: 0,
            id: 0,
            line_index: 0,
        }
    }

    fn increment_index(&mut self) {
        self.index += 1
    }

    fn increment_id(&mut self) {
        self.id += 1
    }
    fn increment_line_index(&mut self) {
        self.line_index += 1
    }
}

fn merge_geojsons_into_one(target: Target, output: String) {
    let features_vec: Vec<Value> = Vec::new();

    match target {
        Target::Files(files) => {
            let file_length = files.clone().len();
            let mut curr_index: usize = 0;

            while curr_index < file_length {
                let file_name = match files.get(curr_index) {
                    Some(files) => Path::new(files),
                    None => {
                        return;
                    }
                };

                let data = fs::read_to_string(file_name).expect("Unable to read file");
                let json: serde_json::Value =
                    serde_json::from_str(&data).expect("JSON does not have correct format.");

                while let Some(value) = json["features"].as_array() {
                    // checkpoint
                }

                curr_index += 1;
            }
        }
        Target::Folder(folder) => {
            println!("Merging files in the folder: {}", folder);
        }
    }

    let geojson = Geojson {
        geo_type: String::from("FeatureCollection"),
        features: features_vec,
    };
}

fn extract_value(field_value: FieldValue) -> Value {
    match field_value {
        FieldValue::Character(c) => {
            json!(Some(c))
        }
        FieldValue::Numeric(n) => json!(Some(n)),
        FieldValue::Logical(l) => json!(Some(l)),
        FieldValue::Date(d) => {
            if let Some(d) = d {
                json!(d.to_unix_days())
            } else {
                json!("")
            }
        }
        FieldValue::Float(f) => json!(Some(f)),
        FieldValue::Integer(i) => json!(Some(i)),
        FieldValue::Currency(c) => json!(c),
        FieldValue::DateTime(d) => {
            json!(d.to_unix_timestamp())
        }
        FieldValue::Double(d) => json!(d),
        FieldValue::Memo(m) => json!(m),
    }
}

fn transform_shapes(
    shape: Shape,
    records: &Vec<Record>,
    features_vec: &mut Vec<Value>,
    state: &mut AppState,
) {
    match shape {
        Shape::Polygon(polygons) => {
            for (i, p) in polygons.rings().iter().enumerate() {
                let coordinates_vec = p
                    .points()
                    .iter()
                    .map(|point| vec![point.x, point.y])
                    .collect::<Vec<_>>();

                if i == records.len() {
                    break;
                }

                let value: Option<&Record> = records.get(i);
                let mut properties: Map<String, Value> = Map::new();

                if let Some(v) = value {
                    let val: std::collections::hash_map::IntoIter<String, dbase::FieldValue> =
                        v.clone().into_iter();

                    for i in val {
                        let v = extract_value(i.1);
                        properties.insert(i.0, v);
                    }
                }
                let j = json!({
                    "type":"Feature",
                    "properties":properties,
                    "geometry":{
                       "coordinates":[
                            coordinates_vec
                        ],
                        "type":"Polygon"
                    },
                    "id":state.id
                });

                features_vec.push(j);
                state.increment_id();
            }
        }
        Shape::NullShape => {
            println!("NullShape");
        }
        Shape::Point(point) => {
            let coordinate_vec = vec![point.x, point.y];

            let mut properties: Map<String, Value> = Map::new();
            for record in records.get(state.index).into_iter() {
                for (k, v) in record.clone().into_iter() {
                    let v = extract_value(v);
                    properties.insert(k, v);
                }
            }

            let j = json!({
                "type":"Feature",
                "properties":properties,
                "geometry":{
                   "coordinates":coordinate_vec,
                    "type":"Point"
                },
                "id":state.id
            });

            features_vec.push(j);
            state.increment_id();
        }
        Shape::PointM(_) => {
            println!("PointM");
        }
        Shape::PointZ(_) => {
            println!("PointZ");
        }
        Shape::Polyline(polyline) => {
            for elem in polyline.parts() {
                let coordinate_vec = elem.iter().map(|f| vec![f.x, f.y]).collect::<Vec<_>>();

                let mut properties: Map<String, Value> = Map::new();
                for record in records.get(state.line_index).into_iter() {
                    for (k, v) in record.clone().into_iter() {
                        let v = extract_value(v);
                        properties.insert(k, v);
                    }
                }

                let j = json!({
                    "type":"Feature",
                    "properties":properties,
                    "geometry":{
                       "coordinates":coordinate_vec,
                        "type":"LineString"
                    },
                    "id":state.id
                });

                features_vec.push(j);
                state.increment_line_index();
                state.increment_id();
            }
        }
        Shape::PolylineM(_) => {
            println!("");
        }
        Shape::PolylineZ(_) => {
            println!("");
        }
        Shape::PolygonM(_) => {
            println!("");
        }
        Shape::PolygonZ(_) => {
            println!("");
        }
        Shape::Multipoint(_) => {
            println!("");
        }
        Shape::MultipointM(_) => {
            println!("");
        }
        Shape::MultipointZ(_) => {
            println!("");
        }
        Shape::Multipatch(_) => {
            println!("");
        }
    };
    state.increment_index();
}

fn extract_info(shp_vec: Vec<PathBuf>, dbf_vec: Vec<PathBuf>, features_vec: &mut Vec<Value>) {
    let mut state = AppState::new();
    for (i, _) in shp_vec.iter().enumerate() {
        let reader = shapefile::ShapeReader::from_path(&shp_vec[i]).unwrap();
        let shapes = reader.read().unwrap();
        let mut reader = dbase::Reader::from_path(&dbf_vec[i]).unwrap();

        let records: Vec<Record> = reader.read().unwrap();

        for shape in shapes {
            transform_shapes(shape, &records, features_vec, &mut state);
        }
    }
}

fn convert_shapefile_into_geojson(shp: &Path) {
    let walker = WalkDir::new(shp);
    let mut shp_vec: Vec<PathBuf> = vec![];
    let mut dbf_vec: Vec<PathBuf> = vec![];
    let mut features_vec: Vec<Value> = Vec::new();

    for entry in walker {
        match entry {
            Ok(entry) => {
                let file_path = entry.path();

                let cloned_path = file_path.to_owned();
                let file_extension = file_path.extension().and_then(|f| f.to_str());

                match file_extension {
                    Some("shp") => {
                        shp_vec.push(cloned_path);
                    }
                    Some("dbf") => {
                        dbf_vec.push(cloned_path);
                    }
                    _ => {}
                }
            }
            Err(e) => eprintln!("Error: {}", e),
        };
    }

    extract_info(shp_vec, dbf_vec, &mut features_vec);

    let g = Geojson {
        geo_type: String::from("FeatureCollection"),
        features: features_vec,
    };

    let id = Uuid::new_v4();

    let json_string = to_string_pretty(&g).expect("Failed to serialize to JSON");

    let file_name = format!("{}.geojson", id.to_string());

    let mut f = fs::File::create(file_name).unwrap();

    f.write_all(json_string.trim_end().as_bytes()).unwrap();
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long = "shpgjson", value_name = "FOLDER NAME")]
    shpgeojson: Option<PathBuf>,
    // #[command(subcommand)]
    // command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Merge files
    MergeFiles {
        /// Files to merge
        #[arg(required = true)]
        files: Vec<String>,

        /// Output file
        #[arg(short, long, value_name = "OUTPUT")]
        output: String,
    },
    /// Merge folder
    MergeFolder {
        /// Folder containing files to merge
        #[arg(required = true)]
        folder: String,

        /// Output file
        #[arg(short, long, value_name = "OUTPUT")]
        output: String,
    },
}

fn main() {
    let cli = Cli::parse();

    if let Some(shp) = cli.shpgeojson.as_ref() {
        let pb = ProgressBar::new_spinner();
        // Todo create progress bar

        // pb.set_style(
        //     ProgressStyle::default_spinner()
        //         .tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"])
        //         .template("{spinner:.green} {msg}"),
        // );
        pb.enable_steady_tick(200);
        pb.set_message("Converting shapefile to GeoJSON...");
        convert_shapefile_into_geojson(shp);

        pb.finish_with_message("Conversion completed successfully!");
    }
}
