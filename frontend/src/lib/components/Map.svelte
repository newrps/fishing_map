<script lang="ts">
  import { onMount, onDestroy, createEventDispatcher, tick } from 'svelte';
  import type { Station, TideResponse, Conditions } from '$lib/api';
  import { regionColor } from '$lib/api';
  import StationPopup from './StationPopup.svelte';

  export let stations: Station[] = [];
  export let selected: Station | null = null;
  export let tide: TideResponse | null = null;
  export let conditions: Conditions | null = null;
  export let selectedDate: string;
  export let loading = false;
  export let error: string | null = null;
  // 외부에서 카메라 이동 트리거. ts 값이 바뀔 때마다 fly.
  export let flyTarget: { lat: number; lon: number; zoom: number; ts: number } | null = null;

  const dispatch = createEventDispatcher<{
    select: Station;
    dateChange: string;
  }>();

  let mapEl: HTMLDivElement;
  let map: any;
  let L: any;
  let markersByCode: Record<string, any> = {};
  let popupEl: HTMLDivElement;        // Svelte 컴포넌트가 마운트되는 컨테이너
  let popupComponent: StationPopup | null = null;
  let leafletPopup: any = null;

  onMount(async () => {
    L = (await import('leaflet')).default;
    await import('leaflet/dist/leaflet.css');

    map = L.map(mapEl, {
      center: [36.0, 127.8],
      zoom: 7,
      zoomControl: true,
      attributionControl: true
    });

    const VWORLD_KEY = import.meta.env.VITE_VWORLD_KEY;
    const baseLayers: Record<string, any> = {};

    if (VWORLD_KEY) {
      const vworldBase = L.tileLayer(
        `https://api.vworld.kr/req/wmts/1.0.0/${VWORLD_KEY}/Base/{z}/{y}/{x}.png`,
        { attribution: '© VWorld', maxZoom: 19, className: 'vworld-base' }
      );
      const vworldWhite = L.tileLayer(
        `https://api.vworld.kr/req/wmts/1.0.0/${VWORLD_KEY}/white/{z}/{y}/{x}.png`,
        { attribution: '© VWorld', maxZoom: 19, className: 'vworld-white' }
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

    const osm = L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
      attribution: '© OpenStreetMap',
      subdomains: 'abc',
      maxZoom: 19
    });
    baseLayers['OpenStreetMap'] = osm;
    if (!VWORLD_KEY) osm.addTo(map);

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

    // 팝업 컨테이너 (Svelte 컴포넌트 마운트용)
    popupEl = document.createElement('div');

    // 팝업 닫힐 때 정리
    map.on('popupclose', () => {
      // selected 해제는 부모가 안 함 — 다시 열 수 있게 유지
    });

    renderMarkers();
  });

  onDestroy(() => {
    popupComponent?.$destroy();
    if (map) map.remove();
  });

  function renderMarkers() {
    if (!map || !L) return;
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

  // 마커 데이터 변경 시 재렌더
  $: if (map && stations.length) renderMarkers();

  // flyTarget 변경 시 카메라 이동
  let lastFlyTs = 0;
  $: if (map && flyTarget && flyTarget.ts !== lastFlyTs) {
    lastFlyTs = flyTarget.ts;
    map.flyTo([flyTarget.lat, flyTarget.lon], flyTarget.zoom, { duration: 0.8 });
  }

  // 선택된 관측소가 바뀌면 팝업 열기
  $: if (map && L && selected) {
    handleSelectionChange(selected);
  }

  // tide/conditions/selectedDate/loading 변경 시 팝업 컴포넌트 props 업데이트
  $: if (popupComponent && selected) {
    popupComponent.$set({
      station: selected,
      tide,
      conditions,
      selectedDate,
      loading,
      error
    });
  }

  async function handleSelectionChange(station: Station) {
    map.flyTo([station.lat, station.lon], Math.max(map.getZoom(), 10), {
      duration: 0.6
    });

    // 마커 강조
    Object.entries(markersByCode).forEach(([code, m]: [string, any]) => {
      m.setStyle({
        radius: code === station.code ? 12 : 8,
        weight: code === station.code ? 3 : 2
      });
    });

    // 기존 팝업 컴포넌트 destroy
    popupComponent?.$destroy();

    // 새 컨테이너 생성하고 Svelte 컴포넌트 마운트
    popupEl = document.createElement('div');
    popupComponent = new StationPopup({
      target: popupEl,
      props: {
        station,
        tide,
        conditions,
        selectedDate,
        loading,
        error
      }
    });

    // 팝업 내에서 dateChange 이벤트 → 부모로 전파
    popupComponent.$on('dateChange', (e: CustomEvent<string>) => {
      dispatch('dateChange', e.detail);
    });

    // Leaflet 팝업 열기
    leafletPopup = L.popup({
      closeButton: true,
      autoClose: false,
      closeOnClick: false,
      maxWidth: 360,
      minWidth: 320,
      className: 'station-popup'
    })
      .setLatLng([station.lat, station.lon])
      .setContent(popupEl)
      .openOn(map);
  }
</script>

<div class="map" bind:this={mapEl}></div>

<style>
  .map {
    width: 100%;
    height: 100%;
    background: #cfe5f3;
  }
  :global(.vworld-base) {
    filter: saturate(0.5) brightness(1.04) hue-rotate(-3deg);
  }
  :global(.seamark-layer) {
    filter: none !important;
  }

  /* Leaflet 팝업 커스텀 */
  :global(.station-popup .leaflet-popup-content-wrapper) {
    border-radius: 14px;
    padding: 0;
    box-shadow: 0 6px 24px rgba(0, 61, 92, 0.2);
  }
  :global(.station-popup .leaflet-popup-content) {
    margin: 14px 16px;
    line-height: 1.4;
  }
  :global(.station-popup .leaflet-popup-tip) {
    box-shadow: 0 6px 24px rgba(0, 61, 92, 0.2);
  }
</style>
