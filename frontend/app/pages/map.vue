<script setup lang="ts">
import { ref, onMounted, type Ref, onUnmounted, h, render } from 'vue';
import {
  type GeoJSONSource,
  type MapMouseEvent,
  Popup,
  Map,
  Marker,
  LngLat,
} from 'maplibre-gl';
import 'maplibre-gl/dist/maplibre-gl.css';
import { Urgency } from '~/components/types/Urgency';
import MapMarkerPopup from '~/components/map/MapMarkerPopup.vue';

interface MapStyle {
  name: string;
  url: string;
}

interface CellIndex {
  row: number;
  col: number;
}

interface GridDefinition {
  westX: number;
  northY: number;
  rows: number;
  cols: number;
}

type GeoJsonFeature = Record<string, unknown>;
type GeoJsonCollection = Record<string, unknown>;

const EARTH_RADIUS = 6378137;
const BIG_CELL_METERS = 10_000;
const SMALL_CELL_METERS = 250;
const SMALL_CELLS_PER_BIG = BIG_CELL_METERS / SMALL_CELL_METERS;
const GERMANY_BBOX = {
  minLng: 5.866,
  maxLng: 15.041,
  minLat: 47.27,
  maxLat: 55.099,
};

const SOURCE_BIG_LINES = 'de-grid-big-lines';
const SOURCE_BIG_LABELS = 'de-grid-big-labels';
const SOURCE_BIG_SELECTED = 'de-grid-big-selected';
const SOURCE_SMALL_LINES = 'de-grid-small-lines';
const SOURCE_SMALL_LABELS = 'de-grid-small-labels';

const LAYER_BIG_LINES = 'de-grid-big-lines-layer';
const LAYER_BIG_LABELS = 'de-grid-big-labels-layer';
const LAYER_BIG_SELECTED = 'de-grid-big-selected-layer';
const LAYER_SMALL_LINES = 'de-grid-small-lines-layer';
const LAYER_SMALL_LABELS = 'de-grid-small-labels-layer';

const availableMapStyles: MapStyle[] = [
  { name: 'World Color', url: 'https://sgx.geodatenzentrum.de/gdz_basemapworld_vektor/styles/bm_web_wld_col.json' },
  { name: 'Germany Color, no terrain', url: 'https://sgx.geodatenzentrum.de/gdz_basemapde_vektor/styles/bm_web_col.json' },
  { name: 'Germany Color, with terrain', url: 'https://sgx.geodatenzentrum.de/gdz_basemapde_vektor/styles/bm_web_top.json' },
  { name: 'Germany Grayscale', url: 'https://sgx.geodatenzentrum.de/gdz_basemapde_vektor/styles/bm_web_gry.json' },
];

const selectedMapStyle: Ref<MapStyle | undefined> = ref(availableMapStyles[0]);
const mapContainer: Ref<HTMLElement | null> = ref(null);
const map: Ref<Map | null> = ref(null);

const selectedBigCell = ref<string>('None');
const selectedSmallCell = ref<string>('None');
const selectedFullCell = ref<string>('None');

const germanyGrid = buildGridDefinition(BIG_CELL_METERS);

function lngLatToWebMercator(lng: number, lat: number) {
  const x = EARTH_RADIUS * (lng * Math.PI / 180);
  const clampedLat = Math.max(-85.05112878, Math.min(85.05112878, lat));
  const y = EARTH_RADIUS * Math.log(Math.tan(Math.PI / 4 + (clampedLat * Math.PI / 360)));
  return { x, y };
}

function webMercatorToLngLat(x: number, y: number) {
  const lng = x / EARTH_RADIUS * (180 / Math.PI);
  const lat = (2 * Math.atan(Math.exp(y / EARTH_RADIUS)) - Math.PI / 2) * (180 / Math.PI);
  return [lng, lat];
}

function buildGridDefinition(cellSizeMeters: number): GridDefinition {
  const northWest = lngLatToWebMercator(GERMANY_BBOX.minLng, GERMANY_BBOX.maxLat);
  const southEast = lngLatToWebMercator(GERMANY_BBOX.maxLng, GERMANY_BBOX.minLat);
  const width = southEast.x - northWest.x;
  const height = northWest.y - southEast.y;

  return {
    westX: northWest.x,
    northY: northWest.y,
    cols: Math.ceil(width / cellSizeMeters),
    rows: Math.ceil(height / cellSizeMeters),
  };
}

