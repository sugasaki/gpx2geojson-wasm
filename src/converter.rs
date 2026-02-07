use geojson::{Feature, FeatureCollection, Geometry, Value};
use serde_json::{Map, Value as JsonValue};

use crate::gpx_types::*;
use crate::options::{ConvertOptions, GpxElementType};

/// Convert parsed GPX data to a GeoJSON FeatureCollection.
pub fn to_feature_collection(data: &GpxData, opts: &ConvertOptions) -> FeatureCollection {
    let mut features = Vec::new();

    if opts.should_include(GpxElementType::Waypoint) {
        for wpt in &data.waypoints {
            features.push(waypoint_to_feature(wpt, opts));
        }
    }

    if opts.should_include(GpxElementType::Route) {
        for rte in &data.routes {
            if rte.points.len() >= 2 {
                features.push(route_to_feature(rte, opts));
            } else if rte.points.len() == 1 {
                features.push(single_point_feature(&rte.points[0], "route", opts));
            }
        }
    }

    if opts.should_include(GpxElementType::Track) {
        for trk in &data.tracks {
            features.extend(track_to_features(trk, opts));
        }
    }

    FeatureCollection {
        bbox: None,
        features,
        foreign_members: None,
    }
}

fn waypoint_to_feature(pt: &GpxPoint, opts: &ConvertOptions) -> Feature {
    let coords = point_coords(pt, opts.include_elevation);
    let geometry = Geometry::new(Value::Point(coords));

    let mut props = Map::new();
    props.insert(
        "gpxType".to_string(),
        JsonValue::String("waypoint".to_string()),
    );

    if opts.include_metadata {
        insert_point_metadata(&mut props, pt);
    }

    Feature {
        bbox: None,
        geometry: Some(geometry),
        id: None,
        properties: Some(props),
        foreign_members: None,
    }
}

fn route_to_feature(rte: &GpxRoute, opts: &ConvertOptions) -> Feature {
    let coords: Vec<Vec<f64>> = rte
        .points
        .iter()
        .map(|pt| point_coords(pt, opts.include_elevation))
        .collect();

    let geometry = Geometry::new(Value::LineString(coords));

    let mut props = Map::new();
    props.insert(
        "gpxType".to_string(),
        JsonValue::String("route".to_string()),
    );

    if opts.include_metadata {
        insert_optional(&mut props, "name", &rte.name);
        insert_optional(&mut props, "cmt", &rte.cmt);
        insert_optional(&mut props, "desc", &rte.desc);
        insert_optional(&mut props, "src", &rte.src);
        insert_optional(&mut props, "type", &rte.route_type);
        if let Some(n) = rte.number {
            props.insert("number".to_string(), JsonValue::Number(n.into()));
        }
        insert_link(&mut props, &rte.link);
    }

    if opts.include_time {
        insert_coordinate_times(&mut props, &rte.points);
    }

    Feature {
        bbox: None,
        geometry: Some(geometry),
        id: None,
        properties: Some(props),
        foreign_members: None,
    }
}

