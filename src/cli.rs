use crate::formats::shapefile::ShapeFile;
use clap::{command, Parser};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// convert shapefile into geojson
    #[arg(long = "shpgeojson", value_name = "FOLDER NAME")]
    shpgeojson: Option<PathBuf>,

    /// convert shapefile into topojson
    #[arg(long = "shptopojson", value_name = "FOLDER NAME")]
    shptopojson: Option<PathBuf>,

    /// convert shapefile into KML
    #[arg(long = "shpkml", value_name = "FOLDER NAME")]
    shpkml: Option<PathBuf>,
}

pub fn init() {
    let cli = Cli::parse();

    if let Some(shpgeojson) = cli.shpgeojson.as_deref() {
        let mut shp = ShapeFile::new();
        shp.populate(shpgeojson);
        shp.to_geojson();
    }

    if let Some(shptopojson) = cli.shptopojson.as_deref() {
        let mut shp = ShapeFile::new();
        shp.populate(shptopojson);
        shp.to_topojson();
    }

    if let Some(shpkml) = cli.shpkml.as_deref() {
        let mut shp = ShapeFile::new();
        shp.populate(shpkml);
        shp.to_kml();
    }
}
