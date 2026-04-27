<script lang="ts">
  import type { Station, TideResponse, TideEvent, Conditions } from '$lib/api';
  import { findSeaPartingSpot } from '$lib/api';

  export let station: Station | null = null;
  export let tide: TideResponse | null = null;
  export let conditions: Conditions | null = null;
  export let loading = false;
  export let error: string | null = null;

  function formatDate(d: string): string {
    if (!d || d.length !== 8) return d;
    return `${d.slice(0, 4)}-${d.slice(4, 6)}-${d.slice(6, 8)}`;
  }

  function kindLabel(e: TideEvent): string {
    return e.kind === 'high' ? '만조' : e.kind === 'low' ? '간조' : e.kind;
  }

  function kindColor(e: TideEvent): string {
    return e.kind === 'high' ? '#1E88E5' : '#EF6C00';
  }

  // 바다갈라짐 가능성 (간조 수위 < 30cm 일 때)
  function getSeaPartingChance(t: TideResponse | null): TideEvent | null {
    if (!t) return null;
    const lows = t.events.filter((e) => e.kind === 'low');
    const lowest = lows.reduce<TideEvent | null>(
      (acc, cur) => (acc === null || cur.level_cm < acc.level_cm ? cur : acc),
      null
    );
    return lowest && lowest.level_cm < 30 ? lowest : null;
  }

  $: seaPartingSpot = station ? findSeaPartingSpot(station.code) : null;
  $: lowestLow = getSeaPartingChance(tide);
</script>