fn track_to_features(trk: &GpxTrack, opts: &ConvertOptions) -> Vec<Feature> {
    let non_empty_segments: Vec<&GpxSegment> =
        trk.segments.iter().filter(|s| !s.points.is_empty()).collect();

    if non_empty_segments.is_empty() {
        return Vec::new();
    }

    // Single point across all segments → Point Feature
    let total_points: usize = non_empty_segments.iter().map(|s| s.points.len()).sum();
    if total_points == 1 {
        let pt = &non_empty_segments[0].points[0];
        return vec![single_point_feature(pt, "track", opts)];
    }

    if opts.join_track_segments || non_empty_segments.len() == 1 {
        // Single feature: LineString (1 segment) or MultiLineString (multiple)
        if non_empty_segments.len() == 1 && non_empty_segments[0].points.len() >= 2 {
            let seg = non_empty_segments[0];
            let coords: Vec<Vec<f64>> = seg
                .points
                .iter()
                .map(|pt| point_coords(pt, opts.include_elevation))
                .collect();

            let geometry = Geometry::new(Value::LineString(coords));
            let mut props = build_track_props(trk, opts);

            if opts.include_time {
                insert_coordinate_times(&mut props, &seg.points);
            }

            return vec![Feature {
                bbox: None,
                geometry: Some(geometry),
                id: None,
                properties: Some(props),
                foreign_members: None,
            }];
        }

        // MultiLineString
        let line_strings: Vec<Vec<Vec<f64>>> = non_empty_segments
            .iter()
            .filter(|s| s.points.len() >= 2)
            .map(|seg| {
                seg.points
                    .iter()
                    .map(|pt| point_coords(pt, opts.include_elevation))
                    .collect()
            })
            .collect();

        if line_strings.is_empty() {
            return Vec::new();
        }

        let geometry = Geometry::new(Value::MultiLineString(line_strings));
        let mut props = build_track_props(trk, opts);

        if opts.include_time {
            let all_times: Vec<Vec<JsonValue>> = non_empty_segments
                .iter()
                .filter(|s| s.points.len() >= 2)
                .map(|seg| {
                    seg.points
                        .iter()
                        .map(|pt| match &pt.time {
                            Some(t) => JsonValue::String(t.clone()),
                            None => JsonValue::Null,
                        })
                        .collect()
                })
                .collect();
            if all_times.iter().any(|times| times.iter().any(|t| !t.is_null())) {
                let mut coord_props = Map::new();
                coord_props.insert("times".to_string(), JsonValue::Array(
                    all_times.into_iter().map(JsonValue::Array).collect(),
                ));
                props.insert(
                    "coordinateProperties".to_string(),
                    JsonValue::Object(coord_props),
                );
            }
        }

        vec![Feature {
            bbox: None,
            geometry: Some(geometry),
            id: None,
            properties: Some(props),
            foreign_members: None,
        }]
    } else {
        // Each segment as a separate Feature
        non_empty_segments
            .iter()
            .filter(|seg| seg.points.len() >= 2)
            .map(|seg| {
                let coords: Vec<Vec<f64>> = seg
                    .points
                    .iter()
                    .map(|pt| point_coords(pt, opts.include_elevation))
                    .collect();

                let geometry = Geometry::new(Value::LineString(coords));
                let mut props = build_track_props(trk, opts);

                if opts.include_time {
                    insert_coordinate_times(&mut props, &seg.points);
                }

                Feature {
                    bbox: None,
                    geometry: Some(geometry),
                    id: None,
                    properties: Some(props),
                    foreign_members: None,
                }
            })
            .collect()
    }
}

fn single_point_feature(pt: &GpxPoint, gpx_type: &str, opts: &ConvertOptions) -> Feature {
    let coords = point_coords(pt, opts.include_elevation);
    let geometry = Geometry::new(Value::Point(coords));

    let mut props = Map::new();
    props.insert(
        "gpxType".to_string(),
        JsonValue::String(gpx_type.to_string()),
    );

    if opts.include_metadata {
        insert_point_metadata(&mut props, pt);
    }

    Feature {
        bbox: None,
        geometry: Some(geometry),
        id: None,
        properties: Some(props),
        foreign_members: None,
    }
}

fn build_track_props(trk: &GpxTrack, opts: &ConvertOptions) -> Map<String, JsonValue> {
    let mut props = Map::new();
    props.insert(
        "gpxType".to_string(),
        JsonValue::String("track".to_string()),
    );

    if opts.include_metadata {
        insert_optional(&mut props, "name", &trk.name);
        insert_optional(&mut props, "cmt", &trk.cmt);
        insert_optional(&mut props, "desc", &trk.desc);
        insert_optional(&mut props, "src", &trk.src);
        insert_optional(&mut props, "type", &trk.track_type);
        if let Some(n) = trk.number {
            props.insert("number".to_string(), JsonValue::Number(n.into()));
        }
        insert_link(&mut props, &trk.link);
    }

    props
}

/// Build [lon, lat] or [lon, lat, ele] coordinate array.
fn point_coords(pt: &GpxPoint, include_elevation: bool) -> Vec<f64> {
    match (include_elevation, pt.ele) {
        (true, Some(ele)) => vec![pt.lon, pt.lat, ele],
        _ => vec![pt.lon, pt.lat],
    }
}

