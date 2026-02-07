use geojson::{FeatureCollection, Value};
use gpx2geojson_wasm::converter::to_feature_collection;
use gpx2geojson_wasm::options::ConvertOptions;
use gpx2geojson_wasm::parser::parse_gpx;

fn load_fixture(path: &str) -> String {
    std::fs::read_to_string(format!("tests/fixtures/{path}")).unwrap()
}

fn convert(gpx: &str) -> FeatureCollection {
    let data = parse_gpx(gpx).unwrap();
    to_feature_collection(&data, &ConvertOptions::default())
}

fn convert_with_opts(gpx: &str, opts: &ConvertOptions) -> FeatureCollection {
    let data = parse_gpx(gpx).unwrap();
    to_feature_collection(&data, opts)
}

// ---- basic/ ----

#[test]
fn test_01_minimal_waypoint() {
    let fc = convert(&load_fixture("basic/01_minimal_waypoint.gpx"));
    assert_eq!(fc.features.len(), 1);

    let f = &fc.features[0];
    let geom = f.geometry.as_ref().unwrap();
    if let Value::Point(coords) = &geom.value {
        assert!((coords[0] - 139.6503).abs() < 1e-4); // lon
        assert!((coords[1] - 35.6762).abs() < 1e-4); // lat
        assert_eq!(coords.len(), 2); // no elevation
    } else {
        panic!("Expected Point");
    }

    let props = f.properties.as_ref().unwrap();
    assert_eq!(props["gpxType"], "waypoint");
}

#[test]
fn test_02_full_waypoint() {
    let fc = convert(&load_fixture("basic/02_full_waypoint.gpx"));
    assert_eq!(fc.features.len(), 1);

    let props = fc.features[0].properties.as_ref().unwrap();
    assert_eq!(props["gpxType"], "waypoint");
    assert_eq!(props["name"], "Tokyo Tower");
    assert_eq!(props["cmt"], "A comment");
    assert_eq!(props["desc"], "A famous landmark in Tokyo");
    assert_eq!(props["src"], "GPS");
    assert_eq!(props["sym"], "Flag, Blue");
    assert_eq!(props["type"], "POI");
    assert_eq!(props["time"], "2025-01-01T12:00:00Z");
    assert_eq!(props["ele"], 40.5);

    let link = props["link"].as_object().unwrap();
    assert_eq!(link["href"], "https://example.com/tokyo-tower");
    assert_eq!(link["text"], "Tokyo Tower Website");
    assert_eq!(link["type"], "text/html");
}

#[test]
fn test_03_simple_route() {
    let fc = convert(&load_fixture("basic/03_simple_route.gpx"));
    assert_eq!(fc.features.len(), 1);

    let f = &fc.features[0];
    let props = f.properties.as_ref().unwrap();
    assert_eq!(props["gpxType"], "route");
    assert_eq!(props["name"], "Tokyo Loop");
    assert_eq!(props["desc"], "A route around central Tokyo");
    assert_eq!(props["number"], 1);

    let geom = f.geometry.as_ref().unwrap();
    if let Value::LineString(coords) = &geom.value {
        assert_eq!(coords.len(), 3);
    } else {
        panic!("Expected LineString");
    }
}

#[test]
fn test_04_simple_track() {
    let fc = convert(&load_fixture("basic/04_simple_track.gpx"));
    assert_eq!(fc.features.len(), 1);

    let f = &fc.features[0];
    let props = f.properties.as_ref().unwrap();
    assert_eq!(props["gpxType"], "track");
    assert_eq!(props["name"], "Morning Run");
    assert_eq!(props["type"], "running");

    let geom = f.geometry.as_ref().unwrap();
    if let Value::LineString(coords) = &geom.value {
        assert_eq!(coords.len(), 5);
        // Check [lon, lat, ele] order
        assert!((coords[0][0] - 139.6503).abs() < 1e-4); // lon
        assert!((coords[0][1] - 35.6762).abs() < 1e-4); // lat
        assert!((coords[0][2] - 10.0).abs() < 1e-4); // ele
    } else {
        panic!("Expected LineString");
    }

    // Check coordinateProperties.times
    let coord_props = props["coordinateProperties"].as_object().unwrap();
    let times = coord_props["times"].as_array().unwrap();
    assert_eq!(times.len(), 5);
    assert_eq!(times[0], "2025-01-01T06:00:00Z");
    assert_eq!(times[4], "2025-01-01T06:04:00Z");
}

#[test]
fn test_05_complete() {
    let fc = convert(&load_fixture("basic/05_complete.gpx"));
    assert_eq!(fc.features.len(), 3);

    let types: Vec<&str> = fc
        .features
        .iter()
        .map(|f| {
            f.properties.as_ref().unwrap()["gpxType"]
                .as_str()
                .unwrap()
        })
        .collect();
    assert_eq!(types, vec!["waypoint", "route", "track"]);
}

// ---- tracks/ ----

