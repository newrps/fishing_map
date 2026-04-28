<script lang="ts">
  import { createEventDispatcher, onMount } from 'svelte';
  import type { Station, TideResponse, TideEvent, Conditions } from '$lib/api';
  import { tideClass, lunarDay } from '$lib/lunar';

  export let station: Station;
  export let selectedDate: string; // YYYYMMDD — 시작 월 결정
  export let ensureTide: (code: string, ymd: string) => Promise<TideResponse>;
  export let ensureConditions: (code: string, ymd: string) => Promise<Conditions>;

  const dispatch = createEventDispatcher<{ close: undefined; pick: string }>();

  type Cell = {
    ymd: string;
    date: Date;
    inMonth: boolean;
    tide: TideResponse | null;
    conditions: Conditions | null;
    loading: boolean;
    error: boolean;
  };

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
  function todayYMD(): string {
    return dateToYmd(new Date());
  }

  let viewYear = 0;
  let viewMonth = 0;
  let cells: Cell[] = [];
  let loadToken = 0; // 빠른 월 변경 시 스테일 응답 무시

  function buildCells(year: number, month: number): Cell[] {
    const first = new Date(year, month, 1);
    const startDow = first.getDay(); // 0=일
    const daysInMonth = new Date(year, month + 1, 0).getDate();
    const totalCells = Math.ceil((startDow + daysInMonth) / 7) * 7;
    const result: Cell[] = [];
    for (let i = 0; i < totalCells; i++) {
      const dayNum = i - startDow + 1;
      const d = new Date(year, month, dayNum);
      result.push({
        ymd: dateToYmd(d),
        date: d,
        inMonth: dayNum >= 1 && dayNum <= daysInMonth,
        tide: null,
        conditions: null,
        loading: dayNum >= 1 && dayNum <= daysInMonth,
        error: false
      });
    }
    return result;
  }

  function loadAll() {
    const token = ++loadToken;
    cells.forEach((c, idx) => {
      if (!c.inMonth) return;
      ensureTide(station.code, c.ymd)
        .then((t) => {
          if (token !== loadToken) return;
          cells[idx] = { ...cells[idx], tide: t, loading: false };
          cells = cells;
        })
        .catch(() => {
          if (token !== loadToken) return;
          cells[idx] = { ...cells[idx], loading: false, error: true };
          cells = cells;
        });
      // 날씨 이모지 — 실패해도 셀은 정상 표시
      ensureConditions(station.code, c.ymd)
        .then((cond) => {
          if (token !== loadToken) return;
          cells[idx] = { ...cells[idx], conditions: cond };
          cells = cells;
        })
        .catch(() => {/* silent */});
    });
  }

  function setView(year: number, month: number) {
    viewYear = year;
    viewMonth = month;
    cells = buildCells(year, month);
    loadAll();
  }

  function prevMonth() {
    const d = new Date(viewYear, viewMonth - 1, 1);
    setView(d.getFullYear(), d.getMonth());
  }
  function nextMonth() {
    const d = new Date(viewYear, viewMonth + 1, 1);
    setView(d.getFullYear(), d.getMonth());
  }
  function gotoCurrentMonth() {
    const t = new Date();
    setView(t.getFullYear(), t.getMonth());
  }

  function lowestLow(events: TideEvent[]): TideEvent | null {
    return events
      .filter((e) => e.kind === 'low')
      .reduce<TideEvent | null>(
        (acc, cur) => (acc === null || cur.level_cm < acc.level_cm ? cur : acc),
        null
      );
  }

  function pick(ymd: string) {
    dispatch('pick', ymd);
  }

  function close() {
    dispatch('close');
  }

  function onKey(e: KeyboardEvent) {
    if (e.key === 'Escape') close();
    else if (e.key === 'ArrowLeft') prevMonth();
    else if (e.key === 'ArrowRight') nextMonth();
  }

  onMount(() => {
    const d = ymdToDate(selectedDate);
    setView(d.getFullYear(), d.getMonth());
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  });

  $: today = todayYMD();
  $: monthLabel = `${viewYear}년 ${viewMonth + 1}월`;
  const DOW = ['일', '월', '화', '수', '목', '금', '토'];
</script>

