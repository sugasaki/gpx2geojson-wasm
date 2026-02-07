import type { FeatureCollection } from "geojson";
import type { ConvertOptions } from "./types.js";
export type { ConvertOptions, GpxElementType } from "./types.js";
export type { FeatureCollection } from "geojson";

import initWasm, {
  gpxToGeoJson as rawGpxToGeoJson,
  gpxToGeoJsonString as rawGpxToGeoJsonString,
} from "../pkg/gpx2geojson_wasm.js";

let initPromise: Promise<void> | null = null;

function ensureInit(): Promise<void> {
  if (!initPromise) {
    initPromise = initWasm().then(() => {});
  }
  return initPromise;
}

export async function gpxToGeoJson(
  gpxString: string,
  options?: ConvertOptions
): Promise<FeatureCollection> {
  await ensureInit();
  return rawGpxToGeoJson(gpxString, options ?? undefined) as FeatureCollection;
}

export async function gpxToGeoJsonString(
  gpxString: string,
  options?: ConvertOptions
): Promise<string> {
  await ensureInit();
  return rawGpxToGeoJsonString(gpxString, options ?? undefined);
}