function rowToLetters(index: number) {
  let value = index + 1;
  let label = '';

  while (value > 0) {
    const current = (value - 1) % 26;
    label = String.fromCharCode(65 + current) + label;
    value = Math.floor((value - 1) / 26);
  }

  return label;
}

function buildBigCellLabel(row: number, col: number) {
  const rowPart = rowToLetters(row);
  const colPart = String(col + 1).padStart(2, '0');
  return `${rowPart}${colPart}`;
}

function buildSmallCellLabel(row: number, col: number) {
  const rowPart = rowToLetters(row);
  const colPart = String(col + 1).padStart(2, '0');
  return `${rowPart}${colPart}`;
}

function getCellBounds(row: number, col: number, sizeMeters: number, grid: GridDefinition) {
  const xMin = grid.westX + col * sizeMeters;
  const xMax = xMin + sizeMeters;
  const yMax = grid.northY - row * sizeMeters;
  const yMin = yMax - sizeMeters;
  return { xMin, xMax, yMin, yMax };
}

function getBigCellByLngLat(lng: number, lat: number): CellIndex | null {
  const point = lngLatToWebMercator(lng, lat);
  const col = Math.floor((point.x - germanyGrid.westX) / BIG_CELL_METERS);
  const row = Math.floor((germanyGrid.northY - point.y) / BIG_CELL_METERS);

  if (row < 0 || col < 0 || row >= germanyGrid.rows || col >= germanyGrid.cols) {
    return null;
  }

  return { row, col };
}

function getSmallCellByLngLat(lng: number, lat: number, bigCell: CellIndex): CellIndex | null {
  const point = lngLatToWebMercator(lng, lat);
  const bounds = getCellBounds(bigCell.row, bigCell.col, BIG_CELL_METERS, germanyGrid);

  if (point.x < bounds.xMin || point.x >= bounds.xMax || point.y <= bounds.yMin || point.y > bounds.yMax) {
    return null;
  }

  const relativeX = point.x - bounds.xMin;
  const relativeY = bounds.yMax - point.y;
  const col = Math.floor(relativeX / SMALL_CELL_METERS);
  const row = Math.floor(relativeY / SMALL_CELL_METERS);

  if (row < 0 || col < 0 || row >= SMALL_CELLS_PER_BIG || col >= SMALL_CELLS_PER_BIG) {
    return null;
  }

  return { row, col };
}

function toPolygonFeature(ring: number[][], properties: Record<string, string | number | boolean | null>): GeoJsonFeature {
  return {
    type: 'Feature',
    geometry: {
      type: 'Polygon',
      coordinates: [ring],
    },
    properties,
  };
}

function toLineFeature(line: number[][], properties: Record<string, string | number | boolean | null>): GeoJsonFeature {
  return {
    type: 'Feature',
    geometry: {
      type: 'LineString',
      coordinates: line,
    },
    properties,
  };
}

function toPointFeature(point: number[], properties: Record<string, string | number | boolean | null>): GeoJsonFeature {
  return {
    type: 'Feature',
    geometry: {
      type: 'Point',
      coordinates: point,
    },
    properties,
  };
}

function buildBigGridLineFeatures() {
  const features: GeoJsonFeature[] = [];
  const yBottom = germanyGrid.northY - germanyGrid.rows * BIG_CELL_METERS;
  const xRight = germanyGrid.westX + germanyGrid.cols * BIG_CELL_METERS;

  for (let col = 0; col <= germanyGrid.cols; col++) {
    const x = germanyGrid.westX + col * BIG_CELL_METERS;
    features.push(
      toLineFeature(
        [
          webMercatorToLngLat(x, germanyGrid.northY),
          webMercatorToLngLat(x, yBottom),
        ],
        {},
      ),
    );
  }

  for (let row = 0; row <= germanyGrid.rows; row++) {
    const y = germanyGrid.northY - row * BIG_CELL_METERS;
    features.push(
      toLineFeature(
        [
          webMercatorToLngLat(germanyGrid.westX, y),
          webMercatorToLngLat(xRight, y),
        ],
        {},
      ),
    );
  }

  return features;
}

