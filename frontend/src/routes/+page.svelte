<script lang="ts">
  import { onMount } from 'svelte';
  import Map from '$lib/components/Map.svelte';
  import MonthCalendar from '$lib/components/MonthCalendar.svelte';
  import { fetchStations, fetchTide, fetchConditions, regionColor } from '$lib/api';
  import type { Station, TideResponse, Conditions, Region } from '$lib/api';

  let stations: Station[] = [];
  let selected: Station | null = null;
  let tide: TideResponse | null = null;
  let conditions: Conditions | null = null;
  let loadingTide = false;
  let tideError: string | null = null;
  let loadStationsError: string | null = null;

  // 관측소 + 날짜별 물때 캐시 — key: `${code}:${ymd}`
  let tideCache: Record<string, TideResponse> = {};
  // 진행 중인 페치 (중복 요청 방지)
  let tideInflight: Record<string, Promise<TideResponse>> = {};

  // 날짜별 날씨/수온 캐시 — key: `${code}:${ymd}`
  let conditionsCache: Record<string, Conditions> = {};
  let conditionsInflight: Record<string, Promise<Conditions>> = {};

  // 처음 선택 시 — 양쪽으로 적당히
  const WINDOW_NEUTRAL = [-3, -2, -1, 1, 2, 3, 4, 5, 6, 7];
  // ›로 미래 이동 중 — 앞으로 길게
  const WINDOW_FORWARD = [-1, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11];
  // ‹로 과거 이동 중 — 뒤로 길게
  const WINDOW_BACKWARD = [-11, -10, -9, -8, -7, -6, -5, -4, -3, -2, -1, 1];

  function todayYMD(): string {
    const d = new Date();
    return `${d.getFullYear()}${String(d.getMonth() + 1).padStart(2, '0')}${String(d.getDate()).padStart(2, '0')}`;
  }
  function ymdToDate(ymd: string): Date {
    return new Date(
      parseInt(ymd.slice(0, 4)),
      parseInt(ymd.slice(4, 6)) - 1,
      parseInt(ymd.slice(6, 8))
    );
  }
  function dateToYmd(d: Date): string {
    return `${d.getFullYear()}${String(d.getMonth() + 1).padStart(2, '0')}${String(d.getDate()).padStart(2, '0')}`;
  }
  let selectedDate: string = todayYMD();

  // 캐시 히트면 즉시 resolve, 진행 중이면 그 promise, 없으면 새로 페치
  function ensureTide(code: string, ymd: string): Promise<TideResponse> {
    const key = `${code}:${ymd}`;
    if (tideCache[key]) return Promise.resolve(tideCache[key]);
    if (tideInflight[key]) return tideInflight[key];
    const p = fetchTide(code, ymd)
      .then((t) => {
        if (selected?.code === code) {
          tideCache = { ...tideCache, [key]: t };
        }
        return t;
      })
      .finally(() => {
        delete tideInflight[key];
      });
    tideInflight[key] = p;
    return p;
  }

  function ensureConditions(code: string, ymd: string): Promise<Conditions> {
    const key = `${code}:${ymd}`;
    if (conditionsCache[key]) return Promise.resolve(conditionsCache[key]);
    if (conditionsInflight[key]) return conditionsInflight[key];
    const p = fetchConditions(code, ymd)
      .then((c) => {
        if (selected?.code === code) {
          conditionsCache = { ...conditionsCache, [key]: c };
        }
        return c;
      })
      .finally(() => {
        delete conditionsInflight[key];
      });
    conditionsInflight[key] = p;
    return p;
  }

  // direction: -1=과거, +1=미래, 0=양방향
  function prefetchAround(code: string, centerYmd: string, direction = 0) {
    const offsets =
      direction > 0 ? WINDOW_FORWARD :
      direction < 0 ? WINDOW_BACKWARD :
      WINDOW_NEUTRAL;
    const center = ymdToDate(centerYmd);
    for (const offset of offsets) {
      const d = new Date(center);
      d.setDate(d.getDate() + offset);
      const ymd = dateToYmd(d);
      ensureTide(code, ymd).catch(() => {/* silent */});
      ensureConditions(code, ymd).catch(() => {/* silent */});
    }
  }

  // 지역별 카메라 좌표 (각 해역의 중앙 + 적절한 줌)
  const REGION_VIEWS: Record<Region, { lat: number; lon: number; zoom: number }> = {
    '서해': { lat: 36.5, lon: 126.2, zoom: 7 },
    '남해': { lat: 34.7, lon: 127.8, zoom: 8 },
    '동해': { lat: 37.5, lon: 129.5, zoom: 7 },
    '제주': { lat: 33.4, lon: 126.5, zoom: 9 }
  };
  const REGIONS: Region[] = ['서해', '남해', '동해', '제주'];
  let flyTarget: { lat: number; lon: number; zoom: number; ts: number } | null = null;

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
    tideCache = {};
    conditionsCache = {};

    const code = selected.code;
    try {
      const [t, c] = await Promise.allSettled([
        ensureTide(code, selectedDate),
        ensureConditions(code, selectedDate)
      ]);
      if (selected?.code !== code) return;
      if (t.status === 'fulfilled') tide = t.value;
      else tideError = t.reason?.message ?? String(t.reason);
      if (c.status === 'fulfilled') conditions = c.value;
    } finally {
      loadingTide = false;
    }

    prefetchAround(code, selectedDate);
  }

  async function handleDateChange(e: CustomEvent<string>) {
    if (!selected) {
      selectedDate = e.detail;
      return;
    }
    const code = selected.code;
    const oldDate = selectedDate;
    selectedDate = e.detail;
    const direction = selectedDate > oldDate ? 1 : selectedDate < oldDate ? -1 : 0;
    const key = `${code}:${selectedDate}`;

    // 양쪽 캐시 히트면 즉시 표시
    if (tideCache[key] && conditionsCache[key]) {
      tide = tideCache[key];
      conditions = conditionsCache[key];
      tideError = null;
      prefetchAround(code, selectedDate, direction);
      return;
    }

    // 일부 또는 전부 캐시 미스 — 인플라이트 promise 기다리거나 새로 페치
    tide = tideCache[key] ?? null;
    conditions = conditionsCache[key] ?? null;
    tideError = null;
    loadingTide = !tide; // tide가 이미 있으면 로딩 표시 생략
    try {
      const [t, c] = await Promise.allSettled([
        ensureTide(code, selectedDate),
        ensureConditions(code, selectedDate)
      ]);
      if (selected?.code !== code || selectedDate !== e.detail) return;
      if (t.status === 'fulfilled') tide = t.value;
      else tideError = t.reason?.message ?? String(t.reason);
      if (c.status === 'fulfilled') conditions = c.value;
    } finally {
      loadingTide = false;
    }
    prefetchAround(code, selectedDate, direction);
  }

  function flyToRegion(r: Region) {
    const v = REGION_VIEWS[r];
    flyTarget = { ...v, ts: Date.now() };
  }

  // 월간 캘린더 모달
  let monthOpen = false;
  function openMonth() { if (selected) monthOpen = true; }
  function closeMonth() { monthOpen = false; }

  async function pickMonthDate(e: CustomEvent<string>) {
    const ymd = e.detail;
    if (!selected) return;
    monthOpen = false;
    // handleDateChange와 동일 흐름 재사용
    await handleDateChange(new CustomEvent('dateChange', { detail: ymd }));
  }
