import React, { useEffect, useRef, useCallback } from "react";
import maplibregl from "maplibre-gl";
import "maplibre-gl/dist/maplibre-gl.css";
import type { ProxyNodeInfo } from "../types";
import type { FeatureCollection, Feature, Point, LineString } from "geojson";
import styles from "./WorldMap.module.css";

interface Props {
  nodes: ProxyNodeInfo[];
  onNodeClick?: (node: ProxyNodeInfo) => void;
}

const COUNTRY_COORDS: Record<string, [number, number]> = {
  US: [-98.5795, 39.8283], CA: [-106.3468, 56.1304], GB: [-3.4359, 55.3781],
  DE: [10.4515, 51.1657], FR: [2.2137, 46.6034], NL: [5.2913, 52.1326],
  JP: [138.2529, 36.2048], KR: [127.7669, 35.9078], SG: [103.8198, 1.3521],
  HK: [114.1694, 22.3193], TW: [120.9605, 23.6978], AU: [133.7751, -25.2744],
  BR: [-51.9253, -14.2350], RU: [105.3188, 61.5240], IN: [78.9629, 20.5937],
  CN: [104.1954, 35.8617], TH: [100.9925, 15.8700], VN: [108.2772, 14.0583],
  ID: [113.9213, -0.7893], PH: [121.7740, 12.8797], MY: [101.9758, 4.2105],
  IT: [12.5674, 41.8719], ES: [-3.7492, 40.4637], SE: [18.6435, 60.1282],
  NO: [8.4689, 60.4720], FI: [26.2726, 61.9241], PL: [19.1451, 51.9194],
  CH: [8.2275, 46.8182], AT: [14.5501, 47.5162], BE: [4.4699, 50.5039],
  DK: [9.5018, 56.2639], IE: [-8.2439, 53.4129], PT: [-8.2245, 39.3999],
  GR: [21.8243, 39.0742], CZ: [15.4730, 49.8175], HU: [19.5033, 47.1625],
  RO: [24.9668, 45.9432], UA: [31.1656, 48.3794], TR: [35.2433, 38.9637],
  IL: [34.8516, 31.0461], AE: [53.8478, 23.4241], ZA: [22.9375, -30.5595],
  NG: [8.6753, 9.0820], EG: [30.8025, 26.8206], KE: [37.9062, -0.0236],
  AR: [-63.5886, -38.4161], CL: [-71.5430, -35.6751], CO: [-74.2973, 4.5709],
  MX: [-102.5528, 23.6345], NZ: [172.8344, -40.9006],
};

const CENTER: [number, number] = [20, 20];

function nodeColor(node: ProxyNodeInfo): string {
  if (!node.healthy) return "#888";
  if (node.latency_ms == null) return "#f0ad4e";
  if (node.latency_ms < 200) return "#22c55e";
  if (node.latency_ms < 500) return "#f0ad4e";
  return "#ef4444";
}

function nodeRadius(score: number): number {
  if (score > 0.8) return 10;
  if (score > 0.4) return 7;
  return 5;
}

