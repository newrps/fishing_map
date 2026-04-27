<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher } from 'svelte';
  import type { Station } from '$lib/api';
  import { regionColor } from '$lib/api';

  export let stations: Station[] = [];
  export let selected: Station | null = null;

  const dispatch = createEventDispatcher<{ select: Station }>();

  let mapEl: HTMLDivElement;
  let map: any;
  let L: any;
  let markersByCode: Record<string, any> = {};

  onMount(async () => {
    // Leaflet must be loaded client-side only (it touches window).
    L = (await import('leaflet')).default;
    await import('leaflet/dist/leaflet.css');

    map = L.map(mapEl, {
      center: [36.0, 127.8],   // 한반도 중앙
      zoom: 7,
      zoomControl: true,
      attributionControl: true
    });

    const VWORLD_KEY = import.meta.env.VITE_VWORLD_KEY;
    const baseLayers: Record<string, any> = {};

    // VWorld 유효 레이어: Base, midnight, Hybrid, Satellite, white
    if (VWORLD_KEY) {
      const vworldWhite = L.tileLayer(
        `https://api.vworld.kr/req/wmts/1.0.0/${VWORLD_KEY}/white/{z}/{y}/{x}.png`,
        { attribution: '© VWorld', maxZoom: 19, className: 'vworld-white' }
      );
      const vworldBase = L.tileLayer(
        `https://api.vworld.kr/req/wmts/1.0.0/${VWORLD_KEY}/Base/{z}/{y}/{x}.png`,
        { attribution: '© VWorld', maxZoom: 19, className: 'vworld-base' }
      );
      const vworldMidnight = L.tileLayer(
        `https://api.vworld.kr/req/wmts/1.0.0/${VWORLD_KEY}/midnight/{z}/{y}/{x}.png`,
        { attribution: '© VWorld', maxZoom: 19 }
      );
      const vworldSatellite = L.tileLayer(
        `https://api.vworld.kr/req/wmts/1.0.0/${VWORLD_KEY}/Satellite/{z}/{y}/{x}.jpeg`,
        { attribution: '© VWorld', maxZoom: 19 }
      );
      baseLayers['VWorld 일반'] = vworldBase;
      baseLayers['VWorld 백지도'] = vworldWhite;
      baseLayers['VWorld 다크'] = vworldMidnight;
      baseLayers['VWorld 위성'] = vworldSatellite;
      vworldBase.addTo(map);
    }

    // OSM (한글 라벨 + 풀컬러 - 백업)
    const osm = L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
      attribution: '© OpenStreetMap',
      subdomains: 'abc',
      maxZoom: 19
    });
    baseLayers['OpenStreetMap'] = osm;
    if (!VWORLD_KEY) osm.addTo(map);

    // 오버레이: OpenSeaMap (등대·항로표지·수심)
    const seamark = L.tileLayer('https://tiles.openseamap.org/seamark/{z}/{x}/{y}.png', {
      attribution: '© OpenSeaMap',
      maxZoom: 18,
      opacity: 0.9,
      className: 'seamark-layer'
    }).addTo(map);

    L.control
      .layers(baseLayers, { '해도 (등대·표지)': seamark }, {
        position: 'topright',
        collapsed: true
      })
      .addTo(map);

    renderMarkers();
  });

  onDestroy(() => {
    if (map) map.remove();
  });

  function renderMarkers() {
    if (!map || !L) return;
    // Clear existing
    Object.values(markersByCode).forEach((m: any) => m.remove());
    markersByCode = {};

    for (const s of stations) {
      const marker = L.circleMarker([s.lat, s.lon], {
        radius: 8,
        fillColor: regionColor(s.region),
        color: '#fff',
        weight: 2,
        fillOpacity: 0.95
      }).addTo(map);

      marker.bindTooltip(`${s.name} (${s.region})`, {
        direction: 'top',
        offset: [0, -8]
      });

      marker.on('click', () => dispatch('select', s));
      markersByCode[s.code] = marker;
    }
  }

  $: if (map && stations.length) renderMarkers();

  $: if (map && selected) {
    map.flyTo([selected.lat, selected.lon], Math.max(map.getZoom(), 10), {
      duration: 0.6
    });
    Object.entries(markersByCode).forEach(([code, m]: [string, any]) => {
      m.setStyle({
        radius: code === selected!.code ? 12 : 8,
        weight: code === selected!.code ? 3 : 2
      });
    });
  }
</script>

<div class="map" bind:this={mapEl}></div>

<style>
  .map {
    width: 100%;
    height: 100%;
    background: #cfe5f3;
  }
  /* VWorld Base는 풀컬러라 회색조로 톤다운 → 바다 강조 */
  :global(.vworld-base) {
    filter: saturate(0.5) brightness(1.04) hue-rotate(-3deg);
  }
  /* OpenSeaMap 오버레이는 색 보존 (등대·표지 색깔 살리기) */
  :global(.seamark-layer) {
    filter: none !important;
  }
</style>