</script>

<!-- 풀스크린 맵 -->
<main class="full-map">
  <Map
    {stations}
    {selected}
    {tide}
    {conditions}
    {selectedDate}
    {flyTarget}
    loading={loadingTide}
    error={tideError}
    on:select={handleSelect}
    on:dateChange={handleDateChange}
    on:openMonth={openMonth}
  />

  {#if monthOpen && selected}
    <MonthCalendar
      station={selected}
      {selectedDate}
      {ensureTide}
      {ensureConditions}
      on:close={closeMonth}
      on:pick={pickMonthDate}
    />
  {/if}

  <!-- 좌상단 브랜드 + 필터 오버레이 -->
  <div class="overlay top-left">
    <div class="brand">
      <svg viewBox="0 0 40 40" class="logo-icon" aria-hidden="true">
        <defs>
          <linearGradient id="seaGrad" x1="0" y1="0" x2="0" y2="1">
            <stop offset="0%" stop-color="#4FC3F7" />
            <stop offset="100%" stop-color="#0277BD" />
          </linearGradient>
        </defs>
        <circle cx="20" cy="20" r="19" fill="url(#seaGrad)" />
        <path d="M4 24 Q10 20 16 24 T28 24 T40 24 V40 H4 Z" fill="#fff" opacity="0.25" />
        <path d="M4 28 Q10 24 16 28 T28 28 T40 28 V40 H4 Z" fill="#fff" opacity="0.4" />
        <circle cx="29" cy="13" r="3.5" fill="#FFE082" />
      </svg>
      <h1>Ps물때지도</h1>
    </div>
    <div class="region-jump">
      {#each REGIONS as r}
        <button
          class="jump-btn"
          style="--c: {regionColor(r)}"
          on:click={() => flyToRegion(r)}
          title="{r}로 이동"
        >
          <span class="dot" style="background: {regionColor(r)}"></span>
          {r}
        </button>
      {/each}
    </div>
  </div>

  {#if loadStationsError}
    <div class="overlay bottom-center error-banner">
      ⚠️ 관측소 목록 로드 실패: {loadStationsError}
    </div>
  {/if}

  <!-- 우하단 안내 -->
  <div class="overlay bottom-right hint-card">
    <small>마커 클릭 → 물때·날씨·수온</small>
  </div>
</main>

<style>
  .full-map {
    position: fixed;
    inset: 0;
    overflow: hidden;
  }
  /* Map의 .map div가 부모 100%를 채우도록 */
  :global(.full-map > .map) {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
  }

  .overlay {
    position: absolute;
    z-index: 500;
    pointer-events: none;
  }
  .overlay > * { pointer-events: auto; }

  .top-left {
    top: 16px;
    left: 16px;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .brand {
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(10px);
    border-radius: 12px;
    padding: 8px 14px;
    display: flex;
    align-items: center;
    gap: 10px;
    box-shadow: 0 2px 12px rgba(0, 61, 92, 0.15);
  }
  .logo-icon {
    width: 32px;
    height: 32px;
  }
  .brand h1 {
    margin: 0;
    font-size: 18px;
    color: #003D5C;
    font-weight: 700;
    letter-spacing: -0.5px;
  }

  .region-jump {
    display: flex;
    gap: 6px;
    background: rgba(255, 255, 255, 0.95);
    backdrop-filter: blur(10px);
    border-radius: 20px;
    padding: 6px;
    box-shadow: 0 2px 12px rgba(0, 61, 92, 0.12);
    flex-wrap: wrap;
  }
  .jump-btn {
    background: transparent;
    border: 1.5px solid transparent;
    border-radius: 14px;
    padding: 5px 12px;
    font-size: 13px;
    color: #003D5C;
    cursor: pointer;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    transition: all 0.15s;
    font-family: inherit;
    font-weight: 500;
  }
  .jump-btn:hover {
    background: rgba(0, 61, 92, 0.08);
    border-color: var(--c);
  }
  .jump-btn:active {
    transform: scale(0.96);
  }
  .jump-btn .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    display: inline-block;
  }

  .bottom-right {
    bottom: 16px;
    right: 16px;
  }
  .hint-card {
    background: rgba(0, 61, 92, 0.85);
    color: #fff;
    padding: 6px 12px;
    border-radius: 14px;
    font-size: 11px;
  }

  .bottom-center {
    bottom: 16px;
    left: 50%;
    transform: translateX(-50%);
  }
  .error-banner {
    background: #ffebee;
    color: #b71c1c;
    border-left: 3px solid #d32f2f;
    padding: 10px 16px;
    border-radius: 8px;
    font-size: 13px;
    box-shadow: 0 2px 8px rgba(0,0,0,0.1);
  }

  @media (max-width: 480px) {
    .top-left {
      top: 8px;
      left: 8px;
      right: 8px;
    }
    .brand h1 { font-size: 16px; }
    .region-jump { font-size: 11px; }
  }
</style>