function buildBigGridLabelFeatures() {
  const features: GeoJsonFeature[] = [];

  for (let row = 0; row < germanyGrid.rows; row++) {
    for (let col = 0; col < germanyGrid.cols; col++) {
      const bounds = getCellBounds(row, col, BIG_CELL_METERS, germanyGrid);
      const centerX = (bounds.xMin + bounds.xMax) / 2;
      const centerY = (bounds.yMin + bounds.yMax) / 2;
      features.push(
        toPointFeature(
          webMercatorToLngLat(centerX, centerY),
          { label: buildBigCellLabel(row, col) },
        ),
      );
    }
  }

  return features;
}

function buildSelectedBigCellFeature(cell: CellIndex | null) {
  if (!cell) {
    return [] as GeoJsonFeature[];
  }

  const bounds = getCellBounds(cell.row, cell.col, BIG_CELL_METERS, germanyGrid);
  const northWest = webMercatorToLngLat(bounds.xMin, bounds.yMax);
  const northEast = webMercatorToLngLat(bounds.xMax, bounds.yMax);
  const southEast = webMercatorToLngLat(bounds.xMax, bounds.yMin);
  const southWest = webMercatorToLngLat(bounds.xMin, bounds.yMin);

  return [
    toPolygonFeature(
      [northWest, northEast, southEast, southWest, northWest],
      { selected: true },
    ),
  ];
}

function buildSmallGridForBigCell(cell: CellIndex | null) {
  if (!cell) {
    return {
      lineFeatures: [] as GeoJsonFeature[],
      labelFeatures: [] as GeoJsonFeature[],
    };
  }

  const lineFeatures: GeoJsonFeature[] = [];
  const labelFeatures: GeoJsonFeature[] = [];
  const bounds = getCellBounds(cell.row, cell.col, BIG_CELL_METERS, germanyGrid);

  for (let i = 0; i <= SMALL_CELLS_PER_BIG; i++) {
    const x = bounds.xMin + i * SMALL_CELL_METERS;
    lineFeatures.push(
      toLineFeature(
        [
          webMercatorToLngLat(x, bounds.yMax),
          webMercatorToLngLat(x, bounds.yMin),
        ],
        {},
      ),
    );
  }

  for (let i = 0; i <= SMALL_CELLS_PER_BIG; i++) {
    const y = bounds.yMax - i * SMALL_CELL_METERS;
    lineFeatures.push(
      toLineFeature(
        [
          webMercatorToLngLat(bounds.xMin, y),
          webMercatorToLngLat(bounds.xMax, y),
        ],
        {},
      ),
    );
  }

  for (let row = 0; row < SMALL_CELLS_PER_BIG; row++) {
    for (let col = 0; col < SMALL_CELLS_PER_BIG; col++) {
      const xCenter = bounds.xMin + (col + 0.5) * SMALL_CELL_METERS;
      const yCenter = bounds.yMax - (row + 0.5) * SMALL_CELL_METERS;
      labelFeatures.push(
        toPointFeature(
          webMercatorToLngLat(xCenter, yCenter),
          { label: buildSmallCellLabel(row, col) },
        ),
      );
    }
  }

  return { lineFeatures, labelFeatures };
}

function buildAllSmallGridLineFeatures() {
  const lineFeatures: GeoJsonFeature[] = [];
  const yBottom = germanyGrid.northY - germanyGrid.rows * BIG_CELL_METERS;
  const xRight = germanyGrid.westX + germanyGrid.cols * BIG_CELL_METERS;
  const totalSmallCols = germanyGrid.cols * SMALL_CELLS_PER_BIG;
  const totalSmallRows = germanyGrid.rows * SMALL_CELLS_PER_BIG;

  for (let col = 0; col <= totalSmallCols; col++) {
    const x = germanyGrid.westX + col * SMALL_CELL_METERS;
    lineFeatures.push(
      toLineFeature(
        [
          webMercatorToLngLat(x, germanyGrid.northY),
          webMercatorToLngLat(x, yBottom),
        ],
        {},
      ),
    );
  }

  for (let row = 0; row <= totalSmallRows; row++) {
    const y = germanyGrid.northY - row * SMALL_CELL_METERS;
    lineFeatures.push(
      toLineFeature(
        [
          webMercatorToLngLat(germanyGrid.westX, y),
          webMercatorToLngLat(xRight, y),
        ],
        {},
      ),
    );
  }

  return lineFeatures;
}

function asCollection(features: GeoJsonFeature[]): GeoJsonCollection {
  return {
    type: 'FeatureCollection',
    features,
  };
}