<div class="panel">
  {#if station}
    <header>
      <div>
        <h2>{station.name}</h2>
        <span class="region">{station.region}</span>
      </div>
      <small>{station.code}</small>
    </header>

    {#if loading}
      <div class="state">정보 가져오는 중…</div>
    {:else if error}
      <div class="state error">⚠️ {error}</div>
    {:else}
      <!-- 현재 상태 (날씨 + 수온) -->
      {#if conditions}
        <section class="now">
          <div class="now-row">
            <div class="now-cell weather">
              <span class="emoji">{conditions.weather_emoji}</span>
              <div class="cell-text">
                <span class="big">
                  {conditions.temperature != null ? conditions.temperature.toFixed(1) + '°' : '—'}
                </span>
                <span class="small">{conditions.weather_label}</span>
              </div>
            </div>
            <div class="now-cell sea">
              <span class="emoji">🌊</span>
              <div class="cell-text">
                <span class="big">
                  {conditions.sea_temperature != null
                    ? conditions.sea_temperature.toFixed(1) + '°'
                    : '—'}
                </span>
                <span class="small">수온</span>
              </div>
            </div>
          </div>

          <div class="meta-row">
            <span class="meta-item">
              💨 {conditions.wind_label} {conditions.wind_speed?.toFixed(0) ?? '—'} km/h
            </span>
            <span class="meta-item">
              🌊 파고 {conditions.wave_height != null ? conditions.wave_height.toFixed(1) + 'm' : '—'}
            </span>
            <span class="meta-item">
              💧 습도 {conditions.humidity ?? '—'}%
            </span>
          </div>
        </section>
      {/if}

      <!-- 바다갈라짐 -->
      {#if seaPartingSpot && lowestLow}
        <section class="sea-parting">
          <div class="sp-header">
            <span class="sp-icon">🌊</span>
            <strong>바다갈라짐 명소</strong>
          </div>
          <div class="sp-name">{seaPartingSpot.name} ({seaPartingSpot.region})</div>
          <div class="sp-info">
            오늘 <strong>{lowestLow.time}</strong> 무렵 간조 ({lowestLow.level_cm}cm)
            <br>
            <span class="hint">→ 갈라짐 현상 가능성 ✨</span>
          </div>
        </section>
      {:else if seaPartingSpot}
        <section class="sea-parting muted">
          <div class="sp-header">
            <span class="sp-icon">🌊</span>
            <strong>바다갈라짐 명소</strong>
          </div>
          <div class="sp-name">{seaPartingSpot.name}</div>
          <div class="sp-info hint">오늘은 간조 수위가 높아 갈라짐 어려움</div>
        </section>
      {/if}

      <!-- 만조/간조 -->
      {#if tide}
        <section>
          <div class="section-title">물때 · {formatDate(tide.date)}</div>
          <ul class="events">
            {#each tide.events as e}
              <li>
                <span class="kind" style="color: {kindColor(e)}">{kindLabel(e)}</span>
                <span class="time">{e.time}</span>
                <span class="level">{e.level_cm} cm</span>
              </li>
            {/each}
          </ul>
        </section>

        <div class="footer-meta">
          위도 {tide.lat.toFixed(4)} · 경도 {tide.lon.toFixed(4)}
        </div>
      {/if}
    {/if}
  {:else}
    <div class="placeholder">
      <p>왼쪽 지도에서 관측소를 선택하세요.</p>
      <p class="hint">전국 63개소 · 물때 · 날씨 · 수온 · 파고 · 바다갈라짐</p>
    </div>
  {/if}
</div>

<style>
  .panel {
    background: #fff;
    border-radius: 16px;
    padding: 20px;
    box-shadow: 0 4px 24px rgba(0, 61, 92, 0.12);
    min-height: 240px;
    display: flex;
    flex-direction: column;
    gap: 14px;
  }

  header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
  }
  header h2 {
    font-size: 22px;
    color: #003D5C;
    margin: 0 0 6px 0;
  }
  .region {
    display: inline-block;
    background: #E3F2FD;
    color: #0077BE;
    padding: 2px 10px;
    border-radius: 12px;
    font-size: 12px;
    font-weight: 500;
  }
  header small {
    color: #999;
    font-family: ui-monospace, monospace;
    font-size: 12px;
  }

  /* 현재 상태 */
  .now {
    background: linear-gradient(135deg, #E3F2FD 0%, #F3E5F5 100%);
    border-radius: 12px;
    padding: 14px 12px;
  }
  .now-row {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    margin-bottom: 10px;
  }
  .now-cell {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 8px 10px;
    background: rgba(255,255,255,0.7);
    border-radius: 10px;
  }
  .now-cell .emoji { font-size: 28px; line-height: 1; }
  .cell-text { display: flex; flex-direction: column; }
  .cell-text .big { font-size: 22px; font-weight: 600; color: #003D5C; line-height: 1.1; }
  .cell-text .small { font-size: 11px; color: #666; }

  .meta-row {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    font-size: 12px;
    color: #444;
  }
  .meta-item {
    background: rgba(255,255,255,0.6);
    padding: 4px 10px;
    border-radius: 12px;
  }

  /* 바다갈라짐 */
  .sea-parting {
    background: #FFF8E1;
    border-left: 3px solid #FFB800;
    border-radius: 8px;
    padding: 10px 14px;
  }
  .sea-parting.muted {
    background: #f5f5f5;
    border-left-color: #ccc;
  }
  .sp-header {
    display: flex;
    align-items: center;
    gap: 6px;
    color: #003D5C;
    font-size: 13px;
    margin-bottom: 4px;
  }
  .sp-icon { font-size: 16px; }
  .sp-name {
    font-size: 14px;
    color: #333;
    margin-bottom: 4px;
  }
  .sp-info {
    font-size: 12px;
    color: #555;
    line-height: 1.5;
  }
  .hint { color: #888; }

  /* 만조/간조 */
  .section-title {
    font-size: 12px;
    color: #6b7c8a;
    margin-bottom: 6px;
    font-weight: 500;
  }
  .events {
    list-style: none;
    padding: 0;
    margin: 0;
  }
  .events li {
    display: grid;
    grid-template-columns: 50px 70px 1fr;
    align-items: center;
    padding: 8px 0;
    border-bottom: 1px solid #f0f0f0;
    font-size: 14px;
  }
  .events li:last-child { border-bottom: none; }
  .kind { font-weight: 600; font-size: 13px; }
  .time { font-family: ui-monospace, monospace; font-size: 15px; color: #333; }
  .level { color: #666; text-align: right; font-size: 13px; }

  .footer-meta {
    color: #999;
    font-size: 11px;
    padding-top: 4px;
  }

  .state {
    padding: 30px 0;
    text-align: center;
    color: #888;
  }
  .state.error { color: #d32f2f; }

  .placeholder p {
    margin: 0;
    color: #888;
  }
  .placeholder p:first-child {
    font-size: 16px;
    color: #003D5C;
    margin-bottom: 8px;
  }
  .placeholder .hint { font-size: 13px; }
</style>
