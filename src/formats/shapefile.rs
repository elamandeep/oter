use crate::utils::utils::{parse_dbase_value, save};
use dbase::Record;
use serde_json::{json, to_string_pretty, Map, Value};
use shapefile::{Shape, ShapeReader};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub struct ShapeFile {
    pub features_vec: Vec<Value>,
    shapes_vec: Vec<Shape>,
    records_vec: Vec<Record>,
}

impl ShapeFile {
    #[doc = "Initializing ShapeFile Struct with value"]
    pub fn new() -> Self {
        Self {
            features_vec: Vec::new(),
            shapes_vec: Vec::new(),
            records_vec: Vec::new(),
        }
    }

    pub fn populate(&mut self, path: &Path) {
        let walker = WalkDir::new(path);
        let mut shp_vec: Vec<PathBuf> = vec![];
        let mut dbf_vec: Vec<PathBuf> = vec![];
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
                        Some("shx") => {}
                        Some("prj") => {}
                        _ => {
                            //Todo fix this issue
                        }
                    }
                }
                Err(e) => eprintln!("Error: {}", e),
            };
        }

        for (i, _) in shp_vec.iter().enumerate() {
            let reader = ShapeReader::from_path(&shp_vec[i]).unwrap();
            let mut shapes = reader.read().unwrap();
            let mut reader = dbase::Reader::from_path(&dbf_vec[i]).unwrap();
            let mut records: Vec<Record> = reader.read().unwrap();
            self.records_vec.append(&mut records);
            self.shapes_vec.append(&mut shapes);
        }
    }
    #[doc = "convert shapefile into geojson"]
    pub fn to_geojson(&mut self) {
        let mut index: usize = 0;
        let mut id: u32 = 0;
        let mut line_index: usize = 0;
        for shape in &self.shapes_vec {
            match shape {
                Shape::Polygon(polygons) => {
                    for (i, p) in polygons.rings().iter().enumerate() {
                        let coordinates_vec = p
                            .points()
                            .iter()
                            .map(|point| vec![point.x, point.y])
                            .collect::<Vec<_>>();

                        if i == self.records_vec.len() {
                            break;
                        }

                        let value: Option<&Record> = self.records_vec.get(i);
                        let mut properties: Map<String, Value> = Map::new();

                        if let Some(v) = value {
                            let val: std::collections::hash_map::IntoIter<
                                String,
                                dbase::FieldValue,
                            > = v.clone().into_iter();

                            for i in val {
                                let v = parse_dbase_value(i.1);
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
                            "id":id
                        });

                        self.features_vec.push(j);
                        id += 1;
                    }
                }
                Shape::NullShape => {
                    println!("NullShape");
                }
                Shape::Point(point) => {
                    let coordinate_vec = vec![point.x, point.y];

                    let mut properties: Map<String, Value> = Map::new();
                    for record in self.records_vec.get(index).into_iter() {
                        for (k, v) in record.clone().into_iter() {
                            let v = parse_dbase_value(v);
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
                        "id":id
                    });

                    self.features_vec.push(j);
                    id += 1;
                }
                Shape::Polyline(polyline) => {
                    for elem in polyline.parts() {
                        let coordinate_vec =
                            elem.iter().map(|f| vec![f.x, f.y]).collect::<Vec<_>>();

                        let mut properties: Map<String, Value> = Map::new();
                        for record in self.records_vec.get(line_index).into_iter() {
                            for (k, v) in record.clone().into_iter() {
                                let v = parse_dbase_value(v);
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
                            "id":id
                        });

                        self.features_vec.push(j);
                        line_index += 1;
                        id += 1;
                    }
                }
                _ => {
                    println!("")
                }
            };

            index += 1;
        }

        let g = json!({
            "features":self.features_vec,
            "type":"FeatureCollection"
        });

        let content = to_string_pretty(&g).expect("Failed to serialize to JSON");

        save("geojson", content);
    }
    #[doc = "convert shapefile into topojson"]
    pub fn to_topojson(&mut self) {
        let mut index: usize = 0;
        let mut id: u32 = 0;
        let mut line_index: usize = 0;
        let mut geom_index: usize = 0;
        let mut geometry_vec: Vec<Value> = Vec::new();
        let mut arcs_vec: Vec<Value> = Vec::new();

        for shape in &self.shapes_vec {
            match shape {
                Shape::NullShape => println!("Null Point"),
                Shape::Point(point) => {
                    let coordinate = vec![point.x, point.y];

                    let mut properties: Map<String, Value> = Map::new();
                    for record in self.records_vec.get(index).into_iter() {
                        for (k, v) in record.clone().into_iter() {
                            let v = parse_dbase_value(v);
                            properties.insert(k, v);
                        }
                    }

                    let j = json!({
                        "properties":properties,
                           "coordinates":coordinate,
                            "type":"Point",
                        "id":id
                    });

                    geometry_vec.push(j);
                    id += 1;
                    index += 1;
                }
                Shape::Polyline(polyline) => {
                    for elem in polyline.parts() {
                        let coordinates = elem.iter().map(|f| vec![f.x, f.y]).collect::<Vec<_>>();

                        let mut properties: Map<String, Value> = Map::new();
                        for record in self.records_vec.get(line_index).into_iter() {
                            for (k, v) in record.clone().into_iter() {
                                let v = parse_dbase_value(v);
                                properties.insert(k, v);
                            }
                        }

                        let j = json!({
                            "type": "LineString",
                            "arcs": [
                                [
                                    geom_index
                                ]
                            ],
                            "id": id,
                            "properties": properties
                        });

                        geometry_vec.push(j);
                        arcs_vec.push(json!(coordinates));
                        geom_index += 1;
                        id += 1;
                        line_index += 1;
                    }
                }
                Shape::Polygon(polygons) => {
                    for (i, p) in polygons.rings().iter().enumerate() {
                        let coordinates = p
                            .points()
                            .iter()
                            .map(|point| vec![point.x, point.y])
                            .collect::<Vec<_>>();

                        let value: Option<&Record> = self.records_vec.get(i);
                        let mut properties: Map<String, Value> = Map::new();

                        if let Some(v) = value {
                            let val: std::collections::hash_map::IntoIter<
                                String,
                                dbase::FieldValue,
                            > = v.clone().into_iter();

                            for i in val {
                                let v = parse_dbase_value(i.1);
                                properties.insert(i.0, v);
                            }
                        }

                        let j = json!({
                            "type": "Polygon",
                            "arcs": [
                                [
                                    geom_index
                                ]
                            ],
                            "id": id,
                            "properties": properties
                        });
                        arcs_vec.push(json!(coordinates));
                        geometry_vec.push(j);
                        geom_index += 1;
                        id += 1;
                    }
                }

                _ => {
                    println!("")
                }
            }
        }

        let g = json!({
               "type": "Topology",
               "objects": {
                   "collection": {
                       "type": "GeometryCollection",
                       "geometries":geometry_vec
                   }
               },
               "arcs":arcs_vec
        });

        let content = to_string_pretty(&g).expect("Failed to serialize to JSON");

        save("topojson", content)
    }

    #[doc = "convert shapefile into KML"]
    pub fn to_kml(&mut self) {
        let mut kml = String::new();

        kml.push_str(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        kml.push_str(r#"<kml xmlns="http://www.opengis.net/kml/2.2">"#);
        kml.push_str(r#"<Document>"#);
        println!("{:?}",self.records_vec);

        for shape in &self.shapes_vec {
            match shape {
                Shape::NullShape => todo!(),
                Shape::Point(point) => {
                    
                    kml.push_str(&format!(
                        r#"<Placemark><Point><coordinates>{},{}</coordinates></Point></Placemark>"#,
                        point.x, point.y
                    ));
                }
                Shape::PointM(point) => {
                    kml.push_str(&format!(
                        r#"<Placemark><Point><coordinates>{},{}</coordinates></Point></Placemark>"#,
                        point.x, point.y
                    ));
                }
                Shape::PointZ(point) => {
                    kml.push_str(&format!(
                        r#"<Placemark><Point><coordinates>{},{},{}</coordinates></Point></Placemark>"#,
                        point.x, point.y, point.z
                    ));
                }
                Shape::Polyline(polyline) => {
                    kml.push_str("<Placemark><LineString><coordinates>");
                    for point in polyline.parts() {
                        kml.push_str(&format!("{},{},0 ", point[0].x, point[0].y));
                    }
                    kml.push_str("</coordinates></LineString></Placemark>");
                }
                Shape::PolylineM(polyline) => {
                    kml.push_str("<Placemark><LineString><coordinates>");
                    for point in polyline.parts() {
                        kml.push_str(&format!("{},{},0 ", point[0].x, point[0].y));
                    }
                    kml.push_str("</coordinates></LineString></Placemark>");
                }
                Shape::PolylineZ(polyline) => {
                    kml.push_str("<Placemark><LineString><coordinates>");
                    for point in polyline.parts() {
                        kml.push_str(&format!("{},{},0 ", point[0].x, point[0].y));
                    }
                    kml.push_str("</coordinates></LineString></Placemark>");
                }
                Shape::Polygon(polygon) => {
                    kml.push_str("<Placemark><Polygon><outerBoundaryIs><LinearRing><coordinates>");
                    for rings in polygon.rings() {
                        for point in rings.points().iter() {
                            kml.push_str(&format!("{},{},0 ", point.x, point.y));
                        }
                    }

                    kml.push_str(
                        "</coordinates></LinearRing></outerBoundaryIs></Polygon></Placemark>",
                    );
                }
                Shape::PolygonM(_) => todo!(),
                Shape::PolygonZ(_) => todo!(),
                Shape::Multipoint(_) => todo!(),
                Shape::MultipointM(_) => todo!(),
                Shape::MultipointZ(_) => todo!(),
                Shape::Multipatch(_) => todo!(),
            }
        }
        kml.push_str("</Document></kml>");

        save("kml", kml);
    }


}

#[cfg(test)]
mod shapefile_test {
    use crate::formats::shapefile::ShapeFile;
    use std::path::Path;

    #[test]
    fn test_shapefile() {
        let mut shp = ShapeFile::new();
        shp.populate(Path::new("./test"));
        shp.to_kml();
    }
}