function setGeoJsonSource(sourceId: string, data: GeoJsonCollection) {
  if (!map.value) return;
  const source = map.value.getSource(sourceId) as GeoJSONSource | undefined;

  if (source) {
    source.setData(data as never);
    return;
  }

  map.value.addSource(sourceId, {
    type: 'geojson',
    data: data as never,
  });
}

function ensureGridLayers() {
  if (!map.value) return;

  if (!map.value.getLayer(LAYER_BIG_LINES)) {
    map.value.addLayer({
      id: LAYER_BIG_LINES,
      type: 'line',
      source: SOURCE_BIG_LINES,
      paint: {
        'line-color': '#000000',
        'line-width': 1,
      },
    });
  }

  if (!map.value.getLayer(LAYER_BIG_SELECTED)) {
    map.value.addLayer({
      id: LAYER_BIG_SELECTED,
      type: 'fill',
      source: SOURCE_BIG_SELECTED,
      paint: {
        'fill-color': '#4DEA4D',
        'fill-opacity': 0.18,
      },
    });
  }

  if (!map.value.getLayer(LAYER_SMALL_LINES)) {
    map.value.addLayer({
      id: LAYER_SMALL_LINES,
      type: 'line',
      source: SOURCE_SMALL_LINES,
      minzoom: 13,
      paint: {
        'line-color': '#000000',
        'line-width': 1,
      },
    });
  }

  if (!map.value.getLayer(LAYER_BIG_LABELS)) {
    map.value.addLayer({
      id: LAYER_BIG_LABELS,
      type: 'symbol',
      source: SOURCE_BIG_LABELS,
      minzoom: 10,
      layout: {
        'text-field': ['get', 'label'],
        'text-size': 11,
      },
      paint: {
        'text-color': '#ffffff',
        'text-halo-color': '#000000',
        'text-halo-width': 1.1,
      },
    });
  }

  if (!map.value.getLayer(LAYER_SMALL_LABELS)) {
    map.value.addLayer({
      id: LAYER_SMALL_LABELS,
      type: 'symbol',
      source: SOURCE_SMALL_LABELS,
      minzoom: 15,
      layout: {
        'text-field': ['get', 'label'],
        'text-size': 10,
      },
      paint: {
        'text-color': '#ffffff',
        'text-halo-color': '#000000',
        'text-halo-width': 1,
      },
    });
  }
}

function syncGridSources(selectedCell: CellIndex | null) {
  const bigLineFeatures = buildBigGridLineFeatures();
  const bigLabelFeatures = buildBigGridLabelFeatures();
  const selectedBigFeatures = buildSelectedBigCellFeature(selectedCell);
  const allSmallLineFeatures = buildAllSmallGridLineFeatures();
  const smallGrid = buildSmallGridForBigCell(selectedCell);

  setGeoJsonSource(SOURCE_BIG_LINES, asCollection(bigLineFeatures));
  setGeoJsonSource(SOURCE_BIG_LABELS, asCollection(bigLabelFeatures));
  setGeoJsonSource(SOURCE_BIG_SELECTED, asCollection(selectedBigFeatures));
  setGeoJsonSource(SOURCE_SMALL_LINES, asCollection(allSmallLineFeatures));
  setGeoJsonSource(SOURCE_SMALL_LABELS, asCollection(smallGrid.labelFeatures));

  ensureGridLayers();
}

const createMarker = (lng: number, lat: number) => {
  if (!map.value) return;

  const popupContent = document.createElement('div');
  const popup = new Popup({ closeOnClick: false })
    .setDOMContent(popupContent)
    .setLngLat(LngLat.convert([lng, lat]));

  const marker = new Marker()
    .setLngLat(LngLat.convert([lng, lat]))
    .setPopup(popup);

  const component = h(MapMarkerPopup, {
    onDelete: () => marker.remove(),
  });
  render(component, popupContent);

  marker.getElement().addEventListener('click', (event) => {
    event.stopPropagation();
    popup.addTo(map.value!);
  });

  marker.addTo(map.value);
};

