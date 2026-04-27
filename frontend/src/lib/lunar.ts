// 양력 → 음력 변환 (Intl.DateTimeFormat의 chinese 캘린더 활용)
// 한국 음력은 중국 음력과 동일한 계산 방식이라 그대로 사용 가능.

export interface LunarInfo {
  day: number;          // 음력 1~30
  label: string;        // "사리" | "조금" | "1물" ~ "8물" | ""
  isFishingPrime: boolean; // 사리 = 낚시 좋은 시기
  badgeColor: string;
}

const lunarFmt = new Intl.DateTimeFormat('en-u-ca-chinese', { day: 'numeric' });

export function lunarDay(date: Date): number {
  // chinese calendar는 1~30 숫자로 음력일 반환
  const parts = lunarFmt.formatToParts(date);
  const dayPart = parts.find((p) => p.type === 'day');
  return dayPart ? parseInt(dayPart.value, 10) : 0;
}

// 8물때식 (서해/한국 표준)
// 음력 1,16 = 7물 / 음력 2,17 = 8물(대사리) / 음력 8,23 = 조금 / 음력 9,24 = 무시
// 사리 = 보름·그믐 전후 (대조), 조금 = 사리 반대편 (소조)
export function tideClass(lunarD: number): LunarInfo {
  // 음력 30일이 없는 달이면 lunarD == 0; 안전하게 처리
  if (!lunarD) {
    return { day: 0, label: '', isFishingPrime: false, badgeColor: '' };
  }

  // 보름·그믐 전후 (사리 대조 5일 윈도우)
  const isSari =
    [29, 30, 1, 2, 3, 14, 15, 16, 17, 18].includes(lunarD);
  // 상현·하현 전후 (조금 소조 4일 윈도우)
  const isJogeum = [7, 8, 9, 10, 22, 23, 24, 25].includes(lunarD);

  let label = '';
  let badgeColor = '';
  if (isSari) {
    // 사리 정점 (음력 1, 2, 16, 17)은 강조
    if ([1, 2, 16, 17].includes(lunarD)) {
      label = '사리';
      badgeColor = '#E53935'; // 빨강 - 대조
    } else {
      label = '큰사리';
      badgeColor = '#FB8C00'; // 주황
    }
  } else if (isJogeum) {
    if ([8, 23].includes(lunarD)) {
      label = '조금';
      badgeColor = '#1E88E5'; // 파랑 - 소조
    } else {
      label = '소조';
      badgeColor = '#64B5F6'; // 연파랑
    }
  }

  return {
    day: lunarD,
    label,
    isFishingPrime: isSari,
    badgeColor
  };
}

export function getLunarInfo(date: Date): LunarInfo {
  return tideClass(lunarDay(date));
}