<div class="backdrop" on:click={close} role="presentation"></div>
<div class="modal" role="dialog" aria-label="한 달 물때 보기">
  <header>
    <div class="title">
      <span class="station">{station.name}</span>
      <span class="region">{station.region}</span>
    </div>
    <button class="close" on:click={close} aria-label="닫기">✕</button>
  </header>

  <div class="month-nav">
    <button class="nav-btn" on:click={prevMonth} aria-label="이전 달">‹</button>
    <button class="month-label" on:click={gotoCurrentMonth} title="이번 달로">
      {monthLabel}
    </button>
    <button class="nav-btn" on:click={nextMonth} aria-label="다음 달">›</button>
  </div>

  <div class="dow-row">
    {#each DOW as d, i}
      <div class="dow" class:sun={i === 0} class:sat={i === 6}>{d}</div>
    {/each}
  </div>

  <div class="grid">
    {#each cells as cell}
      {@const ll = cell.tide ? lowestLow(cell.tide.events) : null}
      {@const lun = tideClass(lunarDay(cell.date), station.region)}
      {@const isToday = cell.ymd === today}
      {@const isSelected = cell.ymd === selectedDate}
      {@const dow = cell.date.getDay()}
      <button
        class="cell"
        class:out={!cell.inMonth}
        class:today={isToday}
        class:selected={isSelected}
        class:sun={dow === 0}
        class:sat={dow === 6}
        class:prime={lun.isFishingPrime && cell.inMonth}
        disabled={!cell.inMonth}
        on:click={() => cell.inMonth && pick(cell.ymd)}
        style={lun.badgeColor ? `--phase: ${lun.badgeColor}` : ''}
      >
        <div class="row1">
          <span class="day">{cell.date.getDate()}</span>
          {#if cell.inMonth && lun.day}
            <span class="lunar">음{lun.day}</span>
          {/if}
        </div>
        {#if cell.inMonth}
          <div class="phase-row">
            {#if lun.mul}
              <span class="phase-badge">{lun.mul}</span>
            {/if}
            {#if cell.conditions?.weather_emoji}
              <span class="wx-emoji" title={cell.conditions.weather_label}>{cell.conditions.weather_emoji}</span>
            {/if}
          </div>
          {#if cell.loading}
            <span class="placeholder">…</span>
          {:else if cell.error}
            <span class="placeholder err">—</span>
          {:else if ll}
            <span class="low-time">{ll.time}</span>
            <span class="low-cm" class:tiny={ll.level_cm < 50}>{ll.level_cm}cm</span>
          {/if}
        {/if}
      </button>
    {/each}
  </div>

  <footer class="legend">
    <span><i style="background:#E53935"></i>사리(6·7·8물)</span>
    <span><i style="background:#FB8C00"></i>어깨(3·4·5·9물)</span>
    <span><i style="background:#1E88E5"></i>조금</span>
    <span><i style="background:#64B5F6"></i>무시</span>
    <span class="region-note">
      {#if station.region === '남해' || station.region === '제주'}
        ※ {station.region} 표기 (서해 -1일)
      {:else if station.region === '동해'}
        ※ 동해 조차 작음 — 참고용
      {:else}
        ※ 서해 14물때식
      {/if}
    </span>
  </footer>
</div>

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 30, 50, 0.55);
    z-index: 1000;
    backdrop-filter: blur(2px);
  }
  .modal {
    position: fixed;
    z-index: 1001;
    left: 50%;
    top: 50%;
    transform: translate(-50%, -50%);
    background: #fff;
    border-radius: 16px;
    box-shadow: 0 12px 48px rgba(0, 61, 92, 0.35);
    width: min(720px, 96vw);
    max-height: 92vh;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }

  header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px 10px;
    border-bottom: 1px solid #f0f0f0;
  }
  .title { display: flex; align-items: baseline; gap: 8px; }
  .station { font-size: 18px; font-weight: 700; color: #003D5C; }
  .region {
    background: #E3F2FD;
    color: #0077BE;
    padding: 2px 8px;
    border-radius: 10px;
    font-size: 11px;
    font-weight: 500;
  }
  .close {
    background: transparent;
    border: none;
    font-size: 18px;
    color: #666;
    cursor: pointer;
    padding: 4px 8px;
    border-radius: 6px;
  }
  .close:hover { background: #f4f4f4; }

  .month-nav {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 14px;
    padding: 10px 0;
    background: #f8fafc;
  }
  .nav-btn {
    background: transparent;
    border: none;
    font-size: 24px;
    color: #003D5C;
    cursor: pointer;
    width: 36px;
    height: 36px;
    border-radius: 8px;
  }
  .nav-btn:hover { background: #e3edf4; }
  .month-label {
    background: transparent;
    border: none;
    font-size: 17px;
    font-weight: 700;
    color: #003D5C;
    cursor: pointer;
    padding: 4px 12px;
    border-radius: 8px;
    min-width: 140px;
  }
  .month-label:hover { background: #e3edf4; }

  .dow-row {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    padding: 4px 8px 0;
    background: #f8fafc;
  }
  .dow {
    text-align: center;
    font-size: 12px;
    font-weight: 600;
    color: #555;
    padding: 6px 0;
  }
  .dow.sun { color: #E53935; }
  .dow.sat { color: #1E88E5; }

  .grid {
    display: grid;
    grid-template-columns: repeat(7, 1fr);
    gap: 4px;
    padding: 8px;
    background: #fff;
    overflow-y: auto;
  }
  .cell {
    background: #fff;
    border: 1.5px solid #eef2f5;
    border-radius: 8px;
    padding: 6px 4px;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    min-height: 78px;
    text-align: center;
    transition: all 0.12s;
    font-family: inherit;
    position: relative;
  }
  .cell:not(:disabled):hover {
    background: #f4f9fc;
    border-color: #bcd5e3;
    transform: translateY(-1px);
  }
  .cell.out { background: #fafafa; cursor: default; opacity: 0.4; }
  .cell.today {
    background: #FFF8E1;
    border-color: #FFB800;
  }
  .cell.selected {
    border-color: #003D5C;
    border-width: 2px;
  }
  .cell.prime {
    box-shadow: inset 3px 0 0 var(--phase, transparent);
  }

  .row1 {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    width: 100%;
    padding: 0 2px;
  }
  .day { font-size: 14px; font-weight: 600; color: #333; }
  .cell.sun .day { color: #E53935; }
  .cell.sat .day { color: #1E88E5; }
  .lunar { font-size: 9px; color: #999; }

  .phase-row {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 4px;
  }
  .phase-badge {
    font-size: 10px;
    font-weight: 600;
    color: #fff;
    background: var(--phase, #ccc);
    padding: 1px 6px;
    border-radius: 8px;
    line-height: 1.4;
  }
  .wx-emoji {
    font-size: 14px;
    line-height: 1;
  }
  .low-time {
    font-family: ui-monospace, monospace;
    font-size: 11px;
    color: #444;
    margin-top: auto;
  }
  .low-cm {
    font-size: 10px;
    color: #888;
  }
  .low-cm.tiny { color: #FB8C00; font-weight: 600; }
  .placeholder { font-size: 11px; color: #ccc; }
  .placeholder.err { color: #d32f2f; }

  footer.legend {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
    padding: 8px 12px;
    border-top: 1px solid #f0f0f0;
    background: #f8fafc;
    font-size: 11px;
    color: #555;
  }
  footer.legend span {
    display: inline-flex;
    align-items: center;
    gap: 4px;
  }
  footer.legend i {
    width: 10px;
    height: 10px;
    border-radius: 3px;
    display: inline-block;
  }
  footer .hint {
    margin-left: auto;
    color: #888;
    font-size: 10px;
  }
  footer .region-note {
    margin-left: auto;
    color: #555;
    font-size: 10px;
    font-weight: 600;
  }

  @media (max-width: 600px) {
    .modal { width: 100vw; max-height: 100vh; height: 100vh; border-radius: 0; }
    .cell { min-height: 64px; padding: 4px 2px; }
    .day { font-size: 13px; }
    .lunar { font-size: 8px; }
    .phase-badge { font-size: 9px; padding: 1px 4px; }
    .low-time, .low-cm { font-size: 9px; }
    footer.legend { font-size: 10px; gap: 6px; }
    footer .hint { display: none; }
  }
</style>
