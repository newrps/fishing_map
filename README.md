# 🎣 물때 지도 (fishing-map)

전국 조위관측소 만조·간조 정보 + Leaflet 지도. KHOA OpenAPI 사용.

## 구성

```
backend/   Rust (axum) — KHOA API 프록시 + CORS
frontend/  SvelteKit + Leaflet + OpenSeaMap
```

## 실행

### 1. 백엔드

```bash
cd backend
# .env 파일에 KHOA_SERVICE_KEY 채우기
cargo run
# → http://127.0.0.1:8765
```

엔드포인트
- `GET /api/health`
- `GET /api/stations` — 전국 50여 개 조위관측소 목록
- `GET /api/tide/:code` — 오늘 만조·간조 (예: `/api/tide/DT_0001`)
- `GET /api/tide/:code/:date` — 특정 날짜 (date=YYYYMMDD)

### 2. 프론트엔드

```bash
cd frontend
npm install
npm run dev
# → http://localhost:5173
```

Vite proxy가 `/api/*` → `http://127.0.0.1:8765`로 자동 전달.

## KHOA 키 발급

1. https://data.go.kr 접속 후 카카오/네이버로 로그인
2. "국립해양조사원 조석예보" 검색
3. 활용신청 → 즉시 자동승인
4. 마이페이지 → 오픈API → 인증키 (Encoding 버전) 복사
5. `backend/.env`의 `KHOA_SERVICE_KEY=`에 붙여넣기

## API 응답 형태

```json
{
  "obs_code": "DT_0001",
  "obs_name": "인천",
  "date": "20260427",
  "lat": 37.45194,
  "lon": 126.59222,
  "events": [
    { "time": "01:32", "datetime": "...", "level_cm": 639, "kind": "high" },
    { "time": "07:43", "datetime": "...", "level_cm": 225, "kind": "low" }
  ]
}
```

`kind`는 인접 조위 비교로 자동 결정 (`high` = 만조, `low` = 간조).
