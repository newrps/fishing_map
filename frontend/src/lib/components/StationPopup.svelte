<script lang="ts">
  import type { Station, TideResponse, TideEvent, Conditions } from '$lib/api';
  import { findSeaPartingSpot } from '$lib/api';
  import { getLunarInfo } from '$lib/lunar';
  import { createEventDispatcher } from 'svelte';

  export let station: Station;
  export let tide: TideResponse | null = null;
  export let conditions: Conditions | null = null;
  export let selectedDate: string;  // YYYYMMDD
  export let loading = false;
  export let error: string | null = null;

  const dispatch = createEventDispatcher<{ dateChange: string; openMonth: undefined }>();

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

  function shiftDate(days: number) {
    const d = ymdToDate(selectedDate);
    d.setDate(d.getDate() + days);
    dispatch('dateChange', dateToYmd(d));
  }

  function isToday(ymd: string): boolean {
    return ymd === dateToYmd(new Date());
  }

  $: dateObj = ymdToDate(selectedDate);
  $: lunar = getLunarInfo(dateObj);
  $: dateLabel = `${dateObj.getMonth() + 1}월 ${dateObj.getDate()}일 (${['일','월','화','수','목','금','토'][dateObj.getDay()]})`;
  $: seaPartingSpot = findSeaPartingSpot(station.code);
  $: lowestLow = (tide?.events ?? [])
    .filter((e) => e.kind === 'low')
    .reduce<TideEvent | null>(
      (acc, cur) => (acc === null || cur.level_cm < acc.level_cm ? cur : acc),
      null
    );

  function kindLabel(e: TideEvent): string {
    return e.kind === 'high' ? '만조' : e.kind === 'low' ? '간조' : e.kind;
  }

  function kindColor(e: TideEvent): string {
    return e.kind === 'high' ? '#1E88E5' : '#EF6C00';
  }
</script>

