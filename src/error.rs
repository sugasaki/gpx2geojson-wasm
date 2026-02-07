use std::num::ParseFloatError;
use wasm_bindgen::JsValue;

#[derive(Debug)]
pub enum Gpx2GeoJsonError {
    XmlParse(quick_xml::Error),
    MissingAttribute {
        element: &'static str,
        attribute: &'static str,
    },
    InvalidAttribute {
        element: &'static str,
        attribute: &'static str,
        value: String,
    },
    FloatParse(ParseFloatError),
}

impl std::fmt::Display for Gpx2GeoJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::XmlParse(e) => write!(f, "XML parse error: {e}"),
            Self::MissingAttribute { element, attribute } => {
                write!(f, "Missing attribute '{attribute}' on <{element}>")
            }
            Self::InvalidAttribute {
                element,
                attribute,
                value,
            } => write!(
                f,
                "Invalid value '{value}' for attribute '{attribute}' on <{element}>"
            ),
            Self::FloatParse(e) => write!(f, "Float parse error: {e}"),
        }
    }
}

impl std::error::Error for Gpx2GeoJsonError {}

impl From<quick_xml::Error> for Gpx2GeoJsonError {
    fn from(e: quick_xml::Error) -> Self {
        Self::XmlParse(e)
    }
}

impl From<ParseFloatError> for Gpx2GeoJsonError {
    fn from(e: ParseFloatError) -> Self {
        Self::FloatParse(e)
    }
}

impl From<Gpx2GeoJsonError> for JsValue {
    fn from(e: Gpx2GeoJsonError) -> Self {
        JsValue::from_str(&e.to_string())
    }
}
