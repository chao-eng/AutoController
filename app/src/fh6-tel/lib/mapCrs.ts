import type * as Leaflet from 'leaflet';

// L.CRS.Simple inverts the Y axis (transformation 1,0,-1,0 → north is up),
// but standard XYZ tile pyramids (e.g. gdal2tiles) are top-left origin
// with Y increasing downward. Override the transformation so tile rows are
// requested in the right order. project()/unproject() use this same
// transformation, so calibration pixel math stays internally consistent.
export function xyzSimpleCRS(L: typeof Leaflet): Leaflet.CRS {
  return L.Util.extend({}, L.CRS.Simple, {
    transformation: new L.Transformation(1, 0, 1, 0),
  }) as Leaflet.CRS;
}