<div class="popup">
  <header>
    <div>
      <h3>{station.name}</h3>
      <span class="region">{station.region}</span>
    </div>
    <small class="code">{station.code}</small>
  </header>

  <!-- 날짜 네비게이션 -->
  <div class="date-nav">
    <button class="nav-btn" on:click={() => shiftDate(-1)} aria-label="이전">‹</button>
    <div class="date-info">
      <div class="date-main">
        {dateLabel}
        {#if isToday(selectedDate)}<span class="today-tag">오늘</span>{/if}
      </div>
      <div class="lunar-info">
        음력 {lunar.day}일
        {#if lunar.label}
          <span class="badge" style="background: {lunar.badgeColor}">{lunar.label}</span>
        {/if}
      </div>
    </div>
    <button class="nav-btn" on:click={() => shiftDate(1)} aria-label="다음">›</button>
  </div>

  <button class="month-trigger" on:click={() => dispatch('openMonth')}>
    📅 한 달 한눈에 보기
  </button>

  {#if loading}
    <div class="state">로딩 중…</div>
  {:else if error}
    <div class="state error">⚠️ {error}</div>
  {:else}
    <!-- 현재 날씨/수온 (실시간 - 항상 표시) -->
    {#if conditions}
      <div class="now">
        {#if !isToday(selectedDate)}
          <div class="live-tag">🟢 현재 (실시간)</div>
        {/if}
        <div class="now-cell">
          <span class="emo">{conditions.weather_emoji}</span>
          <div>
            <div class="big">{conditions.temperature?.toFixed(1) ?? '—'}°</div>
            <div class="lbl">{conditions.weather_label}</div>
          </div>
        </div>
        <div class="now-cell">
          <span class="emo">🌊</span>
          <div>
            <div class="big">{conditions.sea_temperature?.toFixed(1) ?? '—'}°</div>
            <div class="lbl">수온</div>
          </div>
        </div>
        <div class="now-meta">
          💨 {conditions.wind_label} {conditions.wind_speed?.toFixed(0) ?? '—'}km/h ·
          파고 {conditions.wave_height?.toFixed(1) ?? '—'}m ·
          습도 {conditions.humidity ?? '—'}%
        </div>
      </div>
    {/if}

    <!-- 만조/간조 -->
    {#if tide}
      <ul class="events">
        {#each tide.events as e}
          <li>
            <span class="kind" style="color: {kindColor(e)}">{kindLabel(e)}</span>
            <span class="time">{e.time}</span>
            <span class="level">{e.level_cm}cm</span>
          </li>
        {/each}
      </ul>

      <!-- 바다갈라짐 -->
      {#if seaPartingSpot && lowestLow && lowestLow.level_cm < 30}
        <div class="sea-parting">
          🌊 <strong>{seaPartingSpot.name}</strong> · {lowestLow.time} 갈라짐 가능 ✨
        </div>
      {/if}
    {/if}
  {/if}
</div>

<style>
  .popup {
    width: 320px;
    font-family: inherit;
    padding: 4px;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 0 4px 10px 4px;
    border-bottom: 1px solid #f0f0f0;
    margin-bottom: 10px;
  }
  header h3 {
    font-size: 18px;
    margin: 0 0 4px 0;
    color: #003D5C;
  }
  .region {
    background: #E3F2FD;
    color: #0077BE;
    padding: 2px 8px;
    border-radius: 10px;
    font-size: 11px;
    font-weight: 500;
  }
  .code {
    color: #999;
    font-size: 10px;
    font-family: ui-monospace, monospace;
  }

  .date-nav {
    display: grid;
    grid-template-columns: 36px 1fr 36px;
    align-items: center;
    background: #f4f7fa;
    border-radius: 10px;
    padding: 6px;
    margin-bottom: 12px;
  }
  .nav-btn {
    background: transparent;
    border: none;
    font-size: 24px;
    color: #003D5C;
    cursor: pointer;
    line-height: 1;
    border-radius: 6px;
    padding: 4px 0;
    transition: background 0.15s;
  }
  .nav-btn:hover { background: #e3edf4; }

  .date-info {
    text-align: center;
  }
  .date-main {
    font-size: 14px;
    font-weight: 600;
    color: #003D5C;
  }
  .today-tag {
    background: #FFB800;
    color: #fff;
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 8px;
    margin-left: 4px;
    font-weight: 600;
  }
  .lunar-info {
    font-size: 11px;
    color: #888;
    margin-top: 2px;
  }
  .badge {
    color: #fff;
    padding: 1px 6px;
    border-radius: 8px;
    font-size: 10px;
    font-weight: 600;
    margin-left: 4px;
  }

  /* 현재 상태 */
  .now {
    background: linear-gradient(135deg, #E3F2FD 0%, #F3E5F5 100%);
    border-radius: 10px;
    padding: 10px;
    margin-bottom: 10px;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 8px;
  }
  .now-cell {
    display: flex;
    align-items: center;
    gap: 8px;
    background: rgba(255,255,255,0.7);
    border-radius: 8px;
    padding: 6px 8px;
  }
  .live-tag {
    grid-column: 1 / -1;
    font-size: 10px;
    color: #2E7D32;
    font-weight: 600;
    text-align: center;
    background: rgba(255,255,255,0.6);
    padding: 3px 8px;
    border-radius: 6px;
  }
  .now-cell .emo { font-size: 22px; line-height: 1; }
  .big { font-size: 17px; font-weight: 600; color: #003D5C; line-height: 1; }
  .lbl { font-size: 10px; color: #666; margin-top: 2px; }
  .now-meta {
    grid-column: 1 / -1;
    font-size: 10px;
    color: #555;
    text-align: center;
    background: rgba(255,255,255,0.5);
    padding: 4px 8px;
    border-radius: 6px;
  }

  /* 만조/간조 */
  .events {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .events li {
    display: grid;
    grid-template-columns: 50px 70px 1fr;
    align-items: center;
    padding: 6px 4px;
    border-bottom: 1px solid #f5f5f5;
    font-size: 13px;
  }
  .events li:last-child { border-bottom: none; }
  .kind { font-weight: 600; font-size: 12px; }
  .time { font-family: ui-monospace, monospace; font-size: 14px; color: #333; }
  .level { color: #777; text-align: right; font-size: 12px; }

  .sea-parting {
    margin-top: 8px;
    background: #FFF8E1;
    border-left: 3px solid #FFB800;
    border-radius: 6px;
    padding: 6px 10px;
    font-size: 11px;
    color: #555;
  }

  .state {
    padding: 20px 0;
    text-align: center;
    color: #888;
    font-size: 13px;
  }
  .state.error { color: #d32f2f; }

  .month-trigger {
    width: 100%;
    background: linear-gradient(90deg, #003D5C 0%, #0077BE 100%);
    color: #fff;
    border: none;
    border-radius: 10px;
    padding: 8px 12px;
    font-size: 12px;
    font-weight: 600;
    cursor: pointer;
    margin-bottom: 10px;
    font-family: inherit;
    transition: filter 0.15s, transform 0.15s;
  }
  .month-trigger:hover { filter: brightness(1.1); }
  .month-trigger:active { transform: scale(0.98); }
</style>
