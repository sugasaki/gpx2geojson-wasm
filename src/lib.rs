mod converter;
mod error;
mod gpx_types;
mod options;
mod parser;

use wasm_bindgen::prelude::*;

use crate::error::Gpx2GeoJsonError;
use crate::options::ConvertOptions;

/// Convert GPX string to GeoJSON, returned as a JS object.
#[wasm_bindgen(js_name = gpxToGeoJson)]
pub fn gpx_to_geojson(gpx_string: &str, options: JsValue) -> Result<JsValue, JsValue> {
    console_error_panic_hook::set_once();

    let opts = parse_options(options)?;
    let gpx_data = parser::parse_gpx(gpx_string).map_err(Gpx2GeoJsonError::from)?;
    let fc = converter::to_feature_collection(&gpx_data, &opts);
    serde_wasm_bindgen::to_value(&fc).map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Convert GPX string to GeoJSON, returned as a JSON string.
#[wasm_bindgen(js_name = gpxToGeoJsonString)]
pub fn gpx_to_geojson_string(gpx_string: &str, options: JsValue) -> Result<String, JsValue> {
    console_error_panic_hook::set_once();

    let opts = parse_options(options)?;
    let gpx_data = parser::parse_gpx(gpx_string).map_err(Gpx2GeoJsonError::from)?;
    let fc = converter::to_feature_collection(&gpx_data, &opts);
    serde_json::to_string(&fc).map_err(|e| JsValue::from_str(&e.to_string()))
}

fn parse_options(options: JsValue) -> Result<ConvertOptions, JsValue> {
    if options.is_undefined() || options.is_null() {
        Ok(ConvertOptions::default())
    } else {
        serde_wasm_bindgen::from_value(options).map_err(|e| JsValue::from_str(&e.to_string()))
    }
}
