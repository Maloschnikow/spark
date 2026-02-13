<script setup lang="ts">
import { ref, onMounted, type Ref, onUnmounted, h, render, watch } from 'vue';
import { Popup, Map, Marker, LngLat, MapMouseEvent } from 'maplibre-gl';
import 'maplibre-gl/dist/maplibre-gl.css';


const mapContainer: Ref<HTMLElement | null> = ref(null);
const map: Ref<Map | null> = ref(null);

const createMarker = (lng: number, lat: number) => {
    if (!map.value) return;

    const popupContent = document.createElement('div');
    const popup = new Popup({closeOnClick: false})
    .setDOMContent(popupContent)
    .setLngLat(LngLat.convert([lng, lat]));

    const marker = new Marker()
    .setLngLat(LngLat.convert([lng, lat]))
    .setPopup(popup);

    const component = h(MapMarkerPopup, {
        onDelete: () => {
            marker.remove();
        },
    })
    render(component, popupContent);

    marker.getElement().addEventListener('click', (e) => {
        e.stopPropagation();
        popup.addTo(map.value!);
    });

    marker.addTo(map.value);
}

onMounted(() => {
    map.value = new Map({
        container: mapContainer.value!,
        style: 'https://sgx.geodatenzentrum.de/gdz_basemapde_vektor/styles/bm_web_top.json',
        center: [10.4, 51.3], // [lng, lat] not lat, lng!
        zoom: 6,
    });

    map.value.on('click', (e: MapMouseEvent) => {
        const {lng, lat} = e.lngLat;
        createMarker(lng, lat);
    });

    map.value.on('mousemove', () => {
        map.value!.getCanvas().style.cursor = 'default';
    });
});

onUnmounted(() => {
    map.value?.remove();
});
</script>

<template>
    <div ref="mapContainer" class="map-container" />
</template>

<style>
.map-container {
    height: 90vh;
    width: 90vw;
    margin: auto;
    border-width: 0.15rem;
    border-radius: 0.5rem;
    border-color: var(--color-blue1);
    border-style: solid;
}

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
    color: var(--main-fg-color)
}

</style>