fn insert_point_metadata(props: &mut Map<String, JsonValue>, pt: &GpxPoint) {
    insert_optional(props, "name", &pt.name);
    insert_optional(props, "cmt", &pt.cmt);
    insert_optional(props, "desc", &pt.desc);
    insert_optional(props, "src", &pt.src);
    insert_optional(props, "sym", &pt.sym);
    insert_optional(props, "type", &pt.point_type);
    if let Some(ele) = pt.ele {
        props.insert(
            "ele".to_string(),
            JsonValue::Number(serde_json::Number::from_f64(ele).unwrap_or(0.into())),
        );
    }
    if let Some(ref time) = pt.time {
        props.insert("time".to_string(), JsonValue::String(time.clone()));
    }
    insert_link(props, &pt.link);
}

fn insert_optional(props: &mut Map<String, JsonValue>, key: &str, value: &Option<String>) {
    if let Some(v) = value {
        props.insert(key.to_string(), JsonValue::String(v.clone()));
    }
}

fn insert_link(props: &mut Map<String, JsonValue>, link: &Option<GpxLink>) {
    if let Some(link) = link {
        let mut link_obj = Map::new();
        link_obj.insert("href".to_string(), JsonValue::String(link.href.clone()));
        if let Some(ref t) = link.text {
            link_obj.insert("text".to_string(), JsonValue::String(t.clone()));
        }
        if let Some(ref lt) = link.link_type {
            link_obj.insert("type".to_string(), JsonValue::String(lt.clone()));
        }
        props.insert("link".to_string(), JsonValue::Object(link_obj));
    }
}

