<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { getLunarInfo } from '$lib/lunar';

  export let selectedDate: string; // YYYYMMDD
  export let daysAhead: number = 60; // 두 달치 기본 표시

  const dispatch = createEventDispatcher<{ select: string }>();

  const DOW = ['일', '월', '화', '수', '목', '금', '토'];

  function toYMD(d: Date): string {
    const y = d.getFullYear();
    const m = String(d.getMonth() + 1).padStart(2, '0');
    const day = String(d.getDate()).padStart(2, '0');
    return `${y}${m}${day}`;
  }

  function ymdEqual(d: Date, ymd: string): boolean {
    return toYMD(d) === ymd;
  }

  // 오늘부터 N일치
  $: dates = Array.from({ length: daysAhead }, (_, i) => {
    const d = new Date();
    d.setHours(0, 0, 0, 0);
    d.setDate(d.getDate() + i);
    return d;
  });

  $: today = new Date();

  function pick(d: Date) {
    dispatch('select', toYMD(d));
  }

  let scrollEl: HTMLDivElement;

  // 선택된 날짜로 스크롤 위치 자동 조정
  $: if (scrollEl && selectedDate) {
    const idx = dates.findIndex((d) => ymdEqual(d, selectedDate));
    if (idx >= 0) {
      const cell = scrollEl.children[idx] as HTMLElement | undefined;
      cell?.scrollIntoView({ behavior: 'smooth', block: 'nearest', inline: 'nearest' });
    }
  }
</script>

<div class="strip-wrap">
  <div class="strip" bind:this={scrollEl}>
    {#each dates as d, i}
      {@const ymd = toYMD(d)}
      {@const isToday = ymdEqual(today, ymd)}
      {@const isSelected = ymd === selectedDate}
      {@const lunar = getLunarInfo(d)}
      <button
        class="cell"
        class:today={isToday}
        class:selected={isSelected}
        on:click={() => pick(d)}
      >
        <span class="month">{d.getMonth() + 1}월</span>
        <span class="day">{d.getDate()}</span>
        <span class="dow" class:sun={d.getDay() === 0} class:sat={d.getDay() === 6}>
          {DOW[d.getDay()]}
        </span>
        {#if lunar.label}
          <span class="badge" style="background: {lunar.badgeColor}">{lunar.label}</span>
        {:else if lunar.day}
          <span class="lunar-day">음 {lunar.day}</span>
        {/if}
      </button>
    {/each}
  </div>
  <div class="hint-row">
    <span class="hint">← 좌우 스크롤로 더 많은 날짜 보기 ({daysAhead}일치)</span>
  </div>
</div>

<style>
  .strip-wrap {
    background: #fff;
    border-radius: 12px;
    padding: 10px 0 6px 0;
    box-shadow: 0 2px 8px rgba(0, 61, 92, 0.06);
  }

  .strip {
    display: flex;
    gap: 6px;
    overflow-x: auto;
    overflow-y: hidden;
    scroll-snap-type: x mandatory;
    padding: 4px 10px;
    /* 7개 셀 보이도록 */
  }
  .strip::-webkit-scrollbar {
    height: 4px;
  }
  .strip::-webkit-scrollbar-thumb {
    background: rgba(0, 119, 190, 0.3);
    border-radius: 2px;
  }

  .cell {
    flex: 0 0 calc((100% - 36px) / 7); /* 7개 + gap */
    min-width: 44px;
    scroll-snap-align: start;
    background: #f4f7fa;
    border: 1.5px solid transparent;
    border-radius: 10px;
    padding: 6px 4px 8px 4px;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 2px;
    transition: all 0.15s;
    font-family: inherit;
  }
  .cell:hover {
    background: #e8eef3;
  }
  .cell.today {
    background: #fff8e1;
    border-color: #FFB800;
  }
  .cell.selected {
    background: #003D5C;
    border-color: #003D5C;
    color: #fff;
    transform: scale(1.05);
  }

  .month {
    font-size: 10px;
    color: #999;
    font-weight: 500;
  }
  .selected .month { color: rgba(255,255,255,0.7); }

  .day {
    font-size: 18px;
    font-weight: 700;
    color: #003D5C;
    line-height: 1;
  }
  .selected .day { color: #fff; }

  .dow {
    font-size: 11px;
    color: #6b7c8a;
    font-weight: 500;
  }
  .selected .dow { color: rgba(255,255,255,0.85); }
  .dow.sun { color: #d32f2f; }
  .dow.sat { color: #1976d2; }
  .selected .dow.sun, .selected .dow.sat { color: #fff; }

  .badge {
    font-size: 9px;
    color: #fff;
    padding: 1px 6px;
    border-radius: 8px;
    font-weight: 600;
    margin-top: 2px;
    letter-spacing: -0.3px;
  }

  .lunar-day {
    font-size: 9px;
    color: #aaa;
    margin-top: 2px;
  }
  .selected .lunar-day { color: rgba(255,255,255,0.6); }

  .hint-row {
    padding: 2px 14px 0 14px;
    text-align: center;
  }
  .hint {
    font-size: 10px;
    color: #aaa;
  }
</style>