#[test]
fn test_06_multi_segment_separate() {
    let fc = convert(&load_fixture("tracks/06_multi_segment.gpx"));
    // Default: each segment is a separate Feature
    assert_eq!(fc.features.len(), 2);
    for f in &fc.features {
        let geom = f.geometry.as_ref().unwrap();
        assert!(matches!(&geom.value, Value::LineString(_)));
    }
}

#[test]
fn test_06_multi_segment_joined() {
    let gpx = load_fixture("tracks/06_multi_segment.gpx");
    let opts = ConvertOptions {
        join_track_segments: true,
        ..Default::default()
    };
    let fc = convert_with_opts(&gpx, &opts);
    assert_eq!(fc.features.len(), 1);

    let geom = fc.features[0].geometry.as_ref().unwrap();
    if let Value::MultiLineString(lines) = &geom.value {
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0].len(), 2);
        assert_eq!(lines[1].len(), 2);
    } else {
        panic!("Expected MultiLineString");
    }
}

#[test]
fn test_07_multi_track() {
    let fc = convert(&load_fixture("tracks/07_multi_track.gpx"));
    assert_eq!(fc.features.len(), 2);

    let names: Vec<&str> = fc
        .features
        .iter()
        .map(|f| {
            f.properties.as_ref().unwrap()["name"]
                .as_str()
                .unwrap()
        })
        .collect();
    assert_eq!(names, vec!["Morning Run", "Evening Walk"]);
}

#[test]
fn test_08_single_point_track() {
    let fc = convert(&load_fixture("tracks/08_single_point_track.gpx"));
    assert_eq!(fc.features.len(), 1);

    let geom = fc.features[0].geometry.as_ref().unwrap();
    assert!(
        matches!(&geom.value, Value::Point(_)),
        "Single-point track should become a Point Feature"
    );
}

// ---- edge_cases/ ----

#[test]
fn test_09_empty() {
    let fc = convert(&load_fixture("edge_cases/09_empty.gpx"));
    assert!(fc.features.is_empty());
}

#[test]
fn test_10_empty_segments() {
    let fc = convert(&load_fixture("edge_cases/10_empty_segments.gpx"));
    // Only the non-empty segment produces a Feature
    assert_eq!(fc.features.len(), 1);

    let geom = fc.features[0].geometry.as_ref().unwrap();
    if let Value::LineString(coords) = &geom.value {
        assert_eq!(coords.len(), 2);
    } else {
        panic!("Expected LineString");
    }
}

#[test]
fn test_11_cdata_and_entities() {
    let fc = convert(&load_fixture("edge_cases/11_cdata_and_entities.gpx"));
    assert_eq!(fc.features.len(), 1);

    let props = fc.features[0].properties.as_ref().unwrap();
    assert_eq!(props["name"], "Café & Bar <Tokyo>");
    assert_eq!(props["desc"], "Special chars: & < > \" '");
    assert_eq!(props["cmt"], "日本語テスト: 東京タワー");
}

#[test]
fn test_12_no_namespace() {
    let fc = convert(&load_fixture("edge_cases/12_no_namespace.gpx"));
    assert_eq!(fc.features.len(), 2); // 1 waypoint + 1 track

    let types: Vec<&str> = fc
        .features
        .iter()
        .map(|f| {
            f.properties.as_ref().unwrap()["gpxType"]
                .as_str()
                .unwrap()
        })
        .collect();
    assert_eq!(types, vec!["waypoint", "track"]);
}

#[test]
fn test_13_gpx10() {
    let fc = convert(&load_fixture("edge_cases/13_gpx10.gpx"));
    // Should parse GPX 1.0 elements, skipping speed/course/url
    assert_eq!(fc.features.len(), 2); // 1 waypoint + 1 track

    let wpt_props = fc.features[0].properties.as_ref().unwrap();
    assert_eq!(wpt_props["name"], "Legacy Point");

    let trk = &fc.features[1];
    let geom = trk.geometry.as_ref().unwrap();
    if let Value::LineString(coords) = &geom.value {
        assert_eq!(coords.len(), 2);
    } else {
        panic!("Expected LineString");
    }
}

// ---- vendor/ ----

#[test]
fn test_14_garmin_extensions() {
    let fc = convert(&load_fixture("vendor/14_garmin_extensions.gpx"));
    assert_eq!(fc.features.len(), 1);

    let f = &fc.features[0];
    let props = f.properties.as_ref().unwrap();
    assert_eq!(props["gpxType"], "track");
    assert_eq!(props["name"], "Garmin Activity");
    assert_eq!(props["type"], "running");

    // Extensions are skipped but parsing succeeds
    let geom = f.geometry.as_ref().unwrap();
    if let Value::LineString(coords) = &geom.value {
        assert_eq!(coords.len(), 3);
    } else {
        panic!("Expected LineString");
    }

    // Times should be present
    let coord_props = props["coordinateProperties"].as_object().unwrap();
    let times = coord_props["times"].as_array().unwrap();
    assert_eq!(times.len(), 3);
}