fn insert_coordinate_times(props: &mut Map<String, JsonValue>, points: &[GpxPoint]) {
    let times: Vec<JsonValue> = points
        .iter()
        .map(|pt| match &pt.time {
            Some(t) => JsonValue::String(t.clone()),
            None => JsonValue::Null,
        })
        .collect();

    // Only include if at least one time is present
    if times.iter().any(|t| !t.is_null()) {
        let mut coord_props = Map::new();
        coord_props.insert("times".to_string(), JsonValue::Array(times));
        props.insert(
            "coordinateProperties".to_string(),
            JsonValue::Object(coord_props),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_gpx;

    #[test]
    fn test_waypoint_conversion() {
        let xml = r#"<?xml version="1.0"?>
<gpx version="1.1">
  <wpt lat="35.6762" lon="139.6503">
    <ele>40.5</ele>
    <name>Tokyo</name>
  </wpt>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        let fc = to_feature_collection(&data, &ConvertOptions::default());

        assert_eq!(fc.features.len(), 1);
        let f = &fc.features[0];
        let geom = f.geometry.as_ref().unwrap();

        // Check [lon, lat, ele] order
        if let Value::Point(coords) = &geom.value {
            assert!((coords[0] - 139.6503).abs() < 1e-10); // lon
            assert!((coords[1] - 35.6762).abs() < 1e-10); // lat
            assert!((coords[2] - 40.5).abs() < 1e-10); // ele
        } else {
            panic!("Expected Point geometry");
        }

        let props = f.properties.as_ref().unwrap();
        assert_eq!(props["gpxType"], "waypoint");
        assert_eq!(props["name"], "Tokyo");
        assert_eq!(props["ele"], 40.5);
    }

    #[test]
    fn test_track_with_times() {
        let xml = r#"<?xml version="1.0"?>
<gpx version="1.1">
  <trk>
    <name>Run</name>
    <trkseg>
      <trkpt lat="35.0" lon="139.0"><time>2025-01-01T00:00:00Z</time></trkpt>
      <trkpt lat="35.001" lon="139.001"><time>2025-01-01T00:01:00Z</time></trkpt>
    </trkseg>
  </trk>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        let fc = to_feature_collection(&data, &ConvertOptions::default());

        assert_eq!(fc.features.len(), 1);
        let props = fc.features[0].properties.as_ref().unwrap();
        assert_eq!(props["gpxType"], "track");
        assert_eq!(props["name"], "Run");

        let coord_props = props["coordinateProperties"].as_object().unwrap();
        let times = coord_props["times"].as_array().unwrap();
        assert_eq!(times.len(), 2);
        assert_eq!(times[0], "2025-01-01T00:00:00Z");
    }

    #[test]
    fn test_multi_segment_join() {
        let xml = r#"<?xml version="1.0"?>
<gpx version="1.1">
  <trk>
    <trkseg>
      <trkpt lat="35.0" lon="139.0"/>
      <trkpt lat="35.001" lon="139.001"/>
    </trkseg>
    <trkseg>
      <trkpt lat="36.0" lon="140.0"/>
      <trkpt lat="36.001" lon="140.001"/>
    </trkseg>
  </trk>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        let opts = ConvertOptions {
            join_track_segments: true,
            ..Default::default()
        };
        let fc = to_feature_collection(&data, &opts);

        assert_eq!(fc.features.len(), 1);
        let geom = fc.features[0].geometry.as_ref().unwrap();
        match &geom.value {
            Value::MultiLineString(lines) => {
                assert_eq!(lines.len(), 2);
            }
            _ => panic!("Expected MultiLineString"),
        }
    }

    #[test]
    fn test_multi_segment_separate() {
        let xml = r#"<?xml version="1.0"?>
<gpx version="1.1">
  <trk>
    <name>Trail</name>
    <trkseg>
      <trkpt lat="35.0" lon="139.0"/>
      <trkpt lat="35.001" lon="139.001"/>
    </trkseg>
    <trkseg>
      <trkpt lat="36.0" lon="140.0"/>
      <trkpt lat="36.001" lon="140.001"/>
    </trkseg>
  </trk>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        let fc = to_feature_collection(&data, &ConvertOptions::default());

        // Each segment is a separate Feature
        assert_eq!(fc.features.len(), 2);
        for f in &fc.features {
            let props = f.properties.as_ref().unwrap();
            assert_eq!(props["gpxType"], "track");
            assert_eq!(props["name"], "Trail");
        }
    }

    #[test]
    fn test_single_point_track() {
        let xml = r#"<?xml version="1.0"?>
<gpx version="1.1">
  <trk>
    <name>Single</name>
    <trkseg>
      <trkpt lat="35.0" lon="139.0"/>
    </trkseg>
  </trk>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        let fc = to_feature_collection(&data, &ConvertOptions::default());

        assert_eq!(fc.features.len(), 1);
        let geom = fc.features[0].geometry.as_ref().unwrap();
        match &geom.value {
            Value::Point(_) => {} // Expected: 1 point → Point Feature
            _ => panic!("Expected Point geometry for single-point track"),
        }
    }

    #[test]
    fn test_empty_gpx_conversion() {
        let xml = r#"<?xml version="1.0"?><gpx version="1.1"></gpx>"#;
        let data = parse_gpx(xml).unwrap();
        let fc = to_feature_collection(&data, &ConvertOptions::default());
        assert!(fc.features.is_empty());
    }

    #[test]
    fn test_no_elevation() {
        let xml = r#"<?xml version="1.0"?>
<gpx version="1.1">
  <wpt lat="35.0" lon="139.0"><ele>100.0</ele></wpt>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        let opts = ConvertOptions {
            include_elevation: false,
            ..Default::default()
        };
        let fc = to_feature_collection(&data, &opts);

        let geom = fc.features[0].geometry.as_ref().unwrap();
        if let Value::Point(coords) = &geom.value {
            assert_eq!(coords.len(), 2); // No elevation
        }
    }

    #[test]
    fn test_type_filter() {
        let xml = r#"<?xml version="1.0"?>
<gpx version="1.1">
  <wpt lat="35.0" lon="139.0"/>
  <rte><rtept lat="35.0" lon="139.0"/><rtept lat="36.0" lon="140.0"/></rte>
  <trk><trkseg><trkpt lat="35.0" lon="139.0"/><trkpt lat="36.0" lon="140.0"/></trkseg></trk>
</gpx>"#;
        let data = parse_gpx(xml).unwrap();
        let opts = ConvertOptions {
            types: Some(vec![GpxElementType::Waypoint]),
            ..Default::default()
        };
        let fc = to_feature_collection(&data, &opts);

        assert_eq!(fc.features.len(), 1);
        let props = fc.features[0].properties.as_ref().unwrap();
        assert_eq!(props["gpxType"], "waypoint");
    }
}