const WorldMap: React.FC<Props> = ({ nodes, onNodeClick }) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const mapRef = useRef<maplibregl.Map | null>(null);
  const popupRef = useRef<maplibregl.Popup | null>(null);

  const makeNodesGeoJson = useCallback((): FeatureCollection => ({
    type: "FeatureCollection",
    features: nodes.filter(n => n.geo_tag && COUNTRY_COORDS[n.geo_tag]).map(n => ({
      type: "Feature",
      geometry: { type: "Point", coordinates: COUNTRY_COORDS[n.geo_tag!] },
      properties: {
        id: n.url,
        tag: n.tag || n.url.slice(0, 40),
        country: n.geo_tag,
        latency: n.latency_ms,
        healthy: n.healthy,
        score: n.score,
        color: nodeColor(n),
        radius: nodeRadius(n.score),
        speed: n.speed_tier,
        success: n.success_count,
        fail: n.fail_count,
      },
    })),
  }), [nodes]);

  const makeLinesGeoJson = useCallback((): FeatureCollection => ({
    type: "FeatureCollection",
    features: nodes
      .filter(n => n.healthy && n.geo_tag && COUNTRY_COORDS[n.geo_tag])
      .map(n => ({
        type: "Feature",
        geometry: {
          type: "LineString",
          coordinates: [CENTER, COUNTRY_COORDS[n.geo_tag!]],
        },
        properties: {},
      })),
  }), [nodes]);

  useEffect(() => {
    if (!containerRef.current) return;

    const map = new maplibregl.Map({
      container: containerRef.current,
      style: {
        version: 8,
        sources: {
          osm: {
            type: "raster",
            tiles: ["https://tile.openstreetmap.org/{z}/{x}/{y}.png"],
            tileSize: 256,
            attribution: "&copy; OpenStreetMap contributors",
          },
        },
        layers: [{
          id: "osm",
          type: "raster",
          source: "osm",
        }],
      },
      center: CENTER,
      zoom: 2,
      attributionControl: {},
    });

    map.addControl(new maplibregl.NavigationControl(), "top-right");

    map.on("load", () => {
      map.addSource("nodes", {
        type: "geojson",
        data: makeNodesGeoJson(),
        cluster: false,
      });

      map.addSource("lines", {
        type: "geojson",
        data: makeLinesGeoJson(),
      });

      map.addLayer({
        id: "lines-layer",
        type: "line",
        source: "lines",
        paint: {
          "line-color": "#6366f1",
          "line-width": 1,
          "line-opacity": 0.2,
        },
      });

      map.addLayer({
        id: "nodes-layer",
        type: "circle",
        source: "nodes",
        paint: {
          "circle-color": ["get", "color"],
          "circle-radius": ["get", "radius"],
          "circle-stroke-color": "#fff",
          "circle-stroke-width": 1.5,
          "circle-opacity": 0.9,
        },
      });

      map.on("click", "nodes-layer", (e) => {
        if (!e.features?.[0]?.properties) return;
        const props = e.features[0].properties;
        const node = nodes.find(n => n.url === props.id);
        if (node) onNodeClick?.(node);

        if (popupRef.current) popupRef.current.remove();
        const coords = (e.features[0].geometry as Point).coordinates.slice() as [number, number];
        popupRef.current = new maplibregl.Popup({ closeButton: true, closeOnClick: true })
          .setLngLat(coords)
          .setHTML(`
            <div style="font-size:12px;line-height:1.5">
              <b>${props.tag}</b><br/>
              ${props.country} | ${props.latency != null ? props.latency.toFixed(0) + "ms" : "—"}<br/>
              Score: ${props.score.toFixed(2)} | ${props.speed}<br/>
              ✓${props.success} ✗${props.fail}
            </div>
          `)
          .addTo(map);
      });

      map.on("mouseenter", "nodes-layer", () => {
        map.getCanvas().style.cursor = "pointer";
      });
      map.on("mouseleave", "nodes-layer", () => {
        map.getCanvas().style.cursor = "";
      });
    });

    mapRef.current = map;

    return () => {
      map.remove();
      mapRef.current = null;
    };
  }, []);

  useEffect(() => {
    const map = mapRef.current;
    if (!map || !map.isStyleLoaded()) return;
    const nodesSource = map.getSource("nodes") as maplibregl.GeoJSONSource | undefined;
    if (nodesSource) nodesSource.setData(makeNodesGeoJson());
    const linesSource = map.getSource("lines") as maplibregl.GeoJSONSource | undefined;
    if (linesSource) linesSource.setData(makeLinesGeoJson());
  }, [nodes, makeNodesGeoJson, makeLinesGeoJson]);

  return (
    <div className={styles.container}>
      <div ref={containerRef} className={styles.map} />
      <div className={styles.legend}>
        <span><span className={styles.dot} style={{ background: "#22c55e" }} /> Fast (&lt;200ms)</span>
        <span><span className={styles.dot} style={{ background: "#f0ad4e" }} /> Medium (&lt;500ms)</span>
        <span><span className={styles.dot} style={{ background: "#ef4444" }} /> Slow</span>
        <span><span className={styles.dot} style={{ background: "#888" }} /> Offline</span>
      </div>
    </div>
  );
};

export default WorldMap;
