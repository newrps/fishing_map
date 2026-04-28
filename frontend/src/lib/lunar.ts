// 양력 → 음력 변환 (Intl.DateTimeFormat의 chinese 캘린더 활용)
// 한국 음력은 중국 음력과 동일한 계산 방식이라 그대로 사용 가능.

import type { Region } from './api';

export interface LunarInfo {
  day: number;             // 음력 1~30
  mul: string;             // "1물"~"13물" | "조금" | "무시"
  isFishingPrime: boolean; // 사리(큰물) = 6,7,8물
  badgeColor: string;      // tier별 색상
  region?: Region;
}

const lunarFmt = new Intl.DateTimeFormat('en-u-ca-chinese', { day: 'numeric' });

export function lunarDay(date: Date): number {
  const parts = lunarFmt.formatToParts(date);
  const dayPart = parts.find((p) => p.type === 'day');
  return dayPart ? parseInt(dayPart.value, 10) : 0;
}

// 14물때식 (서해 표준 기준 테이블) — 음력 1일을 7물로 시작
type Tier = 'sari' | 'shoulder' | 'jogeum' | 'mushi' | 'normal';

const MUL_TABLE: ReadonlyArray<{ mul: string; tier: Tier }> = [
  { mul: '7물',  tier: 'sari' },     // 음 1, 16
  { mul: '8물',  tier: 'sari' },     // 음 2, 17  ← 한사리(정점)
  { mul: '9물',  tier: 'shoulder' }, // 음 3, 18
  { mul: '10물', tier: 'normal' },   // 음 4, 19
  { mul: '11물', tier: 'normal' },   // 음 5, 20
  { mul: '12물', tier: 'normal' },   // 음 6, 21
  { mul: '13물', tier: 'normal' },   // 음 7, 22
  { mul: '조금', tier: 'jogeum' },   // 음 8, 23
  { mul: '무시', tier: 'mushi' },    // 음 9, 24
  { mul: '1물',  tier: 'normal' },   // 음 10, 25
  { mul: '2물',  tier: 'normal' },   // 음 11, 26
  { mul: '3물',  tier: 'normal' },   // 음 12, 27
  { mul: '4물',  tier: 'shoulder' }, // 음 13, 28
  { mul: '5물',  tier: 'shoulder' }, // 음 14, 29
  { mul: '6물',  tier: 'sari' },     // 음 15, 30
];

const TIER_COLOR: Record<Tier, string> = {
  sari:     '#E53935',
  shoulder: '#FB8C00',
  jogeum:   '#1E88E5',
  mushi:    '#64B5F6',
  normal:   '#9E9E9E',
};

// 지역별 시프트 (음력일 기준 days)
// 남해/제주는 서해보다 같은 음력일에 1단계 빠른(시프트 +1) 물때 표기
//  (동일 mul 라벨이 서해보다 1일 늦게 등장)
const REGION_SHIFT: Record<Region, number> = {
  '서해': 0,
  '남해': 1,
  '제주': 1,
  '동해': 0, // 조차 작음 — 형식상 서해 표준 적용
};

export function tideClass(lunarD: number, region: Region = '서해'): LunarInfo {
  if (!lunarD) {
    return { day: 0, mul: '', isFishingPrime: false, badgeColor: '', region };
  }
  const shift = REGION_SHIFT[region] ?? 0;
  const idx = ((lunarD - 1 - shift) % 15 + 15) % 15;
  const m = MUL_TABLE[idx];
  return {
    day: lunarD,
    mul: m.mul,
    isFishingPrime: m.tier === 'sari',
    badgeColor: TIER_COLOR[m.tier],
    region,
  };
}

export function getLunarInfo(date: Date, region: Region = '서해'): LunarInfo {
  return tideClass(lunarDay(date), region);
}
