// All Korean tide observation stations (KHOA 조위관측소).
// Auto-discovered by scanning DT_0001..DT_0250 against the live KHOA API.
// Region classified by longitude/latitude: 서해(lon<127, lat>33.5),
// 남해(lon>=127 & lat<37 & lon<130), 동해(lon>=129 & lat>=35), 제주(lat<33.5).

pub struct Station {
    pub code: &'static str,
    pub name: &'static str,
    pub lat: f64,
    pub lon: f64,
    pub region: &'static str,
}

pub const ALL: &[Station] = &[
    // ── 서해 ────────────────────────────────────────────
    Station { code: "DT_0059", name: "백령도",     lat: 37.95565, lon: 124.73608, region: "서해" },
    Station { code: "DT_0036", name: "대청도",     lat: 37.82522, lon: 124.71805, region: "서해" },
    Station { code: "DT_0064", name: "교동대교",   lat: 37.78961, lon: 126.33961, region: "서해" },
    Station { code: "DT_0032", name: "강화대교",   lat: 37.73194, lon: 126.52222, region: "서해" },
    Station { code: "DT_0060", name: "연평도",     lat: 37.65766, lon: 125.71441, region: "서해" },
    Station { code: "DT_0058", name: "경인항",     lat: 37.56083, lon: 126.60111, region: "서해" },
    Station { code: "DT_0044", name: "영종대교",   lat: 37.54555, lon: 126.58444, region: "서해" },
    Station { code: "DT_0001", name: "인천",       lat: 37.45194, lon: 126.59222, region: "서해" },
    Station { code: "DT_0093", name: "소무의도",   lat: 37.37306, lon: 126.44006, region: "서해" },
    Station { code: "DT_0052", name: "인천송도",   lat: 37.33805, lon: 126.58611, region: "서해" },
    Station { code: "DT_0043", name: "영흥도",     lat: 37.23861, lon: 126.42861, region: "서해" },
    Station { code: "DT_0065", name: "덕적도",     lat: 37.22633, lon: 126.15655, region: "서해" },
    Station { code: "DT_0038", name: "굴업도",     lat: 37.19444, lon: 125.99500, region: "서해" },
    Station { code: "DT_0008", name: "안산",       lat: 37.19222, lon: 126.64722, region: "서해" },
    Station { code: "DT_0017", name: "대산",       lat: 37.00750, lon: 126.35277, region: "서해" },
    Station { code: "DT_0002", name: "평택",       lat: 36.96694, lon: 126.82277, region: "서해" },
    Station { code: "DT_0050", name: "태안",       lat: 36.91305, lon: 126.23888, region: "서해" },
    Station { code: "DT_0067", name: "안흥",       lat: 36.67463, lon: 126.12955, region: "서해" },
    Station { code: "DT_0025", name: "보령",       lat: 36.40638, lon: 126.48611, region: "서해" },
    Station { code: "DT_0051", name: "서천마량",   lat: 36.12888, lon: 126.49527, region: "서해" },
    Station { code: "DT_0037", name: "어청도",     lat: 36.11722, lon: 125.98472, region: "서해" },
    Station { code: "DT_0024", name: "장항",       lat: 36.00694, lon: 126.68750, region: "서해" },
    Station { code: "DT_0018", name: "군산",       lat: 35.97555, lon: 126.56305, region: "서해" },
    Station { code: "DT_0068", name: "위도",       lat: 35.61808, lon: 126.30181, region: "서해" },
    Station { code: "DT_0003", name: "영광",       lat: 35.42611, lon: 126.42055, region: "서해" },
    Station { code: "DT_0066", name: "향화도",     lat: 35.16766, lon: 126.35955, region: "서해" },
    Station { code: "DT_0007", name: "목포",       lat: 34.77972, lon: 126.37555, region: "서해" },
    Station { code: "DT_0035", name: "흑산도",     lat: 34.68416, lon: 125.43555, region: "서해" },
    Station { code: "DT_0028", name: "진도",       lat: 34.37777, lon: 126.30861, region: "서해" },
    Station { code: "DT_0027", name: "완도",       lat: 34.31555, lon: 126.75972, region: "서해" },
    Station { code: "DT_0094", name: "서거차도",   lat: 34.25142, lon: 125.91544, region: "서해" },
    Station { code: "DT_0041", name: "복사초",     lat: 34.09833, lon: 126.16833, region: "서해" },

    // ── 남해 ────────────────────────────────────────────
    Station { code: "DT_0062", name: "마산",       lat: 35.19750, lon: 128.57638, region: "남해" },
    Station { code: "DT_0054", name: "진해",       lat: 35.14722, lon: 128.64305, region: "남해" },
    Station { code: "DT_0005", name: "부산",       lat: 35.09638, lon: 129.03527, region: "남해" },
    Station { code: "DT_0056", name: "부산항신항", lat: 35.07750, lon: 128.78472, region: "남해" },
    Station { code: "DT_0063", name: "가덕도",     lat: 35.02417, lon: 128.81093, region: "남해" },
    Station { code: "DT_0061", name: "삼천포",     lat: 34.92416, lon: 128.06972, region: "남해" },
    Station { code: "DT_0049", name: "광양",       lat: 34.90367, lon: 127.75483, region: "남해" },
    Station { code: "DT_0014", name: "통영",       lat: 34.82777, lon: 128.43472, region: "남해" },
    Station { code: "DT_0029", name: "거제도",     lat: 34.80138, lon: 128.69916, region: "남해" },
    Station { code: "DT_0016", name: "여수",       lat: 34.74722, lon: 127.76555, region: "남해" },
    Station { code: "DT_0042", name: "교본초",     lat: 34.70472, lon: 128.30638, region: "남해" },
    Station { code: "DT_0092", name: "여호항",     lat: 34.66194, lon: 127.46916, region: "남해" },
    Station { code: "DT_0026", name: "고흥발포",   lat: 34.48111, lon: 127.34277, region: "남해" },
    Station { code: "DT_0031", name: "거문도",     lat: 34.02833, lon: 127.30888, region: "남해" },

    // ── 동해 ────────────────────────────────────────────
    Station { code: "DT_0048", name: "속초등표",   lat: 38.19947, lon: 128.61308, region: "동해" },
    Station { code: "DT_0012", name: "속초",       lat: 38.20722, lon: 128.59416, region: "동해" },
    Station { code: "DT_0006", name: "묵호",       lat: 37.55027, lon: 129.11638, region: "동해" },
    Station { code: "DT_0046", name: "쌍정초",     lat: 37.55616, lon: 130.93921, region: "동해" },
    Station { code: "DT_0013", name: "울릉도",     lat: 37.49138, lon: 130.91361, region: "동해" },
    Station { code: "DT_0057", name: "동해항",     lat: 37.49472, lon: 129.14388, region: "동해" },
    Station { code: "DT_0040", name: "독도",       lat: 37.23888, lon: 131.86722, region: "동해" },
    Station { code: "DT_0039", name: "왕돌초",     lat: 36.71916, lon: 129.73250, region: "동해" },
    Station { code: "DT_0011", name: "후포",       lat: 36.67750, lon: 129.45305, region: "동해" },
    Station { code: "DT_0091", name: "포항",       lat: 36.05177, lon: 129.37627, region: "동해" },
    Station { code: "DT_0020", name: "울산",       lat: 35.50194, lon: 129.38722, region: "동해" },

    // ── 제주 ────────────────────────────────────────────
    Station { code: "DT_0021", name: "추자도",     lat: 33.96194, lon: 126.30027, region: "제주" },
    Station { code: "DT_0004", name: "제주",       lat: 33.52750, lon: 126.54305, region: "제주" },
    Station { code: "DT_0022", name: "성산포",     lat: 33.47472, lon: 126.92777, region: "제주" },
    Station { code: "DT_0010", name: "서귀포",     lat: 33.24000, lon: 126.56166, region: "제주" },
    Station { code: "DT_0023", name: "모슬포",     lat: 33.21444, lon: 126.25111, region: "제주" },
    Station { code: "DT_0047", name: "도농탄",     lat: 33.15805, lon: 126.27472, region: "제주" },
];

pub fn find(code: &str) -> Option<&'static Station> {
    ALL.iter().find(|s| s.code == code)
}
