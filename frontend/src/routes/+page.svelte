<script lang="ts">
  import { onMount } from 'svelte';
  import Map from '$lib/components/Map.svelte';
  import TidePanel from '$lib/components/TidePanel.svelte';
  import { fetchStations, fetchTide, fetchConditions, regionColor } from '$lib/api';
  import type { Station, TideResponse, Conditions, Region } from '$lib/api';

  let stations: Station[] = [];
  let selected: Station | null = null;
  let tide: TideResponse | null = null;
  let conditions: Conditions | null = null;
  let loadingTide = false;
  let tideError: string | null = null;
  let loadStationsError: string | null = null;

  const REGIONS: Region[] = ['서해', '남해', '동해', '제주'];
  let regionFilter: Set<Region> = new Set(REGIONS);

  $: filteredStations = stations.filter((s) => regionFilter.has(s.region));

  onMount(async () => {
    try {
      stations = await fetchStations();
    } catch (e: any) {
      loadStationsError = e.message ?? String(e);
    }
  });

  async function handleSelect(e: CustomEvent<Station>) {
    selected = e.detail;
    tide = null;
    conditions = null;
    tideError = null;
    loadingTide = true;
    try {
      // 물때와 날씨/수온 병렬 호출
      const [t, c] = await Promise.allSettled([
        fetchTide(selected.code),
        fetchConditions(selected.code)
      ]);
      if (t.status === 'fulfilled') tide = t.value;
      else tideError = t.reason?.message ?? String(t.reason);
      if (c.status === 'fulfilled') conditions = c.value;
      // conditions 실패는 silent (날씨 없어도 물때는 보여줘야 함)
    } finally {
      loadingTide = false;
    }
  }

  function toggleRegion(r: Region) {
    if (regionFilter.has(r)) regionFilter.delete(r);
    else regionFilter.add(r);
    regionFilter = new Set(regionFilter); // trigger reactivity
  }
</script>

<div class="layout">
  <aside class="sidebar">
    <header class="brand">
      <div class="logo">
        <svg viewBox="0 0 40 40" class="logo-icon" aria-hidden="true">
          <defs>
            <linearGradient id="seaGrad" x1="0" y1="0" x2="0" y2="1">
              <stop offset="0%" stop-color="#4FC3F7" />
              <stop offset="100%" stop-color="#0277BD" />
            </linearGradient>
          </defs>
          <circle cx="20" cy="20" r="19" fill="url(#seaGrad)" />
          <path
            d="M4 24 Q10 20 16 24 T28 24 T40 24 V40 H4 Z"
            fill="#fff"
            opacity="0.25"
          />
          <path
            d="M4 28 Q10 24 16 28 T28 28 T40 28 V40 H4 Z"
            fill="#fff"
            opacity="0.4"
          />
          <circle cx="29" cy="13" r="3.5" fill="#FFE082" />
        </svg>
        <h1>물때지도</h1>
      </div>
      <p>전국 조위관측소 실시간 만조·간조</p>
    </header>

    <div class="filter">
      <span class="filter-label">지역 필터</span>
      <div class="filter-chips">
        {#each REGIONS as r}
          <button
            class="chip"
            class:active={regionFilter.has(r)}
            style="--c: {regionColor(r)}"
            on:click={() => toggleRegion(r)}
          >
            <span class="dot" style="background: {regionColor(r)}"></span>
            {r}
          </button>
        {/each}
      </div>
    </div>

    {#if loadStationsError}
      <div class="error-banner">⚠️ 관측소 목록 로드 실패: {loadStationsError}</div>
    {/if}

    <TidePanel
      station={selected}
      {tide}
      {conditions}
      loading={loadingTide}
      error={tideError}
    />

    <footer>
      <small>
        조위 데이터: 국립해양조사원 OpenAPI<br>
        지도: OpenStreetMap · CartoDB · OpenSeaMap
      </small>
    </footer>
  </aside>

  <main class="map-area">
    <Map stations={filteredStations} {selected} on:select={handleSelect} />
  </main>
</div>

<style>
  .layout {
    display: grid;
    grid-template-columns: 380px 1fr;
    height: 100vh;
    gap: 0;
  }

  .sidebar {
    background: linear-gradient(180deg, #f4f7fa 0%, #e8eef3 100%);
    padding: 20px 18px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 16px;
    border-right: 1px solid #d8e0e6;
  }

  .brand .logo {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-bottom: 4px;
  }
  .brand .logo-icon {
    width: 36px;
    height: 36px;
    filter: drop-shadow(0 2px 4px rgba(2, 119, 189, 0.25));
  }
  .sidebar header h1 {
    font-size: 24px;
    margin: 0;
    color: #003D5C;
    font-weight: 700;
    letter-spacing: -0.5px;
  }
  .sidebar header p {
    margin: 0;
    color: #6b7c8a;
    font-size: 13px;
  }

  .filter {
    background: #fff;
    border-radius: 12px;
    padding: 14px;
    box-shadow: 0 2px 8px rgba(0, 61, 92, 0.06);
  }
  .filter-label {
    display: block;
    font-size: 12px;
    color: #6b7c8a;
    margin-bottom: 8px;
  }
  .filter-chips {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }
  .chip {
    background: #f0f4f7;
    border: 1.5px solid transparent;
    border-radius: 14px;
    padding: 5px 10px;
    font-size: 13px;
    color: #6b7c8a;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    transition: all 0.15s;
  }
  .chip.active {
    background: #fff;
    border-color: var(--c);
    color: #003D5C;
    font-weight: 500;
  }
  .chip .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    display: inline-block;
  }

  .error-banner {
    background: #ffebee;
    border-left: 3px solid #d32f2f;
    color: #b71c1c;
    padding: 10px 12px;
    border-radius: 6px;
    font-size: 13px;
  }

  footer {
    margin-top: auto;
    padding-top: 12px;
    color: #95a3ad;
    line-height: 1.5;
  }

  .map-area { position: relative; }

  @media (max-width: 768px) {
    .layout {
      grid-template-columns: 1fr;
      grid-template-rows: 1fr auto;
    }
    .sidebar {
      grid-row: 2;
      max-height: 50vh;
    }
    .map-area {
      grid-row: 1;
      min-height: 50vh;
    }
  }
</style>
