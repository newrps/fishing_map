// 양력 → 음력 변환 + 한국 물때 시스템(7물때식/8물때식) 표기
// 출처 비교: 바다타임 물때표 가이드 (https://www.badatime.com/faq.jsp)

import type { Region } from './api';

export interface LunarInfo {
  day: number;             // 음력 1~30
  mul: string;             // "1물"~"14물" | "조금" | "무시"
  isFishingPrime: boolean; // 보름·그믐 ±2일 (사리 큰물)
  badgeColor: string;
  system: TideSystem;
}

export type TideSystem = '7물때식' | '8물때식';
type Tier = 'sari' | 'shoulder' | 'jogeum' | 'mushi' | 'normal';

const lunarFmt = new Intl.DateTimeFormat('en-u-ca-chinese', { day: 'numeric' });

export function lunarDay(date: Date): number {
  const parts = lunarFmt.formatToParts(date);
  const dayPart = parts.find((p) => p.type === 'day');
  return dayPart ? parseInt(dayPart.value, 10) : 0;
}

// 지역별 사용 시스템 — 바다타임 기준
//  · 서해: 7물때식 (음 1=7물, 음 9=무시)
//  · 남해/동해/제주: 8물때식 (음 1=8물, 음 9=1물 — 무시 없음)
export const REGION_SYSTEM: Record<Region, TideSystem> = {
  '서해': '7물때식',
  '남해': '8물때식',
  '제주': '8물때식',
  '동해': '8물때식', // 조차 작아 의미 약함, 형식상 8물때식 분류
};

// 인덱스 0=음1, 1=음2, …, 14=음15 (그리고 음 16~30 동일 반복)
const TABLE_7: ReadonlyArray<{ mul: string; tier: Tier }> = [
  { mul: '7물',  tier: 'sari' },     // 0  음 1, 16
  { mul: '8물',  tier: 'sari' },     // 1  음 2, 17
  { mul: '9물',  tier: 'shoulder' }, // 2  음 3, 18
  { mul: '10물', tier: 'normal' },   // 3
  { mul: '11물', tier: 'normal' },   // 4
  { mul: '12물', tier: 'normal' },   // 5
  { mul: '13물', tier: 'normal' },   // 6
  { mul: '조금', tier: 'jogeum' },   // 7  음 8, 23
  { mul: '무시', tier: 'mushi' },    // 8  음 9, 24
  { mul: '1물',  tier: 'normal' },   // 9
  { mul: '2물',  tier: 'normal' },   // 10
  { mul: '3물',  tier: 'normal' },   // 11
  { mul: '4물',  tier: 'shoulder' }, // 12
  { mul: '5물',  tier: 'shoulder' }, // 13
  { mul: '6물',  tier: 'sari' },     // 14 음 15, 30 (보름·그믐)
];

const TABLE_8: ReadonlyArray<{ mul: string; tier: Tier }> = [
  { mul: '8물',  tier: 'sari' },     // 0  음 1, 16
  { mul: '9물',  tier: 'sari' },     // 1  음 2, 17
  { mul: '10물', tier: 'shoulder' }, // 2
  { mul: '11물', tier: 'normal' },   // 3
  { mul: '12물', tier: 'normal' },   // 4
  { mul: '13물', tier: 'normal' },   // 5
  { mul: '14물', tier: 'normal' },   // 6
  { mul: '조금', tier: 'jogeum' },   // 7  음 8, 23
  { mul: '1물',  tier: 'normal' },   // 8  음 9, 24 (무시 없음)
  { mul: '2물',  tier: 'normal' },   // 9
  { mul: '3물',  tier: 'normal' },   // 10
  { mul: '4물',  tier: 'normal' },   // 11
  { mul: '5물',  tier: 'shoulder' }, // 12
  { mul: '6물',  tier: 'shoulder' }, // 13
  { mul: '7물',  tier: 'sari' },     // 14 음 15, 30 (보름·그믐)
];

const TIER_COLOR: Record<Tier, string> = {
  sari:     '#E53935',
  shoulder: '#FB8C00',
  jogeum:   '#1E88E5',
  mushi:    '#64B5F6',
  normal:   '#9E9E9E',
};

export function tideClass(lunarD: number, region: Region = '서해'): LunarInfo {
  const system = REGION_SYSTEM[region];
  if (!lunarD) {
    return { day: 0, mul: '', isFishingPrime: false, badgeColor: '', system };
  }
  const idx = (lunarD - 1) % 15;
  const table = system === '7물때식' ? TABLE_7 : TABLE_8;
  const m = table[idx];
  return {
    day: lunarD,
    mul: m.mul,
    isFishingPrime: m.tier === 'sari',
    badgeColor: TIER_COLOR[m.tier],
    system,
  };
}

export function getLunarInfo(date: Date, region: Region = '서해'): LunarInfo {
  return tideClass(lunarDay(date), region);
}
