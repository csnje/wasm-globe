// Build script used to generate Rust code containing data structures.

use std::fs::File;
use std::io::{BufWriter, Write};

use shapefile::PolygonRing;

const DATA_FILENAME: &str = "src/data.rs";
const COASTLINE_SHAPEFILE_FILENAME: &str = "data/ne_110m_coastline/ne_110m_coastline.shp";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(DATA_FILENAME)?;
    let mut file = BufWriter::new(file);

    file.write_all("// This file is code generated.\n\n".as_bytes())?;
    write_data(&mut file, COASTLINE_SHAPEFILE_FILENAME, "COASTLINE_POINTS")?;

    Ok(())
}

fn write_data(
    file: &mut BufWriter<File>,
    shapefile_filename: &str,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    file.write_all(format!("pub const {}: &[&[(f64, f64)]] = &[\n", name).as_bytes())?;

    let mut reader = shapefile::Reader::from_path(shapefile_filename)?;
    for shape_record in reader.iter_shapes_and_records() {
        let (shape, _record) = shape_record?;
        match shape {
            shapefile::Shape::Polyline(polyline) => {
                file.write_all("    &[\n".as_bytes())?;
                for part in polyline.parts() {
                    for point in part {
                        file.write_all(
                            format!("        ({}f64, {}f64),\n", point.x, point.y).as_bytes(),
                        )?;
                    }
                }
                file.write_all("    ],\n".as_bytes())?;
            }
            shapefile::Shape::Polygon(polygon) => {
                file.write_all("    &[\n".as_bytes())?;
                for ring in polygon.rings() {
                    match ring {
                        PolygonRing::Outer(points) => {
                            for point in points {
                                file.write_all(
                                    format!("        ({}f64, {}f64),\n", point.x, point.y)
                                        .as_bytes(),
                                )?;
                            }
                        }
                        PolygonRing::Inner(_) => {}
                    }
                }
                file.write_all("    ],\n".as_bytes())?;
            }
            _ => file.write_all(format!("!!!ERROR({})!!!", shape).as_bytes())?,
        }
    }
    file.write_all("];\n".as_bytes())?;

    Ok(())
}
