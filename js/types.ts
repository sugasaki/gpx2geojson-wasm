import type { FeatureCollection } from "geojson";

export type { FeatureCollection };

export type GpxElementType = "waypoint" | "route" | "track";

export interface ConvertOptions {
  includeElevation?: boolean;
  includeTime?: boolean;
  includeMetadata?: boolean;
  types?: GpxElementType[];
  joinTrackSegments?: boolean;
}
