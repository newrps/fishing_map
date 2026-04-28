export type Region = '서해' | '남해' | '동해' | '제주';

export interface Station {
  code: string;
  name: string;
  lat: number;
  lon: number;
  region: Region;
}

export interface TideEvent {
  time: string;       // "HH:MM"
  datetime: string;   // "YYYY-MM-DD HH:MM"
  level_cm: number;
  kind: 'high' | 'low' | string;
}

export interface TideResponse {
  obs_code: string;
  obs_name: string;
  date: string;       // YYYYMMDD
  lat: number;
  lon: number;
  events: TideEvent[];
}

export interface Conditions {
  temperature: number | null;
  humidity: number | null;
  weather_code: number | null;
  weather_label: string;
  weather_emoji: string;
  wind_speed: number | null;
  wind_direction: number | null;
  wind_label: string;
  sea_temperature: number | null;
  wave_height: number | null;
  wave_period: number | null;
  wave_direction: number | null;
  time: string;
  source: 'live' | 'forecast' | 'archive';
}

const BASE = '/api';

export async function fetchStations(): Promise<Station[]> {
  const res = await fetch(`${BASE}/stations`);
  if (!res.ok) throw new Error(`stations: ${res.status}`);
  return res.json();
}

export async function fetchTide(code: string, date?: string): Promise<TideResponse> {
  const url = date ? `${BASE}/tide/${code}/${date}` : `${BASE}/tide/${code}`;
  const res = await fetch(url);
  if (!res.ok) {
    const msg = await res.text();
    throw new Error(`tide: ${res.status} ${msg}`);
  }
  return res.json();
}

export async function fetchConditions(code: string, date?: string): Promise<Conditions> {
  const url = date ? `${BASE}/conditions/${code}/${date}` : `${BASE}/conditions/${code}`;
  const res = await fetch(url);
  if (!res.ok) {
    const msg = await res.text();
    throw new Error(`conditions: ${res.status} ${msg}`);
  }
  return res.json();
}

// 바다갈라짐 명소 7곳 — 가장 가까운 관측소 코드와 매핑
// 출처: 국립해양조사원 바다갈라짐 명소
export const SEA_PARTING_SPOTS = [
  { name: '진도 갯재길',   region: '전남', lat: 34.4068, lon: 126.3105, station: 'DT_0028' },
  { name: '무창포 석대도', region: '충남', lat: 36.2150, lon: 126.5085, station: 'DT_0025' },
  { name: '사도-추도',     region: '전남', lat: 34.6450, lon: 127.7833, station: 'DT_0016' },
  { name: '화도',          region: '충남', lat: 36.5478, lon: 126.3392, station: 'DT_0067' },
  { name: '실미도-무의도', region: '인천', lat: 37.3681, lon: 126.4078, station: 'DT_0093' },
  { name: '제부도',        region: '경기', lat: 37.1714, lon: 126.5853, region2: '서해', station: 'DT_0008' },
  { name: '소이작도',      region: '인천', lat: 37.2500, lon: 126.1166, station: 'DT_0065' }
] as const;

export function findSeaPartingSpot(stationCode: string) {
  return SEA_PARTING_SPOTS.find((s) => s.station === stationCode);
}

export function regionColor(region: Region): string {
  switch (region) {
    case '서해': return '#F39C12';
    case '남해': return '#27AE60';
    case '동해': return '#2980B9';
    case '제주': return '#8E44AD';
  }
}