function updateCellSelection(lng: number, lat: number) {
  const bigCell = getBigCellByLngLat(lng, lat);

  if (!bigCell) {
    selectedBigCell.value = 'None';
    selectedSmallCell.value = 'None';
    selectedFullCell.value = 'None';
    syncGridSources(null);
    return;
  }

  const bigLabel = buildBigCellLabel(bigCell.row, bigCell.col);
  const smallCell = getSmallCellByLngLat(lng, lat, bigCell);

  selectedBigCell.value = bigLabel;

  if (!smallCell) {
    selectedSmallCell.value = 'None';
    selectedFullCell.value = bigLabel;
  }
  else {
    const smallLabel = buildSmallCellLabel(smallCell.row, smallCell.col);
    selectedSmallCell.value = smallLabel;
    selectedFullCell.value = `${bigLabel}-${smallLabel}`;
  }

  syncGridSources(bigCell);
}

onMounted(() => {
  map.value = new Map({
    container: mapContainer.value!,
    style: selectedMapStyle.value?.url,
    center: [10.4, 51.3],
    zoom: 6,
  });

  map.value.on('load', () => {
    syncGridSources(null);
  });

  map.value.on('style.load', () => {
    const selectedCell = selectedBigCell.value === 'None'
      ? null
      : (() => {
          const selectedText = selectedBigCell.value;
          const match = selectedText.match(/^([A-Z]+)(\d{2,})$/);
          if (!match?.[1] || !match[2]) return null;
          const rowText = match[1];
          const colText = match[2];
          let row = 0;

          for (let i = 0; i < rowText.length; i++) {
            row = row * 26 + (rowText.charCodeAt(i) - 64);
          }

          return {
            row: row - 1,
            col: Number(colText) - 1,
          };
        })();
    syncGridSources(selectedCell);
  });

  map.value.on('click', (event: MapMouseEvent) => {
    const { lng, lat } = event.lngLat;
    createMarker(lng, lat);
    updateCellSelection(lng, lat);
  });

  map.value.on('mousemove', () => {
    map.value!.getCanvas().style.cursor = 'default';
  });
});

onUnmounted(() => {
  map.value?.remove();
});

watch(selectedMapStyle, (newMapStyle) => {
  if (!newMapStyle) return;
  map.value?.setStyle(newMapStyle.url);
});
</script>

<template>
  <div class="gridContainer">
    <div class="mapStyleSelectorWrapper">
      Available Map Styles:
      <div
        v-for="style in availableMapStyles"
        :key="style.name"
        style="display: inline; width:max-content;"
      >
        <DefaultButton
          v-if="style.url != selectedMapStyle?.url"
          class="mapStyleButton"
          @click="selectedMapStyle = style"
        >
          {{ style.name }}
        </DefaultButton>
        <DefaultButton
          v-else
          class="mapStyleButton"
          :urgency="Urgency.Okay"
          :disabled="true"
        >
          {{ style.name }}
        </DefaultButton>
      </div>
    </div>

    <div
      ref="mapContainer"
      class="map-container"
    />

    <aside class="infoPanel">
      <h3>Germany Grid</h3>
      <p>Big cell (10x10 km): {{ selectedBigCell }}</p>
      <p>Small cell (250x250 m): {{ selectedSmallCell }}</p>
      <p>Combined label: {{ selectedFullCell }}</p>
      <p class="hintText">
        Click inside Germany to select a big cell and show its 250 m subgrid.
      </p>
      <p class="hintText">
        Label format example: AZ26-F12
      </p>
    </aside>
  </div>
</template>

<style scoped>
.gridContainer {
  height: 90vh;
  border-width: 0.15rem;
  border-radius: 0.5rem;
  border-color: var(--color-blue1);
  border-style: solid;
  margin-left: auto;
  display: grid;
  grid-template-rows: min-content auto;
  grid-template-columns: 70% 30%;
}

.mapStyleSelectorWrapper {
  grid-column: 1 / 3;
  justify-self: center;
  align-self: center;
}

.mapStyleButton {
  margin: 0.5rem;
}

.map-container {
  grid-row: 2;
  grid-column: 1;
  border: 0.3rem solid var(--color-green1);
  border-radius: 0.5rem;
}

.infoPanel {
  grid-row: 2;
  grid-column: 2;
  border-left: 0.15rem solid var(--color-blue1);
  padding: 1rem;
  overflow-y: auto;
}

.infoPanel h3 {
  margin-top: 0;
}

.hintText {
  opacity: 0.85;
}
</style>

<style>
.maplibregl-marker {
  cursor: pointer;
}

.maplibregl-popup-content {
  background-color: var(--main-bg-color);
}

.maplibregl-popup-tip {
  border-top-color: var(--main-bg-color) !important;
}

.maplibregl-popup-close-button {
  color: var(--main-fg-color);
}
</style>
