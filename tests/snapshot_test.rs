use gpx2geojson_wasm::converter::to_feature_collection;
use gpx2geojson_wasm::options::ConvertOptions;
use gpx2geojson_wasm::parser::parse_gpx;
use std::path::Path;

fn load_fixture(path: &str) -> String {
    std::fs::read_to_string(format!("tests/fixtures/{path}")).unwrap()
}

fn convert(gpx: &str) -> serde_json::Value {
    let data = parse_gpx(gpx).unwrap();
    let fc = to_feature_collection(&data, &ConvertOptions::default());
    serde_json::to_value(&fc).unwrap()
}

fn convert_with_opts(gpx: &str, opts: &ConvertOptions) -> serde_json::Value {
    let data = parse_gpx(gpx).unwrap();
    let fc = to_feature_collection(&data, opts);
    serde_json::to_value(&fc).unwrap()
}

/// Compare actual GeoJSON output against the expected snapshot file.
/// When `UPDATE_SNAPSHOTS=1` is set, write/overwrite the expected file instead.
fn assert_snapshot(actual: &serde_json::Value, expected_path: &str) {
    let path = format!("tests/fixtures/expected/{expected_path}");

    if matches!(std::env::var("UPDATE_SNAPSHOTS").as_deref(), Ok("1")) {
        let dir = Path::new(&path).parent().unwrap();
        std::fs::create_dir_all(dir).unwrap();
        let pretty = serde_json::to_string_pretty(actual).unwrap();
        std::fs::write(&path, pretty.as_bytes()).unwrap();
        eprintln!("Updated snapshot: {path}");
        return;
    }

    let expected_str = std::fs::read_to_string(&path)
        .unwrap_or_else(|_| panic!("Expected file not found: {path}. Run with UPDATE_SNAPSHOTS=1 to generate."));
    let expected: serde_json::Value = serde_json::from_str(&expected_str)
        .unwrap_or_else(|e| panic!("Failed to parse {path}: {e}"));

    assert_eq!(
        *actual, expected,
        "Snapshot mismatch for {path}.\nRun with UPDATE_SNAPSHOTS=1 to update."
    );
}

/// Convert a fixture with default options and compare against the expected snapshot.
fn assert_snapshot_default(fixture: &str, expected: &str) {
    let gpx = load_fixture(fixture);
    let actual = convert(&gpx);
    assert_snapshot(&actual, expected);
}

// ---- basic/ ----

#[test]
fn snapshot_01_minimal_waypoint() {
    assert_snapshot_default(
        "basic/01_minimal_waypoint.gpx",
        "basic/01_minimal_waypoint.geojson",
    );
}

#[test]
fn snapshot_02_full_waypoint() {
    assert_snapshot_default(
        "basic/02_full_waypoint.gpx",
        "basic/02_full_waypoint.geojson",
    );
}

#[test]
fn snapshot_03_simple_route() {
    assert_snapshot_default(
        "basic/03_simple_route.gpx",
        "basic/03_simple_route.geojson",
    );
}

#[test]
fn snapshot_04_simple_track() {
    assert_snapshot_default(
        "basic/04_simple_track.gpx",
        "basic/04_simple_track.geojson",
    );
}

#[test]
fn snapshot_05_complete() {
    assert_snapshot_default(
        "basic/05_complete.gpx",
        "basic/05_complete.geojson",
    );
}

// ---- tracks/ ----

#[test]
fn snapshot_06_multi_segment_separate() {
    assert_snapshot_default(
        "tracks/06_multi_segment.gpx",
        "tracks/06_multi_segment.geojson",
    );
}

#[test]
fn snapshot_06_multi_segment_joined() {
    let gpx = load_fixture("tracks/06_multi_segment.gpx");
    let opts = ConvertOptions {
        join_track_segments: true,
        ..Default::default()
    };
    let actual = convert_with_opts(&gpx, &opts);
    assert_snapshot(&actual, "tracks/06_multi_segment_joined.geojson");
}

#[test]
fn snapshot_07_multi_track() {
    assert_snapshot_default(
        "tracks/07_multi_track.gpx",
        "tracks/07_multi_track.geojson",
    );
}

#[test]
fn snapshot_08_single_point_track() {
    assert_snapshot_default(
        "tracks/08_single_point_track.gpx",
        "tracks/08_single_point_track.geojson",
    );
}

// ---- edge_cases/ ----

#[test]
fn snapshot_09_empty() {
    assert_snapshot_default(
        "edge_cases/09_empty.gpx",
        "edge_cases/09_empty.geojson",
    );
}

#[test]
fn snapshot_10_empty_segments() {
    assert_snapshot_default(
        "edge_cases/10_empty_segments.gpx",
        "edge_cases/10_empty_segments.geojson",
    );
}

#[test]
fn snapshot_11_cdata_and_entities() {
    assert_snapshot_default(
        "edge_cases/11_cdata_and_entities.gpx",
        "edge_cases/11_cdata_and_entities.geojson",
    );
}

#[test]
fn snapshot_12_no_namespace() {
    assert_snapshot_default(
        "edge_cases/12_no_namespace.gpx",
        "edge_cases/12_no_namespace.geojson",
    );
}

#[test]
fn snapshot_13_gpx10() {
    assert_snapshot_default(
        "edge_cases/13_gpx10.gpx",
        "edge_cases/13_gpx10.geojson",
    );
}

// ---- vendor/ ----

#[test]
fn snapshot_14_garmin_extensions() {
    assert_snapshot_default(
        "vendor/14_garmin_extensions.gpx",
        "vendor/14_garmin_extensions.geojson",
    );
}
