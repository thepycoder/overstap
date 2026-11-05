'use strict';

// Icon paths from Lucide (ISC licence), plus a simple scooter.
const ICONS = {
  bike: '<circle cx="18.5" cy="17.5" r="3.5"/><circle cx="5.5" cy="17.5" r="3.5"/><circle cx="15" cy="5" r="1"/><path d="M12 17.5V14l-3-3 4-3 2 3h2"/>',
  walk: '<path d="M4 16v-2.38C4 11.5 2.97 10.5 3 8c.03-2.72 1.49-6 4.5-6C9.37 2 10 3.8 10 5.5c0 3.11-2 5.66-2 8.68V16a2 2 0 1 1-4 0Z"/><path d="M20 20v-2.38c0-2.12 1.03-3.12 1-5.62-.03-2.72-1.49-6-4.5-6C14.63 6 14 7.8 14 9.5c0 3.11 2 5.66 2 8.68V20a2 2 0 1 0 4 0Z"/><path d="M16 17h4"/><path d="M4 13h4"/>',
  train: '<path d="M8 3.1V7a4 4 0 0 0 8 0V3.1"/><path d="m9 15-1-1"/><path d="m15 15 1-1"/><path d="M9 19c-2.8 0-5-2.2-5-5v-4a8 8 0 0 1 16 0v4c0 2.8-2.2 5-5 5Z"/><path d="m8 19-2 3"/><path d="m16 19 2 3"/>',
  metro: '<path d="M2 22V12a10 10 0 1 1 20 0v10"/><path d="M15 6.8v1.4a3 2.8 0 1 1-6 0V6.8"/><path d="M10 15h.01"/><path d="M14 15h.01"/><path d="M10 19a4 4 0 0 1-4-4v-3a6 6 0 1 1 12 0v3a4 4 0 0 1-4 4Z"/><path d="m9 19-2 3"/><path d="m15 19 2 3"/>',
  tram: '<rect width="16" height="16" x="4" y="3" rx="2"/><path d="M4 11h16"/><path d="M12 3v8"/><path d="m8 19-2 3"/><path d="m18 22-2-3"/><path d="M8 15h.01"/><path d="M16 15h.01"/>',
  bus: '<path d="M4 6 2 7"/><path d="M10 6h4"/><path d="m22 7-2-1"/><rect width="16" height="16" x="4" y="3" rx="2"/><path d="M4 11h16"/><path d="M8 15h.01"/><path d="M16 15h.01"/><path d="M6 19v2"/><path d="M18 21v-2"/>',
  cab: '<path d="M19 17h2c.6 0 1-.4 1-1v-3c0-.9-.7-1.7-1.5-1.9C18.7 10.6 16 10 16 10s-1.3-1.4-2.2-2.3c-.5-.4-1.1-.7-1.8-.7H5c-.6 0-1.1.4-1.4.9l-1.4 2.9A3.7 3.7 0 0 0 2 12v4c0 .6.4 1 1 1h2"/><circle cx="7" cy="17" r="2"/><path d="M9 17h6"/><circle cx="17" cy="17" r="2"/>',
  scooter: '<circle cx="5" cy="18" r="2.5"/><circle cx="19" cy="18" r="2.5"/><path d="M7.5 18h9"/><path d="M15 4h3l-1.6 11.5"/>',
  locate: '<line x1="2" x2="5" y1="12" y2="12"/><line x1="19" x2="22" y1="12" y2="12"/><line x1="12" x2="12" y1="2" y2="5"/><line x1="12" x2="12" y1="19" y2="22"/><circle cx="12" cy="12" r="7"/><circle cx="12" cy="12" r="3"/>',
  swap: '<path d="m21 16-4 4-4-4"/><path d="M17 20V4"/><path d="m3 8 4-4 4 4"/><path d="M7 4v16"/>',
  back: '<path d="m12 19-7-7 7-7"/><path d="M19 12H5"/>',
  refresh: '<path d="M21 12a9 9 0 1 1-9-9c2.52 0 4.93 1 6.74 2.74L21 8"/><path d="M21 3v5h-5"/>',
  pin: '<path d="M20 10c0 4.993-5.539 10.193-7.399 11.799a1 1 0 0 1-1.202 0C9.539 20.193 4 14.993 4 10a8 8 0 0 1 16 0"/><circle cx="12" cy="10" r="3"/>',
  history: '<path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"/><path d="M3 3v5h5"/><path d="M12 7v5l4 2"/>',
  station: '<rect width="16" height="16" x="4" y="3" rx="3"/><path d="M4 11h16"/><path d="M8 15h.01"/><path d="M16 15h.01"/><path d="m8 19-2 3"/><path d="m16 19 2 3"/>',
  bookmark: '<path d="m19 21-7-4-7 4V5a2 2 0 0 1 2-2h10a2 2 0 0 1 2 2v16z"/>',
  share: '<circle cx="18.5" cy="17.5" r="3.5"/><circle cx="5.5" cy="17.5" r="3.5"/><path d="M12 17.5V14l-3-3 4-3 2 3h2"/><path d="M3 4h6"/><path d="m7 2 2 2-2 2"/>',
};
const icon = (name, cls = '') => `<svg class="i ${cls}" viewBox="0 0 24 24" aria-hidden="true">${ICONS[name] || ICONS.train}</svg>`;

// Ways of travelling around public transport; each is its own search (see src/planner.rs).
const TABS = [
  { id: 'folding', name: 'Folding bike', icon: 'bike', title: 'Your folding bike on both ends, and on board' },
  { id: 'shared', name: 'Shared bikes', icon: 'share', title: 'Blue-bike, Donkey, Velo, Hoppy… for the first and last mile' },
  { id: 'foot', name: 'On foot', icon: 'walk', title: 'Public transport and walking' },
];
const DEFAULT_TAB = 'folding';

// Timetable geometry. ppm = pixels per minute along the time axis.
const ZOOMS = [2, 3, 4, 5, 6, 8, 10, 13, 16];
const AXIS_H = 30;
const AXIS_W = 50;
const LANE_H = 66;
const EDGE = { h: 160, v: 90 };
const MIN = 60000;

// Your own bike is timed at RIDE_KMH. overstap also searches as if you rode at FAST_KMH, so
// trains you'd only catch by riding hard show up too, marked with the speed they need.
// Shared e-bikes are timed at EBIKE_KMH (MOTIS rides every rental at about 15 km/h).
const RIDE_KMH = 18;
const FAST_KMH = 25;
const EBIKE_KMH = 20;
// Longest ride before and after transit, per tab (minutes at MOTIS's speed for rentals).
const MAX_BIKE_MIN = { folding: 45, shared: 60 };
const WINDOW_MIN = 60;
// Line colours for shared-bike operators (by GBFS provider id prefix).
const PROVIDER_COLORS = [['blue-bike', '#1b7fd1'], ['donkey', '#f0642a'], ['velo', '#d71920'], ['hoppy', '#6aa82c'], ['dott', '#1aa0d8']];
const rideSec = (meters, kmh) => meters / (kmh / 3.6);

const store = {
  get(key, fallback) {
    try { const v = localStorage.getItem('ovs.' + key); return v ? JSON.parse(v) : fallback; } catch { return fallback; }
  },
  set(key, value) {
    try { this.setOrThrow(key, value); } catch { /* storage unavailable */ }
  },
  setOrThrow(key, value) {
    localStorage.setItem('ovs.' + key, JSON.stringify(value));
  },
};

const narrowQuery = matchMedia('(max-width: 860px)');

const state = {
  config: { center: [51.05, 4.25], timezone: 'Europe/Brussels' },
  from: null,
  to: null,
  timeMode: 'now',
  time: '',
  tab: DEFAULT_TAB,
  // Your own (non-folding) bike: 'none', 'here' (with you) or 'parked' at bikeAt. The planner
  // uses it for whatever stretch it can reach (ADR-018). Only for the Shared and On foot tabs;
  // the folding bike is with you anyway. Kept in the browser: it's where the bike really is.
  bike: 'none',
  bikeAt: null,
  journeys: [],
  byKey: new Map(),
  paging: { later: 'idle', earlier: 'off' },
  status: 'idle',
  error: null,
  updatedAt: null,
  seq: 0,
  selected: null,
  // How an open route is shown: 'list' (its stops) or 'braid' (to scale, with its variants).
  detailView: 'list',
  hot: null,
  showAll: false,
  savedId: null,
  rideId: null,
  pendingRoute: null,
  editing: false,
  ppm: { h: 6, v: 4 },
  sched: { o: null, anchor: null, fit: new Set(), expect: null, last: { x: 0, y: 0 } },
};

const $ = (sel, el = document) => el.querySelector(sel);
const esc = s => String(s ?? '').replace(/[&<>"']/g, c => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));
const ll = c => `${(+c[0]).toFixed(6)},${(+c[1]).toFixed(6)}`;
const toDate = s => (s ? new Date(s) : null);
const sum = xs => xs.reduce((a, b) => a + (b || 0), 0);

let timeFmt = new Intl.DateTimeFormat('en-GB', { hour: '2-digit', minute: '2-digit' });
const fmtTime = d => (d ? timeFmt.format(d) : '–');
function fmtDur(sec) {
  const m = Math.max(1, Math.round((sec || 0) / 60));
  return m < 60 ? `${m} min` : `${Math.floor(m / 60)} h ${String(m % 60).padStart(2, '0')}`;
}
function shortDur(sec) {
  const m = Math.max(1, Math.round((sec || 0) / 60));
  return m < 60 ? `${m}′` : `${Math.floor(m / 60)}h${String(m % 60).padStart(2, '0')}`;
}
const fmtDist = m => (m < 1000 ? `${Math.round(m / 10) * 10} m` : `${(m / 1000).toFixed(1)} km`);

async function api(path, params = {}, body = null) {
  const url = new URL(path, location.href);
  for (const [k, v] of Object.entries(params)) if (v != null && v !== '') url.searchParams.set(k, v);
  const res = await fetch(url, body ? { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify(body) } : undefined);
  const data = await res.json().catch(() => ({ error: `HTTP ${res.status}` }));
  if (!res.ok) throw new Error(data.error || `HTTP ${res.status}`);
  return data;
}

// ---------- reading overstap's journeys (MOTIS itineraries) ----------

const STREET_MODES = ['WALK', 'BIKE', 'RENTAL', 'CAR', 'CAR_PARKING', 'ODM', 'FLEX'];

function vehicleOf(mode) {
  if (/RAIL|TRAIN/.test(mode)) return 'train';
  if (/SUBWAY|METRO/.test(mode)) return 'metro';
  if (/TRAM/.test(mode)) return 'tram';
  if (/BUS|COACH/.test(mode)) return 'bus';
  return 'train';
}

// GTFS colours come without '#'; NMBS has none, so trains get one by category.
function routeOf(l, vehicle) {
  const name = l.displayName || l.routeShortName || '';
  let color = l.routeColor ? `#${l.routeColor}` : null;
  let text = l.routeTextColor ? `#${l.routeTextColor}` : null;
  if (!color && vehicle === 'train') {
    color = /^(IC|EC|EUR|THA|ICE)/.test(name) ? '#1c4f9c' : /^S/.test(name) ? '#5b8fd6' : '#3d6fb8';
    text = '#ffffff';
  }
  if (!color && vehicle === 'bus') { color = '#ffcc00'; text = '#000000'; }
  if (text && color && text.toLowerCase() === color.toLowerCase()) text = null;
  return { id: l.routeId || name, display_name: name, color: color || '#555555', ui_color: color || '#555555', text_color: text || '#ffffff', agency: l.agencyName };
}

const providerColor = id => (PROVIDER_COLORS.find(([p]) => (id || '').startsWith(p)) || [null, '#0e9f8e'])[1];

// Every leg keeps its timetable times in schedDep/schedArr (rides) or origDep/origArr (the
// rest); layout() derives the dep/arr actually shown, including live delays.
function normLeg(l) {
  const origDep = toDate(l.scheduledStartTime);
  const origArr = toDate(l.scheduledEndTime);
  const base = {
    mode: l.mode, origDep, origArr, dur: l.duration || (origArr - origDep) / 1000 || 0, distance: l.distance || 0,
    shape: l.legGeometry?.points ? decodePolyline(l.legGeometry.points, l.legGeometry.precision || 6) : [],
    fromName: l.from?.name, toName: l.to?.name,
  };
  if (!STREET_MODES.includes(l.mode)) {
    const vehicle = vehicleOf(l.mode);
    const route = routeOf(l, vehicle);
    const places = [l.from, ...(l.intermediateStops || []), l.to].filter(Boolean);
    const leg = {
      ...base, kind: 'transit', route, vehicle,
      schedDep: origDep, schedArr: origArr, dur: (origArr - origDep) / 1000,
      headsign: l.headsign,
      trainNo: vehicle === 'train' ? l.tripShortName : null,
      tripId: l.tripId,
      bikesAllowed: l.bikesAllowed,
      stops: places.map(p => ({ id: p.stopId, name: p.name, coords: [p.lat, p.lon], sched: toDate(p.scheduledDeparture || p.scheduledArrival) })),
    };
    leg.live = liveOf(l.from, l.to, l.realTime, l.cancelled);
    return leg;
  }
  if (l.mode === 'RENTAL') {
    const r = l.rental || {};
    const electric = r.propulsionType && r.propulsionType !== 'HUMAN';
    const kind = /SCOOTER|MOPED/.test(r.formFactor || '') ? 'scooter' : 'bike';
    const dur = electric && kind === 'bike' && base.distance ? rideSec(base.distance, EBIKE_KMH) : base.dur;
    return { ...base, kind, dur, rental: { ...r, electric, color: providerColor(r.providerId) } };
  }
  if (l.mode === 'BIKE') return { ...base, kind: 'bike' };
  if (l.mode === 'WALK') return { ...base, kind: 'walk', inStation: base.distance < 20 };
  return { ...base, kind: 'other' };
}

// Live state of a ride between two of its stops, from MOTIS places (plan or trip).
function liveOf(from, to, realTime, cancelled) {
  return {
    isLive: !!realTime,
    status: cancelled || from?.cancelled || to?.cancelled ? 3 : 0,
    platform: from?.track || from?.scheduledTrack || null,
    platformChanged: !!(from?.track && from?.scheduledTrack && from.track !== from.scheduledTrack),
    expDep: realTime ? toDate(from?.departure) : null,
    expArr: realTime ? toDate(to?.arrival) : null,
  };
}

// MOTIS shapes are encoded polylines with `precision` decimals.
function decodePolyline(str, precision = 6) {
  const points = [];
  let i = 0;
  let lat = 0;
  let lng = 0;
  while (i < str.length) {
    for (const axis of [0, 1]) {
      let shift = 0;
      let result = 0;
      let b;
      do { b = str.charCodeAt(i++) - 63; result |= (b & 0x1f) << shift; shift += 5; } while (b >= 0x20);
      const delta = result & 1 ? ~(result >> 1) : result >> 1;
      if (axis === 0) lat += delta; else lng += delta;
    }
    points.push([lat / 10 ** precision, lng / 10 ** precision]);
  }
  return points;
}

// `win` holds the times a route has to fit, in ms: `from` when you can leave, `to` when you must
// arrive. Routes from other searches (paging, saved without one) have none.
function normJourney(raw, win = null, ownBike = false) {
  const legs = (raw.legs || []).map(normLeg);
  if (!legs.length) return null;
  // MOTIS models a change as a walk between two rides; it belongs in the gap, not a row.
  legs.forEach((l, k) => { if (l.kind === 'walk' && legs[k - 1]?.kind === 'transit' && legs[k + 1]?.kind === 'transit') l.transfer = true; });
  for (const l of legs) {
    // Only your own bike is re-timed (and sped up to catch a train); rentals keep their times.
    if (l.kind === 'bike' && !l.rental && l.distance) l.rideDur = l.dur = rideSec(l.distance, RIDE_KMH);
  }
  const j = {
    raw, legs,
    label: raw.label || null,
    notes: raw.notes || [],
    bikeMeters: sum(legs.filter(l => l.kind === 'bike').map(l => l.distance)),
    walkMin: Math.round(sum(legs.filter(l => l.kind === 'walk').map(l => l.dur)) / 60),
    rentals: legs.filter(l => l.rental),
    ownBike,
    key: null,
    from: null,
    to: null,
    liveAt: null,
    win,
    fastKmh: 0,
    missed: false,
  };
  layout(j);
  j.schedStart = j.start;
  j.schedEnd = j.end;
  j.changes = Math.max(0, j.transit.length - 1);
  // Where MOTIS itself starts the route: paging goes on from there.
  j.qStart = toDate(raw.startTime);
  return j;
}

// Stretches the rides among `legs` so all of them take `avail` ms. Returns the speed that takes:
// 0 when your usual pace already fits (give or take a minute), Infinity (leaving the times
// alone) when FAST_KMH doesn't.
function fitRides(legs, avail) {
  if (sum(legs.map(l => l.dur)) * 1000 <= avail + MIN) return 0;
  const rides = legs.filter(l => l.rideDur);
  const rideMs = avail - sum(legs.filter(l => !l.rideDur).map(l => l.dur)) * 1000;
  const kmh = sum(rides.map(l => l.distance)) / (rideMs / 1000) * 3.6;
  if (!rides.length || rideMs <= 0 || kmh > FAST_KMH) return Infinity;
  for (const l of rides) {
    l.dur = rideSec(l.distance, kmh);
    l.kmh = kmh;
  }
  return kmh;
}

// Times every leg. Rides use their live expected times when known, else the timetable. Legs
// before the first timed ride are slid to end exactly when it leaves (MOTIS may start them
// earlier and wait on the platform), and later legs follow on from each other.
// Rides that don't fit the route's time window at your usual pace are sped up, up to FAST_KMH.
function layout(j) {
  const legs = j.legs;
  for (const l of legs) {
    if (l.rideDur) { l.dur = l.rideDur; l.kmh = 0; }
  }
  j.fastKmh = 0;
  j.missed = false;
  for (const l of legs) {
    if (l.kind !== 'transit') continue;
    const lv = l.live;
    l.dep = lv?.expDep || l.schedDep || null;
    l.arr = lv?.expArr || (lv?.expDep && l.schedArr ? new Date(+l.schedArr + (lv.expDep - l.schedDep)) : l.schedArr) || null;
    if (l.dep && !l.arr) l.arr = new Date(+l.dep + l.dur * 1000);
    l.delay = lv?.expDep && l.schedDep ? Math.round((lv.expDep - l.schedDep) / MIN) : 0;
    l.cancelled = lv?.status === 3;
    l.atRisk = false;
  }
  let anchor = legs.findIndex(l => l.kind === 'transit' && l.dep);
  if (anchor === -1) {
    anchor = 0;
    legs[0].dep = legs[0].origDep || new Date();
    legs[0].arr = new Date(+legs[0].dep + legs[0].dur * 1000);
  } else if (j.win) {
    const last = legs.length - 1 - [...legs].reverse().findIndex(l => l.kind === 'transit' && l.dep);
    const before = j.win.from != null ? fitRides(legs.slice(0, anchor), legs[anchor].dep - j.win.from) : 0;
    const after = j.win.to != null ? fitRides(legs.slice(last + 1), j.win.to - legs[last].arr) : 0;
    j.missed = before === Infinity || after === Infinity;
    if (!j.missed) j.fastKmh = Math.ceil(Math.max(before, after));
  }
  for (let k = anchor - 1; k >= 0; k--) {
    legs[k].arr = legs[k + 1].dep;
    legs[k].dep = new Date(+legs[k].arr - legs[k].dur * 1000);
  }
  for (let k = anchor + 1; k < legs.length; k++) {
    const l = legs[k];
    const prevArr = legs[k - 1].arr;
    if (l.kind === 'transit' && l.dep) {
      // A delay earlier on can make a connection impossible.
      if (prevArr > l.dep) l.atRisk = true;
      continue;
    }
    l.dep = prevArr;
    l.arr = new Date(+prevArr + l.dur * 1000);
  }
  j.transit = legs.filter(l => l.kind === 'transit');
  j.start = legs[0].dep;
  j.end = legs[legs.length - 1].arr;
  j.duration = (j.end - j.start) / 1000;
  j.delay = j.schedEnd && j.transit.length ? Math.round((j.end - j.schedEnd) / MIN) : 0;
  j.cancelled = j.transit.some(l => l.cancelled);
  j.atRisk = j.transit.some(l => l.atRisk);
}

// The vehicle a ride is on: its trip, or its line and timetable departure when MOTIS has no trip.
const tripOf = l => l.tripId || `${l.route.id}@${Math.round(l.schedDep / MIN)}`;
const legPart = l => (l.kind === 'transit' ? tripOf(l) : l.rental ? `${l.kind}:${l.rental.providerId}` : l.kind);

// One key per real option: the same vehicles reached the same way, whichever search found it.
function journeyKey(j, tab) {
  const parts = [];
  for (const l of j.legs) {
    const p = legPart(l);
    if (parts[parts.length - 1] !== p) parts.push(p);
  }
  if (!j.transit.length) parts.push(Math.round(j.start / MIN));
  return `${tab}|${parts.join('>')}`;
}

function normalize(d, win = null, tab = state.tab) {
  return (d.journeys || []).map(raw => {
    const j = normJourney(raw, win, tab === 'folding');
    if (j) j.key = journeyKey(j, tab);
    return j;
  }).filter(Boolean);
}

const wanted = j => !j.missed;
const titleCase = s => s.toLowerCase().replace(/_/g, ' ').replace(/^\w/, c => c.toUpperCase());

// ---------- planning & paging ----------

function isoFromDate(d) {
  const pad = n => String(Math.floor(Math.abs(n))).padStart(2, '0');
  const off = -d.getTimezoneOffset();
  return `${d.getFullYear()}-${pad(d.getMonth() + 1)}-${pad(d.getDate())}T${pad(d.getHours())}:${pad(d.getMinutes())}:00${off >= 0 ? '+' : '-'}${pad(off / 60)}:${pad(off % 60)}`;
}

function defaultTimeValue() {
  const d = new Date(Date.now() + 15 * MIN);
  d.setMinutes(Math.ceil(d.getMinutes() / 5) * 5, 0, 0);
  return isoFromDate(d).slice(0, 16);
}

const targetTime = () => (state.timeMode === 'now' || !state.time ? null : new Date(state.time));

// One overstap search. `time` is when to leave (or arrive, with ARRIVE_BY); none means now.
function fetchJourneys(timeMode, time, win = null) {
  const tab = state.tab;
  const params = { start: ll(state.from.coords), end: ll(state.to.coords), mode: tab, window: WINDOW_MIN, maxbike: MAX_BIKE_MIN[tab] };
  if (tab === 'folding') Object.assign(params, { kmh: RIDE_KMH, fast: FAST_KMH });
  const bike = bikeParam();
  if (bike) Object.assign(params, { bike, kmh: RIDE_KMH });
  if (time) params.time = isoFromDate(time);
  if (timeMode === 'ARRIVE_BY') params.arrive = 1;
  const from = slim(state.from);
  const to = slim(state.to);
  return api('/api/journeys', params).then(d => normalize(d, win, tab).map(j => Object.assign(j, { from, to, tab, bike })));
}

// Adds a page of results; returns the journeys it contributed.
function merge(page, { initial = false } = {}) {
  const fresh = [];
  for (const j of page) {
    if (!wanted(j) || state.byKey.has(j.key)) continue;
    // Options without public transport don't depend on the time; keep the first page's.
    if (!initial && !j.transit.length) continue;
    state.byKey.set(j.key, j);
    state.journeys.push(j);
    fresh.push(j);
  }
  state.journeys.sort((a, b) => a.start - b.start || a.end - b.end);
  // Reopen the route that was open before a reload (the URL keeps it) once the search finds it.
  if (state.pendingRoute && state.byKey.has(state.pendingRoute)) {
    state.selected = state.pendingRoute;
    state.pendingRoute = null;
  }
  return fresh;
}

const liveThenRender = journeys => refreshLive(journeys).then(changed => { if (changed) render(); });

async function plan() {
  writeHash();
  const seq = ++state.seq;
  Object.assign(state, {
    journeys: [], byKey: new Map(), selected: null, hot: null,
    paging: { later: 'idle', earlier: state.timeMode === 'now' ? 'off' : 'idle' },
  });
  state.sched.anchor = null;
  state.sched.fit = new Set(['h', 'v']);
  setHeader(true);
  if (!state.from || !state.to) {
    state.status = 'idle';
    render();
    return;
  }
  state.status = 'loading';
  render();
  const target = targetTime();
  const ref = target || new Date();
  const arriving = state.timeMode === 'arrive';
  // This search's routes have to fit your leave (or arrive-by) time; paging and saved routes don't.
  const win = arriving ? { from: null, to: +ref } : { from: +ref, to: null };
  try {
    // MOTIS covers a whole window in one search (WINDOW_MIN), so no sweeping is needed.
    const page = await fetchJourneys(arriving ? 'ARRIVE_BY' : 'DEPART_AT', ref, win);
    if (seq !== state.seq) return;
    merge(page, { initial: true });
    state.status = 'ok';
    state.updatedAt = new Date();
    rememberTrip();
  } catch (e) {
    if (seq !== state.seq) return;
    state.status = 'error';
    state.error = e.message;
    state.pendingRoute = null;
  }
  if (state.pendingRoute && state.status === 'ok' && !state.selected) state.pendingRoute = null;
  writeHash();
  render();
}

// Further pages go by departure time either way, on from where MOTIS started the routes
// shown, so every departure in between gets covered.
async function loadMore(dir) {
  if (state.status !== 'ok' || state.paging[dir] !== 'idle') return;
  const seq = state.seq;
  state.paging[dir] = 'loading';
  renderSchedule();
  const timed = state.journeys.filter(j => j.transit.length);
  const ref = +(targetTime() || new Date());
  const span = WINDOW_MIN * MIN;
  let start = dir === 'later'
    ? new Date((timed.length ? Math.max(...timed.map(j => +j.qStart)) : ref) + MIN)
    : new Date((timed.length ? Math.min(...timed.map(j => +j.qStart)) : ref) - span);
  let result = 'done';
  try {
    // One retry further along, in case of a gap in the timetable (nights).
    for (let attempt = 0; attempt < 2; attempt++) {
      const page = await fetchJourneys('DEPART_AT', start);
      if (seq !== state.seq) return;
      const fresh = merge(page);
      if (fresh.length) { result = 'idle'; break; }
      start = new Date(+start + (dir === 'later' ? 1 : -1) * span);
    }
  } catch {
    result = 'error';
  }
  if (seq !== state.seq) return;
  state.paging[dir] = result;
  render();
}

// Routes that another route clearly beats: it leaves at most BEAT_MARGIN minutes earlier,
// arrives no later, and saves enough time in total: at least BEAT_MARGIN minutes, plus
// CHANGE_WORTH for every change it has more. With fewer changes it only has to leave no
// earlier (the same train reached by bike instead of an extra hop). So near-equal alternatives both stay, but a
// route with one change less that arrives two hours later doesn't. Cancelled routes, broken
// connections and routes you'd have to ride faster for never beat anything, and saved routes
// are never counted as beaten.
const BEAT_MARGIN = 5;
const CHANGE_WORTH = 15;

function beatenJourneys() {
  const all = state.journeys;
  const saved = new Set(savedEntries().map(s => s.id));
  const t = d => Math.round(+d / MIN);
  const beats = (a, b) => {
    if (a.cancelled || a.atRisk || a.fastKmh) return false;
    if (t(a.start) < t(b.start) - BEAT_MARGIN || t(a.end) > t(b.end)) return false;
    const saving = (t(a.start) - t(b.start)) + (t(b.end) - t(a.end));
    const extra = a.changes - b.changes;
    if (extra < 0) return t(a.start) >= t(b.start);
    return saving >= BEAT_MARGIN + CHANGE_WORTH * extra;
  };
  return new Set(all.filter(b => !saved.has(b.key) && all.some(a => a !== b && beats(a, b))));
}

function shownJourneys() {
  if (state.showAll) return state.journeys;
  const beaten = beatenJourneys();
  return state.journeys.filter(j => !beaten.has(j));
}

// ---------- live departures: delays, cancellations, tracks ----------

// MOTIS's plans already carry live times as of the search; after that each ride's trip is
// fetched again (POST /api/live), and matched by the stops you board and leave at.
function applyTrip(l, trip) {
  const places = [trip.from, ...(trip.intermediateStops || []), trip.to].filter(Boolean);
  const board = l.stops[0];
  const alight = l.stops[l.stops.length - 1];
  const i = places.findIndex(p => p.stopId === board.id && Math.abs(toDate(p.scheduledDeparture) - board.sched) < 2 * MIN);
  if (i === -1) return false;
  const k = places.findIndex((p, n) => n > i && p.stopId === alight.id);
  if (k === -1) return false;
  l.live = liveOf(places[i], places[k], trip.realTime, trip.cancelled);
  return true;
}

// Live data only exists close to departure, so only ask for journeys starting within 3 hours.
// A route you've opened or saved may be the one you're on, so it's followed until it's well
// overdue; in a list, routes that ended 10 minutes ago are done with.
async function refreshLive(journeys) {
  const now = Date.now();
  const open = currentDetail();
  const grace = j => (j.saved || j === open ? 90 : 10) * MIN;
  const due = journeys.filter(j => j.transit.length && +j.end > now - grace(j) && +j.start < now + 180 * MIN).slice(0, 30);
  const ids = [...new Set(due.flatMap(j => j.transit.map(l => l.tripId)).filter(Boolean))];
  if (!ids.length) return false;
  const trips = {};
  for (let i = 0; i < ids.length; i += 20) {
    try {
      Object.assign(trips, (await api('/api/live', {}, { trips: ids.slice(i, i + 20) })).trips);
    } catch { /* keep what we had */ }
  }
  let changed = false;
  for (const j of due) {
    let any = false;
    for (const l of j.transit) if (trips[l.tripId] && applyTrip(l, trips[l.tripId])) any = true;
    if (any) { j.liveAt = new Date(); layout(j); changed = true; }
  }
  return changed;
}

async function liveTick() {
  if (document.hidden) return;
  const mode = document.body.dataset.mode;
  const open = currentDetail();
  const journeys = open ? [open] : mode === 'results' ? state.journeys : savedJourneys();
  if (await refreshLive(journeys)) render();
}

// ---------- saved routes (kept in this browser only) ----------

// Saved journeys are rebuilt from the raw itinerary once and then kept, so live updates stick.
const savedCache = new Map();
const savedEntries = () => store.get('saved', []);

function savedJourney(entry) {
  let j = savedCache.get(entry.id);
  if (!j) {
    j = normJourney(entry.raw, entry.win || null, entry.tab === 'folding');
    if (!j) return null;
    Object.assign(j, { key: entry.id, from: entry.from, to: entry.to, tab: entry.tab, bike: entry.bike || null, saved: true });
    savedCache.set(entry.id, j);
  }
  return j;
}

const savedJourneys = () => savedEntries().map(savedJourney).filter(Boolean).sort((a, b) => a.start - b.start);
const isSaved = key => savedEntries().some(s => s.id === key);

function saveJourney(j) {
  const list = savedEntries().filter(s => s.id !== j.key);
  list.push({ id: j.key, savedAt: Date.now(), tab: j.tab, bike: j.bike || null, from: j.from, to: j.to, raw: j.raw, win: j.win });
  try {
    store.setOrThrow('saved', list);
  } catch {
    alert('Could not save this route: browser storage is full or unavailable.');
  }
}

function unsaveJourney(key) {
  store.set('saved', savedEntries().filter(s => s.id !== key));
  savedCache.delete(key);
  if (state.savedId === key) state.savedId = null;
}

// Saved routes are kept until half a day after they end.
function pruneSaved() {
  const cutoff = Date.now() - 12 * 60 * MIN;
  store.set('saved', savedEntries().filter(s => { const j = savedJourney(s); return j && +j.end > cutoff; }));
}

// The one to offer first: the next saved route that isn't over yet.
function nextSaved() {
  const list = savedJourneys();
  return list.find(j => +j.end > Date.now() - 10 * MIN) || list[list.length - 1] || null;
}

function currentDetail() {
  if (state.savedId) {
    const entry = savedEntries().find(s => s.id === state.savedId);
    return entry ? savedJourney(entry) : null;
  }
  return (state.selected && state.byKey.get(state.selected)) || null;
}

function openSaved(id) {
  state.rideId = null;
  state.savedId = id;
  state.editing = false;
  $('#savedMenu').hidden = true;
  setHot(null);
  writeHash();
  render();
  $('#detail').scrollTop = 0;
  const j = currentDetail();
  if (j) liveThenRender([j]);
}

// ---------- URL + local memory ----------

function writeHash() {
  const p = new URLSearchParams();
  const enc = pl => `${(+pl.coords[0]).toFixed(5)},${(+pl.coords[1]).toFixed(5)},${pl.name || ''}`;
  if (state.from) p.set('from', enc(state.from));
  if (state.to) p.set('to', enc(state.to));
  if (state.tab !== DEFAULT_TAB) p.set('tab', state.tab);
  if (state.timeMode !== 'now' && state.time) { p.set('when', state.timeMode); p.set('t', state.time); }
  // The open route, so a reload (or the phone discarding the tab) brings you straight back.
  if (state.rideId) p.set('ride', state.rideId);
  else if (state.savedId) p.set('saved', state.savedId);
  else if (state.selected || state.pendingRoute) p.set('route', state.selected || state.pendingRoute);
  if (!state.rideId && state.detailView === 'braid' && (p.has('saved') || p.has('route'))) p.set('view', 'braid');
  history.replaceState(null, '', p.toString() ? '#' + p.toString() : location.pathname);
}

function readHash() {
  const p = new URLSearchParams(location.hash.slice(1));
  const dec = v => {
    if (!v) return null;
    const [lat, lng, ...name] = v.split(',');
    return Number.isFinite(+lat) && Number.isFinite(+lng) ? { coords: [+lat, +lng], name: name.join(',') || 'Pinned place' } : null;
  };
  state.from = dec(p.get('from'));
  state.to = dec(p.get('to'));
  state.tab = TABS.some(t => t.id === p.get('tab')) ? p.get('tab') : DEFAULT_TAB;
  state.timeMode = p.get('when') && p.get('t') ? p.get('when') : 'now';
  state.time = state.timeMode === 'now' ? '' : p.get('t');
  state.savedId = savedEntries().some(s => s.id === p.get('saved')) ? p.get('saved') : null;
  state.pendingRoute = p.get('route');
  state.detailView = p.get('view') === 'braid' ? 'braid' : 'list';
  state.rideId = savedEntries().some(s => s.id === p.get('ride')) ? p.get('ride') : null;
}

const samePlace = (a, b) => a && b && Math.abs(a.coords[0] - b.coords[0]) < 1e-4 && Math.abs(a.coords[1] - b.coords[1]) < 1e-4;
const slim = p => ({ name: p.name, address: p.address || null, coords: [+p.coords[0], +p.coords[1]] });

function rememberPlace(p) {
  if (!p || p.current) return;
  const list = store.get('places', []).filter(x => !samePlace(x, p));
  store.set('places', [slim(p), ...list].slice(0, 8));
}

function rememberTrip() {
  if (state.from.current || state.to.current) return;
  const trip = { from: slim(state.from), to: slim(state.to) };
  const list = store.get('trips', []).filter(t => !(samePlace(t.from, trip.from) && samePlace(t.to, trip.to)));
  store.set('trips', [trip, ...list].slice(0, 6));
}

// ---------- map ----------

const map = L.map('map', { zoomControl: false }).setView(state.config.center, 9);
L.control.zoom({ position: 'topright' }).addTo(map);
// Dark mode inverts these tiles in CSS (.leaflet-tile-pane).
L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
  maxZoom: 19,
  attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> · routes by MOTIS',
}).addTo(map);

const routeLayer = L.layerGroup().addTo(map);
const markers = {};
let drawnJourney = null;

function placeMarker(role) {
  const p = role === 'bikeAt' && state.bike !== 'parked' ? null : state[role];
  if (!p) { markers[role]?.remove(); delete markers[role]; return; }
  if (!markers[role]) {
    markers[role] = L.marker(p.coords, {
      draggable: true, zIndexOffset: 1000, keyboard: false,
      icon: L.divIcon({ className: '', html: `<div class="pin ${role}"></div>`, iconSize: [18, 18], iconAnchor: [9, 9] }),
    }).addTo(map);
    markers[role].on('dragend', e => { const { lat, lng } = e.target.getLatLng(); setFromPoint(role, lat, lng); });
  } else {
    markers[role].setLatLng(p.coords);
  }
}

function legColor(l) {
  const css = getComputedStyle(document.documentElement);
  if (l.kind === 'transit') return l.route.ui_color || l.route.color || '#555';
  if (l.rental) return l.rental.color;
  if (l.kind === 'bike') return css.getPropertyValue(l.kmh ? '--fast' : '--accent').trim();
  if (l.kind === 'scooter') return css.getPropertyValue('--scooter').trim();
  if (l.kind === 'cab') return css.getPropertyValue('--cab').trim();
  return css.getPropertyValue('--walk').trim();
}

function drawJourney(j, { fit = true } = {}) {
  if (drawnJourney === j && !fit) return;
  drawnJourney = j;
  routeLayer.clearLayers();
  const points = [];
  if (!j) return;
  for (const l of j.legs) {
    if (!l.shape.length) continue;
    points.push(...l.shape);
    const color = legColor(l);
    if (l.kind === 'walk') {
      L.polyline(l.shape, { color, weight: 4, dashArray: '1 8', lineCap: 'round', interactive: false }).addTo(routeLayer);
      continue;
    }
    L.polyline(l.shape, { color: '#ffffff', weight: 9, opacity: 0.9, interactive: false }).addTo(routeLayer);
    L.polyline(l.shape, { color, weight: 5, interactive: false }).addTo(routeLayer);
    if (l.kind === 'transit') {
      for (const s of [l.stops[0], l.stops[l.stops.length - 1]]) {
        if (!s?.coords) continue;
        L.circleMarker(s.coords, { radius: 5, color, weight: 3, fillColor: '#fff', fillOpacity: 1 })
          .bindTooltip(esc(s.name), { direction: 'top', className: 'stop-tip' }).addTo(routeLayer);
      }
    }
  }
  if (fit && points.length) {
    // Keep the route clear of the floating detail panel on wide screens.
    const panel = !narrowQuery.matches && document.body.dataset.mode === 'detail';
    const pad = narrowQuery.matches ? 24 : 60;
    map.fitBounds(L.latLngBounds(points), { paddingTopLeft: [panel ? 450 : pad, pad], paddingBottomRight: [pad, pad], maxZoom: 16 });
  }
}

// ---------- endpoints & autocomplete ----------

function setEndpoint(role, place, { replan = true } = {}) {
  if (role === 'bikeAt') {
    setBike(place ? 'parked' : 'none', place && slim(place));
    if (place) rememberPlace(state.bikeAt);
    if (replan) { state.savedId = null; plan(); }
    return;
  }
  state[role] = place ? { name: place.name, address: place.address, coords: [+place.coords[0], +place.coords[1]], current: !!place.current } : null;
  $('#' + role).value = place ? place.name : '';
  placeMarker(role);
  rememberPlace(state[role]);
  if (replan) { state.savedId = null; plan(); }
}

async function setFromPoint(role, lat, lng) {
  setEndpoint(role, { name: 'Pinned place', coords: [lat, lng] });
  try {
    const r = await api('/api/reverse', { lat: lat.toFixed(6), lng: lng.toFixed(6) });
    if (r.name && state[role] && samePlace(state[role], { coords: [lat, lng] })) {
      state[role].name = r.name;
      state[role].address = r.address;
      $('#' + role).value = r.name;
      writeHash();
      rememberPlace(state[role]);
      renderSummary();
    }
  } catch { /* keep the generic name */ }
}

function locate(role) {
  if (!navigator.geolocation) return alert('Location is not available in this browser.');
  const input = $('#' + role);
  input.value = 'Locating…';
  navigator.geolocation.getCurrentPosition(
    pos => setEndpoint(role, { name: 'Current location', coords: [pos.coords.latitude, pos.coords.longitude], current: true }),
    err => { input.value = state[role]?.name || ''; alert('Could not get your location: ' + err.message); },
    { enableHighAccuracy: true, timeout: 10000, maximumAge: 60000 },
  );
}

function placeIcon(p) {
  if (p.kind === 'here') return 'locate';
  if (p.kind === 'recent') return 'history';
  if (/station|stop/i.test(p.type || '') || /Station|Stop/.test(p.id || '')) return 'station';
  return 'pin';
}

function setupField(role) {
  const input = $('#' + role);
  const list = input.parentElement.querySelector('.suggest');
  let items = [];
  let active = -1;
  let timer = null;
  let seq = 0;

  const close = () => { list.hidden = true; active = -1; };
  const show = () => {
    list.innerHTML = items.length
      ? items.map((p, k) => `<li role="option" data-k="${k}" aria-selected="${k === active}">
          <span class="s-ico">${icon(placeIcon(p))}</span>
          <span><b>${esc(p.name)}</b>${p.address ? `<small>${esc(p.address)}</small>` : ''}</span></li>`).join('')
      : '<li class="s-empty">No places found</li>';
    list.hidden = false;
  };
  const recents = () => {
    items = [{ kind: 'here', name: 'Current location' }, ...store.get('places', []).map(p => ({ ...p, kind: 'recent' }))];
    active = -1;
    show();
  };
  async function search() {
    const q = input.value.trim();
    if (q.length < 2) return recents();
    const mine = ++seq;
    const other = state[{ from: 'to', to: 'from', bikeAt: 'to' }[role]];
    const near = other?.coords || [map.getCenter().lat, map.getCenter().lng];
    try {
      const res = await api('/api/search', { q, near: ll(near) });
      if (mine !== seq) return;
      items = res;
      active = items.length ? 0 : -1;
      show();
    } catch { /* leave the list as it was */ }
  }
  async function choose(p) {
    close();
    if (!p) return;
    if (p.kind === 'here') return locate(role);
    let place = p;
    if (!place.coords) {
      input.value = p.name + '…';
      try { place = { ...p, ...(await api('/api/resolve', { id: p.id })) }; } catch { input.value = p.name; return; }
      place.name = p.name;
      place.address = place.address || p.address;
    }
    if (!place.coords) { input.value = p.name; return; }
    setEndpoint(role, place);
    if (role === 'from' && !state.to) $('#to').focus();
  }

  input.addEventListener('input', () => { clearTimeout(timer); timer = setTimeout(search, 250); });
  input.addEventListener('focus', () => { input.select(); if (!input.value.trim() || state[role]?.name === input.value) recents(); });
  input.addEventListener('blur', () => setTimeout(() => {
    close();
    if (!input.value.trim() && state[role]) setEndpoint(role, null);
    else if (state[role] && input.value !== state[role].name) input.value = state[role].name;
  }, 150));
  input.addEventListener('keydown', e => {
    if (list.hidden && e.key === 'ArrowDown') return search();
    if (e.key === 'ArrowDown' || e.key === 'ArrowUp') {
      e.preventDefault();
      active = (active + (e.key === 'ArrowDown' ? 1 : -1) + items.length) % Math.max(items.length, 1);
      show();
    } else if (e.key === 'Enter') {
      e.preventDefault();
      choose(items[active] || items[0]);
    } else if (e.key === 'Escape') {
      close();
    }
  });
  list.addEventListener('pointerdown', e => {
    const li = e.target.closest('li[data-k]');
    if (!li) return;
    e.preventDefault();
    choose(items[+li.dataset.k]);
  });
}

// ---------- shared bits of route rendering ----------

// Place names where each leg starts and ends. Only transit legs carry names; the rest borrow
// them from neighbouring stations, or stay null when there is nothing sensible to show.
function legNames(j) {
  const legs = j.legs;
  const n = legs.length;
  const names = legs.map(l => (l.kind === 'transit' ? { from: l.stops[0]?.name, to: l.stops[l.stops.length - 1]?.name } : { from: null, to: null }));
  for (let k = 0; k < n; k++) {
    const l = legs[k];
    const nm = names[k];
    if (l.kind === 'transit') continue;
    nm.from = k === 0 ? j.from.name : names[k - 1].to;
    if (k === n - 1) nm.to = j.to.name;
    else if (legs[k + 1].kind === 'transit') nm.to = names[k + 1].from;
    else if (legs[k + 1].kind === 'walk' && legs[k + 2]?.kind === 'transit') nm.to = names[k + 2].from;
    else if (l.kind === 'walk') nm.to = nm.from;
    if (l.rental) {
      nm.from = l.rental.fromStationName || nm.from;
      nm.to = l.rental.toStationName || nm.to;
    }
  }
  return names;
}

// Per-journey markers: missed (leave-now routes whose start has passed), late (after an
// arrive-by time) and the single best option for the chosen time mode.
function journeyFlags(routes) {
  const target = targetTime();
  const now = Date.now();
  const flags = new Map(routes.map(j => [j.key, {
    past: state.timeMode === 'now' && +j.start < now - MIN,
    late: state.timeMode === 'arrive' && +j.end > +target,
    lateMin: target ? Math.round((j.end - target) / MIN) : 0,
    best: false,
  }]));
  let best = null;
  for (const j of routes) {
    const f = flags.get(j.key);
    if (j.cancelled || j.atRisk || j.fastKmh) continue;
    if (state.timeMode === 'arrive') {
      if (j.transit.length && !f.late && (!best || j.start > best.start)) best = j;
    } else if (!f.past && !(state.timeMode === 'depart' && j.start < target)) {
      if (!best || j.end < best.end || (+j.end === +best.end && j.duration < best.duration)) best = j;
    }
  }
  if (best) flags.get(best.key).best = true;
  return flags;
}

const pill = r => `<span class="pill" style="background:${esc(r.ui_color || r.color || '#555')};color:${esc(r.text_color || '#fff')}">${esc(r.display_name || r.id)}</span>`;
const blockColors = l => (l.kind === 'transit' ? `background:${esc(l.route.ui_color || l.route.color || '#555')};color:${esc(l.route.text_color || '#fff')};`
  : l.rental ? `background:${esc(l.rental.color)};color:#fff;` : '');

function legTitle(l, nm, j) {
  if (l.kind === 'transit') {
    const extra = [l.live?.platform && `track ${l.live.platform}`, l.cancelled ? 'cancelled' : l.delay >= 1 && `${l.delay} min late`].filter(Boolean).join(', ');
    return `${l.route.display_name || ''}${l.headsign ? ` towards ${l.headsign}` : ''}${extra ? ` (${extra})` : ''}\n${nm.from || ''} ${fmtTime(l.dep)} → ${nm.to || ''} ${fmtTime(l.arr)} (${fmtDur(l.dur)})`;
  }
  const what = l.rental ? (l.rental.systemName || l.rental.providerId) : { bike: 'Ride your bike', scooter: 'Scooter', cab: 'Taxi or on-demand', walk: j.ownBike ? 'Walk with your bike' : 'Walk', other: 'Travel' }[l.kind];
  return `${what}${l.distance ? ` ${fmtDist(l.distance)}` : ''} · ${fmtDur(l.dur)}${l.kmh ? ` at ${Math.ceil(l.kmh)} km/h` : ''}${nm.to ? `\n→ ${nm.to}` : ''}`;
}

// ---------- rendering ----------

// The stage shows the timetable for results, and the map before a search or for a route's details.
function render() {
  const riding = currentRide();
  const open = riding || currentDetail();
  const mode = riding ? 'ride' : open ? 'detail' : !state.from || !state.to ? 'idle' : 'results';
  const stage = mode === 'results' ? 'schedule' : 'map';
  if (mode !== 'results') setHeader(true);
  Object.assign(document.body.dataset, { mode, stage, dview: state.detailView });
  document.body.classList.toggle('editing', state.editing);

  renderSummary();
  renderTabs();
  renderViewbar();
  renderSavedBar();
  renderNotice();
  if (mode === 'idle') renderIntro();
  if (mode === 'detail') renderDetail(open);
  if (mode === 'ride') renderRide(riding);
  // A saved route can be from another search; its own line shows the ends, so hide the search pins.
  for (const role of ['from', 'to']) {
    if (open?.saved || riding) { markers[role]?.remove(); delete markers[role]; } else placeMarker(role);
  }
  if (stage === 'schedule') renderSchedule();
  else requestAnimationFrame(() => { map.invalidateSize(); drawJourney(open); });
}

// The search bar slides away while you scroll through the timetable, so it can use the whole
// screen; scrolling back, or tapping the pill that replaces it, brings it back.
function setHeader(visible) {
  document.body.classList.toggle('hdr-hidden', !visible);
}

function renderSummary() {
  if (!state.from || !state.to) return;
  const when = (state.timeMode === 'now' ? 'Leave now' : `${state.timeMode === 'arrive' ? 'Arrive by' : 'Depart'} ${fmtTime(targetTime())}`)
    + ({ here: ' · your bike', parked: ` · bike at ${state.bikeAt?.name}` }[bikeParam() && state.bike] || '');
  const html = `<span class="sum-route"><b>${esc(state.from.name)}</b> → <b>${esc(state.to.name)}</b></span><span class="sum-when">${esc(when)}</span>`;
  $('#summary').innerHTML = html;
  $('#pill').innerHTML = `${icon('pin')}${html}`;
}

// The API's `bike`: 'here', where it's parked as lat,lng, or null (none, or the folding tab).
function bikeParam() {
  if (state.tab === 'folding') return null;
  if (state.bike === 'here') return 'here';
  return state.bike === 'parked' && state.bikeAt ? ll(state.bikeAt.coords) : null;
}

function setBike(mode, at = state.bikeAt) {
  state.bike = mode;
  state.bikeAt = at;
  store.set('bike', { mode, at: at && slim(at) });
  $('#bikeAt').value = at?.name || '';
  placeMarker('bikeAt');
}

function renderTabs() {
  $('#tabs').innerHTML = TABS.map(t =>
    `<button type="button" data-tab="${t.id}" aria-pressed="${t.id === state.tab}" title="${esc(t.title)}">${icon(t.icon)} ${t.name}</button>`).join('');
  const sel = $('#bike-mode');
  sel.hidden = state.tab === 'folding';
  sel.value = state.bike;
  sel.querySelector('[value="parked"]').textContent = state.bikeAt ? `My bike: at ${state.bikeAt.name}` : 'My bike: parked…';
  $('#bikeat').hidden = sel.hidden || state.bike !== 'parked';
}

function renderViewbar() {
  const o = state.sched.o || (narrowQuery.matches ? 'v' : 'h');
  const z = ZOOMS.indexOf(state.ppm[o]);
  $('[data-zoom="-1"]').disabled = z <= 0;
  $('[data-zoom="1"]').disabled = z >= ZOOMS.length - 1;
  $('.refresh').title = state.updatedAt ? `Refresh (updated ${fmtTime(state.updatedAt)})` : 'Refresh';
  const hidden = state.status === 'ok' ? beatenJourneys().size : 0;
  const toggle = $('.show-all');
  toggle.hidden = hidden < 1;
  toggle.setAttribute('aria-pressed', String(state.showAll));
  toggle.textContent = state.showAll ? `Hide ${hidden} slower` : `+${hidden} slower`;
  toggle.title = state.showAll
    ? 'Hide routes that another route beats: leaving about as late or later and arriving clearly earlier (an extra change counts as 15 min)'
    : `${hidden} route${hidden > 1 ? 's are' : ' is'} hidden because another route leaves about as late or later and arrives clearly earlier (an extra change counts as 15 min)`;
}

function renderNotice() {
  let html = '';
  if (state.status === 'error') {
    html = `<div class="notice error">${esc(state.error)} <button type="button" data-action="retry">Try again</button></div>`;
  } else if (state.status === 'ok' && !state.journeys.length) {
    html = state.tab === 'foot'
      ? '<div class="notice">No public transport routes found for this trip.</div>'
      : `<div class="notice">No ${state.tab === 'folding' ? 'folding-bike' : 'shared-bike'} routes for this trip. <button type="button" data-tab="foot">Try on foot</button></div>`;
  }
  $('#notice').innerHTML = html;
}

// ---------- intro card + detail header pieces ----------

function renderIntro() {
  const trips = store.get('trips', []);
  const saved = savedJourneys();
  $('#intro').innerHTML = `<h2>overstap</h2>
    <p>Pick a start and destination above. <b>Folding bike</b> rides to the station, takes the bike on board and rides the last stretch; <b>Shared bikes</b> adds Blue-bike, Donkey, Velo and Hoppy, with their parking rules; <b>On foot</b> is public transport and walking. Tell it where <b>your own bike</b> is (with you, or parked at a station) and Shared bikes and On foot also use it for the stretch it can reach.</p>
    ${saved.length ? `<div class="group-title">Saved routes</div><div class="saved-list">${saved.map(savedItemHTML).join('')}</div>` : ''}
    ${trips.length ? `<div class="group-title">Recent trips</div><div class="trips">${trips.map((t, k) =>
      `<button type="button" class="trip" data-trip="${k}">${icon('history')}<span>${esc(t.from.name)} → ${esc(t.to.name)}</span></button>`).join('')}</div>` : ''}`;
}

// Short live status of a whole journey: cancelled, a broken connection, late, or on time.
function statusHTML(j) {
  if (j.cancelled) return '<span class="st cancel">Cancelled</span>';
  if (j.atRisk) return '<span class="st cancel">Connection at risk</span>';
  if (j.delay >= 1) return `<span class="st late">+${j.delay} min</span>`;
  if (j.transit.some(l => l.live?.isLive)) return '<span class="st ok">On time</span>';
  return '';
}

const dayLabel = d => {
  const today = new Date();
  return d.toDateString() === today.toDateString() ? '' : `${d.toLocaleDateString('en-GB', { weekday: 'short', day: 'numeric', month: 'short', timeZone: state.config.timezone })} · `;
};

function savedItemHTML(j) {
  return `<div class="saved-item${state.savedId === j.key ? ' on' : ''}">
    <button type="button" class="saved-open" data-saved="${esc(j.key)}">
      <span class="si-time">${esc(dayLabel(j.start))}<b>${fmtTime(j.start)} → ${fmtTime(j.end)}</b> ${statusHTML(j)}</span>
      <span class="si-route">${esc(j.from.name)} → ${esc(j.to.name)}</span>
    </button>
    <button type="button" class="icon-btn" data-unsave="${esc(j.key)}" title="Remove saved route" aria-label="Remove saved route">×</button></div>`;
}

// Header shortcut straight to the next saved route, plus a menu when there are more.
function renderSavedBar() {
  const list = savedJourneys();
  const menu = $('#savedMenu');
  if (!list.length) { $('#saved').innerHTML = ''; menu.hidden = true; return; }
  const next = nextSaved();
  $('#saved').innerHTML = `<button type="button" class="saved-next${state.savedId === next.key ? ' on' : ''}" data-saved="${esc(next.key)}" title="Open saved route">
      ${icon('bookmark')}<span class="sn-text"><b>${esc(dayLabel(next.start))}${fmtTime(next.start)}</b> ${esc(next.to.name)}</span>${statusHTML(next)}</button>`
    + (list.length > 1 ? `<button type="button" class="saved-more" data-action="saved-menu" aria-expanded="${!menu.hidden}" title="All saved routes">${list.length} ▾</button>` : '');
  if (list.length < 2) menu.hidden = true;
  if (!menu.hidden) menu.innerHTML = list.map(savedItemHTML).join('');
}

function segLabel(l, pct) {
  if (l.kind === 'transit') return pct > 7 ? esc(l.route.display_name || '') : '';
  if (l.kind === 'bike' || l.kind === 'scooter' || l.kind === 'cab') return pct > 4.5 ? icon(l.kind) : '';
  return '';
}

function stripHTML(j) {
  const t0 = j.start.getTime();
  const total = Math.max(j.end.getTime() - t0, 1);
  return `<div class="strip">${j.legs.map(l => {
    const left = ((l.dep - t0) / total) * 100;
    const width = Math.max(((l.arr - l.dep) / total) * 100, l.kind === 'walk' ? 0.6 : 1.6);
    return `<span class="seg-b ${l.kind}${l.kmh ? ' fast' : ''}" style="left:${left.toFixed(2)}%;width:${width.toFixed(2)}%;${blockColors(l)}">${segLabel(l, width)}</span>`;
  }).join('')}</div>`;
}

function metaHTML(j) {
  const parts = [];
  if (j.bikeMeters) parts.push(`<span class="m bike">${icon('bike')} ${fmtDist(j.bikeMeters)}</span>`);
  const first = j.transit[0];
  if (first) parts.push(`<span class="m">${pill(first.route)} ${fmtTime(first.dep)} · ${esc(first.stops[0]?.name || '')}</span>`);
  if (j.transit.length) parts.push(`<span class="m">${j.changes ? `${j.changes} change${j.changes > 1 ? 's' : ''}` : 'Direct'}</span>`);
  if (j.fastKmh) parts.push(`<span class="m fast">Ride at ${j.fastKmh} km/h to make it</span>`);
  for (const r of new Set(j.rentals.map(l => l.rental.systemName || l.rental.providerId))) parts.push(`<span class="m">${icon('share')} ${esc(r)}</span>`);
  if (j.walkMin && !j.ownBike) parts.push(`<span class="m">${icon('walk')} ${j.walkMin} min</span>`);
  return parts.join('');
}

function timesHTML(j, f) {
  return `<span class="tt"><b>${fmtTime(j.start)}</b><span class="arrow">→</span><b${j.delay >= 1 ? ' class="dly"' : ''}>${fmtTime(j.end)}</b>${f?.late ? `<em>+${f.lateMin} min</em>` : ''}${f?.best ? `<span class="best-tag">${state.timeMode === 'arrive' ? 'Latest departure' : 'Earliest arrival'}</span>` : ''}</span>`;
}

// ---------- timetable ----------

// Packs routes into lanes like a calendar, in departure order: each route takes the first lane
// that is free by the time it leaves. Done afresh on every render, so hiding or adding routes
// never leaves empty lanes; routes form a staircase, and later routes reuse lanes once the
// earlier ones are over.
function assignLanes(routes, o, ppm) {
  // Room for the captions: leave time above a column, arrival and total time below it.
  const before = (o === 'v' ? 22 : 0) / ppm * MIN;
  const after = (o === 'v' ? 42 : 14) / ppm * MIN;
  const laneEnds = [];
  const laneOf = new Map();
  for (const j of [...routes].sort((a, b) => a.start - b.start || a.end - b.end)) {
    let lane = laneEnds.findIndex(end => end <= +j.start - before);
    if (lane === -1) lane = laneEnds.length;
    laneEnds[lane] = +j.end + after;
    laneOf.set(j.key, lane);
  }
  return laneOf;
}

function blockHTML(l, nm, p, len, o, j) {
  const style = `--p:${p.toFixed(1)}px;--len:${Math.max(len, l.kind === 'walk' ? 1 : 2).toFixed(1)}px;${blockColors(l)}`;
  const title = esc(legTitle(l, nm, j));
  if (l.kind === 'walk') return `<div class="blk walk" style="${style}" title="${title}"></div>`;
  const tier = o === 'h' ? (len >= 60 ? 2 : len >= 30 ? 1 : 0) : (len >= 34 ? 2 : len >= 16 ? 1 : 0);
  const head = l.kind === 'transit' ? `<span>${esc(l.route.display_name || '')}</span>` : icon(l.kind === 'other' ? 'pin' : l.kind);
  const delay = l.kind === 'transit' && !l.cancelled && l.delay >= 1 ? `<em class="dly">+${l.delay}</em>` : '';
  // A ride you'd have to speed up for shows that speed, where it would show the duration or place.
  const kmh = l.kmh ? `${Math.ceil(l.kmh)} km/h` : '';
  const sub = kmh || nm.to;
  const body = tier === 0 ? delay : `<span class="l1">${head}<span>${tier === 1 && kmh ? kmh : shortDur(l.dur)}</span>${delay}</span>${tier === 2 && sub ? `<span class="l2">${esc(sub)}</span>` : ''}`;
  return `<div class="blk ${l.kind}${l.kmh ? ' fast' : ''}${l.cancelled ? ' cancelled' : ''}" style="${style}" title="${title}">${body}</div>`;
}

// Flags shown in a route's caption: saved, cancelled, broken connection, arrival delay.
function capStatus(j) {
  const bits = [];
  if (isSaved(j.key)) bits.push(`<span class="cap-saved" title="Saved">${icon('bookmark')}</span>`);
  if (j.cancelled) bits.push('<span class="cap-warn">✕ cancelled</span>');
  else if (j.atRisk) bits.push('<span class="cap-warn">⚠ connection</span>');
  return bits.join('');
}

function routeHTML(j, o, px, lane, f) {
  const p = px(+j.start);
  const len = px(+j.end) - p;
  const names = legNames(j);
  const blocks = j.legs.map((l, k) => blockHTML(l, names[k], px(+l.dep) - p, px(+l.arr) - px(+l.dep), o, j)).join('');
  const late = f.late ? ` <em>+${f.lateMin}′</em>` : '';
  const star = f.best ? '★ ' : '';
  const total = `<span class="total">${fmtDur(j.duration)}</span>`;
  const arrive = `<b${j.delay >= 1 ? ' class="dly"' : ''}>${fmtTime(j.end)}</b>`;
  const status = capStatus(j);
  const caps = o === 'h'
    ? `<div class="cap">${star}${status}${total} <b>${fmtTime(j.start)}</b> → ${arrive}${late}${j.transit.length ? ` · ${j.changes ? `${j.changes} change${j.changes > 1 ? 's' : ''}` : 'direct'}` : ''}</div>`
    : `<div class="cap cap-start">${star}${status}<b>${fmtTime(j.start)}</b></div><div class="cap cap-end">${arrive}${late}${total}</div>`;
  const cls = ['route', f.best && 'best', f.late && 'late', f.past && 'past', j.cancelled && 'cancelled', state.hot === j.key && 'hot'].filter(Boolean).join(' ');
  const label = `Leave ${fmtTime(j.start)}, arrive ${fmtTime(j.end)}, ${fmtDur(j.duration)}${j.fastKmh ? `, riding at ${j.fastKmh} km/h` : ''}`;
  return `<div class="${cls}" data-key="${esc(j.key)}" style="--p:${p.toFixed(1)}px;--len:${len.toFixed(1)}px;--lane:${lane}" role="button" tabindex="0" aria-label="${esc(label)}">${caps}<div class="track"></div>${blocks}</div>`;
}

function edgeHTML(dir, p, len) {
  const st = state.paging[dir];
  if (st === 'off') return '';
  const label = {
    loading: `<span class="spin"></span>Finding ${dir} routes…`,
    done: `No ${dir} routes found`,
    error: `Couldn't load ${dir} routes · retry`,
    idle: dir === 'later' ? 'Later routes ›' : '‹ Earlier routes',
  }[st];
  return `<div class="edge ${dir} is-${st}" style="--p:${p.toFixed(1)}px;--len:${len}px"><button type="button" data-more="${dir}"${st === 'loading' || st === 'done' ? ' disabled' : ''}>${label}</button></div>`;
}

function renderSchedule() {
  const el = $('#schedule');
  const s = state.sched;
  if (state.status === 'loading') {
    s.o = null;
    el.className = 'schedule';
    el.innerHTML = '<div class="sch-skel"><div></div><div></div><div></div><div></div></div>';
    return;
  }
  if (state.status !== 'ok') { s.o = null; el.innerHTML = ''; return; }
  const W = el.clientWidth;
  const H = el.clientHeight;
  if (!W || !H) return;

  const o = W >= 640 ? 'h' : 'v';
  const client = o === 'h' ? W : H;
  const routes = shownJourneys();
  // On a new search, zoom so a typical trip plus some slack takes most of the screen.
  if (s.fit.has(o) && routes.length) {
    s.fit.delete(o);
    const durs = routes.filter(j => j.transit.length).map(j => j.duration / 60).sort((a, b) => a - b);
    const typical = durs.length ? durs[Math.floor(durs.length / 2)] : 60;
    const want = (client * 0.78) / (typical + 30);
    state.ppm[o] = [...ZOOMS].reverse().find(z => z <= want) ?? ZOOMS[0];
    s.anchor = null;
  }
  const ppm = state.ppm[o];
  const flags = journeyFlags(routes);
  const target = targetTime();
  const now = Date.now();
  const ref = target ? +target : now;

  const Q = 15 * MIN;
  let t0 = Math.min(ref, ...routes.map(j => +j.start)) - 20 * MIN;
  let t1 = Math.max(ref, ...routes.map(j => +j.end)) + 20 * MIN;
  if (!target) t0 = Math.min(t0, now - 10 * MIN);
  t0 = Math.floor(t0 / Q) * Q;
  t1 = Math.ceil(t1 / Q) * Q;
  const E0 = state.paging.earlier === 'off' ? 0 : EDGE[o];
  // On a phone the search pill floats over the bottom of the screen; leave room under the
  // "Later routes" button so it can be scrolled clear of the pill.
  const E1 = EDGE[o] + (o === 'v' ? 80 : 0);
  const px = t => E0 + ((t - t0) / MIN) * ppm;
  const mainLen = px(t1) + E1;

  const laneOf = assignLanes(routes, o, ppm);
  const lanes = Math.max(1, ...routes.map(j => laneOf.get(j.key) + 1));
  const laneW = o === 'v' ? Math.max(72, Math.min(130, (W - AXIS_W - 10) / lanes)) : 0;
  // The canvas is at least as tall as the viewport through CSS, so the search bar sliding away
  // (which changes only the height) needs no re-render.
  const cross = o === 'h' ? lanes * LANE_H + 16 : Math.max(lanes * laneW + 10, W - AXIS_W);

  let ticks = '';
  let grid = '';
  const every = ppm * 15 >= 44 ? 15 : ppm * 30 >= 44 ? 30 : 60;
  for (let t = t0; t <= t1; t += Q) {
    const label = fmtTime(new Date(t));
    const mm = +label.slice(3);
    grid += `<div class="gl${mm === 0 ? ' hour' : ''}" style="--p:${px(t)}px"></div>`;
    if (mm % every === 0) ticks += `<span class="tick${mm === 0 ? ' hour' : ''}" style="--p:${px(t)}px">${label}</span>`;
  }

  let marks = '';
  let lines = '';
  const mark = (cls, t, text) => {
    if (t < t0 || t > t1) return;
    marks += `<span class="amark ${cls}" style="--p:${px(t)}px">${text}</span>`;
    lines += `<div class="mline ${cls}" style="--p:${px(t)}px"></div>`;
  };
  const shade = (a, b) => (b > a ? `<div class="shade" style="--p:${a}px;--len:${b - a}px"></div>` : '');
  let shades = '';
  if (state.timeMode === 'now') shades = shade(0, px(now));
  else if (state.timeMode === 'depart') shades = shade(E0, px(+target));
  else shades = shade(px(+target), px(t1));
  mark('now', now, o === 'h' ? `Now ${fmtTime(new Date(now))}` : fmtTime(new Date(now)));
  if (target) mark('target', +target, o === 'h' ? `${state.timeMode === 'arrive' ? 'Arrive by' : 'Depart'} ${fmtTime(target)}` : fmtTime(target));

  const edges = (E0 ? edgeHTML('earlier', 0, E0) : '') + edgeHTML('later', px(t1), E1);
  const body = routes.map(j => routeHTML(j, o, px, laneOf.get(j.key), flags.get(j.key))).join('');

  const canvasStyle = o === 'h' ? `width:${mainLen}px;height:${AXIS_H + cross}px` : `width:${AXIS_W + cross}px;height:${mainLen}px;--laneW:${laneW}px`;
  const axisStyle = o === 'h' ? `width:${mainLen}px` : `height:${mainLen}px`;
  const bodyStyle = o === 'h' ? `width:${mainLen}px` : `height:${mainLen}px`;
  el.className = `schedule ${o}${state.hot ? ' has-hot' : ''}`;
  el.innerHTML = `<div class="sch-canvas" style="${canvasStyle}">
    <div class="sch-axis" style="${axisStyle}">${ticks}${marks}</div>
    <div class="sch-body" style="${bodyStyle}">${grid}${shades}${edges}${body}${lines}</div></div>`;

  // Keep the same moment at the start of the viewport across re-renders and page loads.
  Object.assign(s, { o, ppm, t0, E0 });
  if (s.anchor == null) {
    // Arrive-by: the deadline sits near the far end so on-time routes are fully in view.
    const lead = state.timeMode === 'arrive' ? (client * 0.82) / ppm : state.timeMode === 'depart' ? 10 : 6;
    s.anchor = ref - lead * MIN;
  }
  const pos = Math.max(0, px(s.anchor));
  if (o === 'h') el.scrollLeft = pos; else el.scrollTop = pos;
  // Remember where we put it, so the scroll event this causes isn't mistaken for the user.
  s.expect = { x: el.scrollLeft, y: el.scrollTop };
  s.last = { ...s.expect };
}

function zoom(step) {
  const s = state.sched;
  const o = s.o;
  if (!o) return;
  const el = $('#schedule');
  const client = o === 'h' ? el.clientWidth : el.clientHeight;
  const center = s.anchor + (client / 2 / s.ppm) * MIN;
  const nearest = ZOOMS.reduce((best, z, k) => (Math.abs(z - state.ppm[o]) < Math.abs(ZOOMS[best] - state.ppm[o]) ? k : best), 0);
  const next = ZOOMS[Math.min(ZOOMS.length - 1, Math.max(0, nearest + step))];
  if (next === state.ppm[o]) return;
  state.ppm[o] = next;
  s.anchor = center - (client / 2 / next) * MIN;
  renderSchedule();
  renderViewbar();
}

function setHot(key) {
  if (state.hot === key) return;
  state.hot = key;
  document.querySelectorAll('[data-key]').forEach(n => n.classList.toggle('hot', n.dataset.key === key));
  $('#schedule').classList.toggle('has-hot', !!key);
}

function openDetail(key) {
  if (!state.byKey.has(key)) return;
  state.selected = key;
  state.savedId = null;
  state.editing = false;
  setHot(null);
  writeHash();
  render();
  $('#detail').scrollTop = 0;
  liveThenRender([state.byKey.get(key)]);
}

// ---------- detail ----------

// A ride's departure time: struck-through timetable time when it is late or cancelled.
function depTimeHTML(l) {
  if (l.cancelled) return `<time class="cancel"><s>${fmtTime(l.schedDep || l.dep)}</s></time>`;
  if (l.delay >= 1) return `<time class="late">${fmtTime(l.dep)}<s>${fmtTime(l.schedDep)}</s></time>`;
  return `<time class="${l.live?.isLive ? 'ontime' : ''}">${fmtTime(l.dep)}</time>`;
}

function liveLabel(l) {
  if (l.cancelled) return '<span class="st cancel">Cancelled</span>';
  if (l.delay >= 1) return `<span class="st late">${l.delay} min late</span>`;
  if (l.live?.isLive) return '<span class="st ok">On time</span>';
  return '';
}

// Platforms come with live data, shortly before departure; trains say so until then.
function trackHTML(l) {
  if (l.live?.platform) return `<span class="trk${l.live.platformChanged ? ' changed' : ''}">Track ${esc(l.live.platform)}</span>`;
  return '';
}

// The body of one segment, between its start and end bubbles.
function legHTML(l, j) {
  let body;
  if (l.kind === 'transit') {
    const subs = [liveLabel(l), esc(l.route.agency || ''), l.trainNo && `train ${esc(l.trainNo)}`, j.ownBike && l.bikesAllowed === false && '<span class="st cancel">No bikes on board</span>'].filter(Boolean);
    const between = l.stops.slice(1, -1);
    // Timetable times along the way, moved by the ride's current delay.
    const shift = l.schedDep && l.dep ? l.dep - l.schedDep : 0;
    const stopsList = between.map(s => `<li><time>${s.sched ? fmtTime(new Date(+s.sched + shift)) : ''}</time>${esc(s.name)}</li>`).join('');
    body = `<div class="tl-title">${icon(l.vehicle)} ${pill(l.route)} ${l.headsign ? `<span>towards ${esc(l.headsign)}</span>` : ''}</div>
      ${subs.length ? `<div class="tl-sub">${subs.join(' · ')}</div>` : ''}
      ${between.length
        ? `<details class="stops"><summary>${between.length} stop${between.length > 1 ? 's' : ''} · ${fmtDur(l.dur)}</summary><ol>${stopsList}</ol></details>`
        : `<div class="stops">Non-stop · ${fmtDur(l.dur)}</div>`}`;
  } else {
    const titles = { bike: 'Ride your bike', scooter: 'Ride a scooter', cab: 'Taxi or on-demand ride', walk: j.ownBike ? 'Walk with your bike' : 'Walk', other: titleCase(l.mode || 'Travel') };
    const r = l.rental;
    const title = r ? `${esc(r.systemName || r.providerId)}${r.electric ? ' e-bike' : ''}` : titles[l.kind];
    const rule = r ? ({ ROUNDTRIP_STATION: 'Round trip: bring it back to this station', ANY_STATION: 'Park at any station or hub' }[r.returnConstraint]
      || (r.toStationName ? `Park at ${esc(r.toStationName)}` : ''))
      // Your own bike without the folding bike's ticket stays at the station.
      : l.kind !== 'bike' || j.ownBike ? ''
      : j.legs.slice(j.legs.indexOf(l) + 1).some(x => x.kind === 'transit') ? (l.toName ? `Leave it at ${esc(l.toName)}` : '')
      : j.legs.slice(0, j.legs.indexOf(l)).some(x => x.kind === 'transit') ? 'Where you parked it' : '';
    const detail = [l.distance ? fmtDist(l.distance) : null, fmtDur(l.dur), r?.electric && `timed at ${EBIKE_KMH} km/h`, l.kmh && `<b class="fast">${Math.ceil(l.kmh)} km/h, faster than your usual ${RIDE_KMH}</b>`].filter(Boolean).join(' · ');
    body = `<div class="tl-title">${icon(l.kind === 'other' ? 'pin' : l.kind)} ${title}</div><div class="tl-sub">${detail}</div>${rule ? `<div class="tl-sub">${rule}</div>` : ''}`;
  }
  return `<li class="tl-leg ${l.kind}${l.cancelled ? ' cancelled' : ''}" style="--c:${esc(legColor(l))}"><time></time><span class="mark"></span><div class="tl-body">${body}</div></li>`;
}

// A bubble where a segment starts or ends. The segment's line runs out of a start bubble and
// into an end bubble; the journey's origin and destination get their own markers.
function stopHTML({ leg = null, side, cls = '', time = null, timeHTML = '', timeCls = '', name = '', sub = '', extra = '' }) {
  const rail = leg ? `<i class="rail-${side === 'start' ? 'out' : 'in'}${leg.kind === 'walk' ? ' dotted' : ''}"></i>` : '';
  return `<li class="tl-stop ${side} ${cls}" style="--c:${esc(leg ? legColor(leg) : 'var(--faint)')}">${timeHTML || `<time class="${timeCls}">${time ? fmtTime(time) : ''}</time>`}<span class="mark">${rail}</span>
    <div class="tl-place"><b>${esc(name)}</b>${extra}${sub ? `<small>${esc(sub)}</small>` : ''}</div></li>`;
}

// The space between two segments: as tall as the time it leaves you (within limits), labelled
// with that time. Walks through stations happen in here instead of getting their own row.
function gapHTML(prev, next, walkSec) {
  const sec = (next.dep - prev.arr) / 1000;
  const min = Math.round(sec / 60);
  const height = Math.max(22, Math.min(120, 16 + Math.max(0, min) * 3));
  let label;
  if (next.atRisk) {
    label = `<span class="warn">⚠ You get there at ${fmtTime(prev.arr)}, but the ${esc(next.route?.display_name || 'next ride')} leaves at ${fmtTime(next.dep)}</span>`;
  } else {
    const what = prev.kind === 'transit' && next.kind === 'transit' ? 'change'
      : next.kind === 'transit' ? 'to the platform'
      : prev.kind === 'transit' ? 'out of the station'
      : walkSec ? 'walk' : '';
    const walk = walkSec >= 4.5 * 60 && what !== 'walk' ? ` · incl. ${Math.round(walkSec / 60)} min walk` : '';
    label = `<span class="gap-label">${min < 1 ? 'Just in time' : `${fmtDur(sec)}${what ? ` ${what}` : ''}${walk}`}</span>`;
  }
  return `<li class="tl-gap" style="--h:${height}px"><time></time><span class="mark"></span><div class="tl-gap-body">${label}</div></li>`;
}

// Walks with your bike, changes and short walks aren't segments of their own.
const isSegment = (l, j) => l.kind !== 'walk' || (!j.ownBike && !l.transfer && l.dur >= 5 * 60);

function renderDetail(j) {
  const f = j.saved ? null : journeyFlags(shownJourneys()).get(j.key);
  const braid = braidOf(j);
  const braided = state.detailView === 'braid';
  const saved = isSaved(j.key);
  const hasLive = j.transit.some(l => l.live?.isLive);
  const liveNote = hasLive
    ? `<span class="live-dot"></span>Live times · updated ${fmtTime(j.liveAt)}`
    : 'Timetable times · live times appear closer to departure';
  const back = `<button type="button" class="back" data-action="back">${icon('back')} ${j.saved ? 'Back' : 'Schedule'}</button>`;
  const variants = braid.strands.length + braid.ghosts.length;
  const views = `<div class="seg d-view" role="group" aria-label="Show this route as">
      <button type="button" data-show="list" aria-pressed="${!braided}">Stops</button>
      <button type="button" data-show="braid" aria-pressed="${braided}">Variants${variants ? ` <span class="count">${variants}</span>` : ''}</button></div>`;

  // Keep expanded stop lists open when the same route re-renders after a live refresh.
  const el = $('#detail');
  const open = el.dataset.shown === j.key ? [...el.querySelectorAll('details')].map(d => d.open) : [];
  el.dataset.shown = j.key;
  el.innerHTML = `<div class="d-top">${back}
      <span class="d-btns"><button type="button" class="save-btn" data-action="toggle-save" aria-pressed="${saved}">${icon('bookmark')} ${saved ? 'Saved' : 'Save'}</button>
      <button type="button" class="ride-btn" data-action="ride-start">${icon('train')} Ride</button></span></div>
    <div class="d-head">
      ${j.saved ? `<div class="d-route">${esc(dayLabel(j.start))}${esc(j.from.name)} → ${esc(j.to.name)}</div>` : ''}
      <div class="card-head">${timesHTML(j, f)}<span class="dur">${fmtDur(j.duration)}</span></div>
      ${braided ? '' : `${stripHTML(j)}<div class="d-chips">${metaHTML(j)}</div>`}
      ${j.notes.length ? `<ul class="d-notes">${j.notes.map(n => `<li>${esc(n)}</li>`).join('')}</ul>` : ''}
      <div class="d-live">${statusHTML(j)} ${liveNote}</div>
      ${views}
    </div>
    ${braided ? braidHTML(j, braid, el.clientWidth - 32) : `<ol class="tl">${stopsHTML(j)}</ol>`}`;
  el.querySelectorAll('details').forEach((d, k) => { if (open[k]) d.open = true; });
}

// The route as a list: a bubble per place, its rides between them, the time you have at each change.
function stopsHTML(j) {
  const names = legNames(j);
  const legs = j.legs;
  let segs = legs.map((l, k) => ({ l, k })).filter(({ l }) => isSegment(l, j));
  if (!segs.length) segs = legs.map((l, k) => ({ l, k }));
  const walked = (from, to) => sum(legs.slice(from, to).map(l => l.dur));
  const first = segs[0];
  const last = segs[segs.length - 1];
  const rows = [];

  if (first.k > 0) {
    rows.push(stopHTML({ side: 'end', cls: 'origin', time: j.start, name: j.from.name, sub: j.from.address }));
    rows.push(gapHTML({ arr: j.start, kind: 'origin' }, first.l, walked(0, first.k)));
  }
  segs.forEach(({ l, k }, i) => {
    if (i > 0) rows.push(gapHTML(segs[i - 1].l, l, walked(segs[i - 1].k + 1, k)));
    const origin = i === 0 && first.k === 0;
    const destination = i === segs.length - 1 && last.k === legs.length - 1;
    rows.push(stopHTML({
      leg: l, side: 'start', cls: origin ? 'origin' : '',
      timeHTML: l.kind === 'transit' ? depTimeHTML(l) : '', time: l.dep,
      name: origin ? j.from.name : names[k].from, sub: origin ? j.from.address : '',
      extra: l.kind === 'transit' ? trackHTML(l) : '',
    }));
    rows.push(legHTML(l, j));
    rows.push(stopHTML({
      leg: l, side: 'end', cls: destination ? 'destination' : '', time: l.arr,
      timeCls: (l.kind === 'transit' && l.delay >= 1) || (destination && j.delay >= 1) ? 'late' : '',
      name: destination ? j.to.name : names[k].to, sub: destination ? j.to.address : '',
    }));
  });
  if (last.k < legs.length - 1) {
    rows.push(gapHTML(last.l, { dep: j.end, kind: 'destination' }, walked(last.k + 1, legs.length)));
    rows.push(stopHTML({ side: 'start', cls: 'destination', time: j.end, timeCls: j.delay >= 1 ? 'late' : '', name: j.to.name, sub: j.to.address }));
  }
  return rows.join('');
}

// ---------- variants: the braided lane ----------
//
// The open route as one lane, to scale, with the other routes of this search braided in where
// they differ from it (canvas "Variations of one route", board 1). A strand leaves the lane where
// another route gets off one of its vehicles and rejoins it where that route gets back on: the
// bike to Geel joins the IC at Geel. Routes that share no vehicle but leave from one of its
// stations show as dashed blocks: the other rides from there. Faded means beaten: you'd leave
// earlier and get home no sooner. Tapping a strand or a dashed block opens that route.

// How far before and after the route a strand may reach, and how close to the route's own
// departure another ride from the same station has to leave.
const BRAID_BEFORE = 75 * MIN;
const BRAID_AFTER = 100 * MIN;
const BRAID_GHOST = 90 * MIN;
const sameStop = (a, b) => !!a?.name && a.name.toLowerCase() === (b?.name || '').toLowerCase();

// The other routes of the current search; a saved route from another search has none.
function braidPool(j) {
  if (j.saved && !(j.tab === state.tab && samePlace(j.from, state.from) && samePlace(j.to, state.to))) return [];
  return state.journeys.filter(o => o.key !== j.key && o.transit.length && !o.cancelled && !o.atRisk);
}

// 'worse': leave no later and get home no sooner; 'better': the reverse; 'trade': one of each.
// A couple of minutes either way is the same (a Donkey a minute slower than the Blue-bike isn't beaten).
const BRAID_SAME = 2;
function verdictOf(o, j) {
  const t = d => Math.round(+d / MIN);
  const near = m => (Math.abs(m) <= BRAID_SAME ? 0 : m);
  const ds = near(t(o.start) - t(j.start));
  const de = near(t(o.end) - t(j.end));
  if (ds <= 0 && de >= 0 && (ds || de)) return 'worse';
  if (ds >= 0 && de <= 0 && (ds || de)) return 'better';
  return ds || de ? 'trade' : 'same';
}

// Where the other routes differ from j. Strands come from routes sharing at least one vehicle
// with j: what they do before the first shared ride, between two, and after the last, wherever
// that isn't what j does. Ghosts come from routes sharing none: their first ride from a station
// j leaves from. Each strand or ghost keeps its route `o`, the one to switch to.
function braidOf(j) {
  const mine = new Map();
  j.legs.forEach((l, k) => { if (l.kind === 'transit') mine.set(tripOf(l), k); });
  const sig = legs => legs.filter(l => l.kind !== 'walk').map(legPart).join('>');
  const t = d => Math.round(+d / MIN);
  const found = [];
  const ghosts = new Map();
  for (const o of braidPool(j)) {
    const verdict = verdictOf(o, j);
    const shared = [];
    o.legs.forEach((l, k) => { if (l.kind === 'transit' && mine.has(tripOf(l))) shared.push(k); });
    if (!shared.length) {
      for (const l of o.transit) {
        const at = j.transit.find(m => sameStop(m.stops[0], l.stops[0]) && Math.abs(l.dep - m.dep) <= BRAID_GHOST);
        if (!at) continue;
        // The same ride can start several routes; keep the one that gets you home first.
        const prev = ghosts.get(tripOf(l));
        if (!prev || o.end < prev.o.end) ghosts.set(tripOf(l), { o, verdict, leg: l, from: l.dep, to: l.arr });
        break;
      }
      continue;
    }
    const jk = shared.map(k => mine.get(tripOf(o.legs[k])));
    if (jk.some((k, i) => i && k <= jk[i - 1])) continue;
    const runs = [];
    const first = o.legs[shared[0]];
    if (sig(o.legs.slice(0, shared[0])) !== sig(j.legs.slice(0, jk[0])) || !sameStop(first.stops[0], j.legs[jk[0]].stops[0])) {
      runs.push({ type: 'before', legs: o.legs.slice(0, shared[0]), from: o.start, to: first.dep, join: { k: jk[0], t: first.dep, stop: first.stops[0] } });
    }
    for (let i = 1; i < shared.length; i++) {
      const a = o.legs[shared[i - 1]];
      const b = o.legs[shared[i]];
      const legs = o.legs.slice(shared[i - 1] + 1, shared[i]);
      if (sig(legs) === sig(j.legs.slice(jk[i - 1] + 1, jk[i])) && sameStop(a.stops.at(-1), j.legs[jk[i - 1]].stops.at(-1)) && sameStop(b.stops[0], j.legs[jk[i]].stops[0])) continue;
      runs.push({ type: 'between', legs, from: a.arr, to: b.dep, leave: { k: jk[i - 1], t: a.arr, stop: a.stops.at(-1) }, join: { k: jk[i], t: b.dep, stop: b.stops[0] } });
    }
    const last = o.legs[shared.at(-1)];
    if (sig(o.legs.slice(shared.at(-1) + 1)) !== sig(j.legs.slice(jk.at(-1) + 1)) || !sameStop(last.stops.at(-1), j.legs[jk.at(-1)].stops.at(-1))) {
      runs.push({ type: 'after', legs: o.legs.slice(shared.at(-1) + 1), from: last.arr, to: o.end, leave: { k: jk.at(-1), t: last.arr, stop: last.stops.at(-1) } });
    }
    found.push({ o, verdict, runs });
  }
  // A route that differs from j in one place only is the one to switch to for that difference.
  found.sort((a, b) => a.runs.length - b.runs.length || a.o.end - b.o.end);
  const seen = new Set();
  const strands = [];
  for (const { o, verdict, runs } of found) {
    for (const r of runs) {
      if (!r.legs.length || r.to - r.from < MIN || r.from < j.start - BRAID_BEFORE || r.to > +j.end + BRAID_AFTER) continue;
      const id = [r.type, sig(r.legs), r.join?.stop?.name, r.leave?.stop?.name, t(r.from), t(r.to)].join('|');
      if (seen.has(id)) continue;
      seen.add(id);
      strands.push({ ...r, o, verdict });
    }
  }
  return { strands, ghosts: [...ghosts.values()] };
}

const box = (x, y, w, h) => `left:${x.toFixed(1)}px;top:${y.toFixed(1)}px;width:${w.toFixed(1)}px;height:${h.toFixed(1)}px;`;

// What a block in the braid says: the line and when it leaves, or the mode, how long and where
// to. `short` keeps just what fits a small block.
function braidLabel(l, nm, short = false) {
  if (l.kind === 'transit') return short ? esc(l.route.display_name || '') : `${esc(l.route.display_name || '')} ${fmtTime(l.dep)}`;
  if (short) return shortDur(l.dur);
  const what = l.rental ? esc(l.rental.systemName || l.rental.providerId) : { bike: 'Bike', scooter: 'Scooter', cab: 'Taxi', walk: 'Walk', other: 'Travel' }[l.kind];
  return `${what} ${shortDur(l.dur)}${nm?.to ? ` → ${esc(nm.to)}` : ''}`;
}

function braidHTML(j, { strands, ghosts }, width) {
  const W = Math.max(300, width || 358);
  const AX = 36;
  const MIN_MAIN = 92;
  const COL = 30;
  const STEP = 36;
  const ppm = Math.min(6, Math.max(2, 520 / ((j.end - j.start) / MIN + 20)));
  const pad = px => (px / ppm) * MIN;

  // Strands that bring you to the route go left of it, the rest right; each side packs them
  // into columns like the timetable packs routes into lanes, with room for their captions.
  const pack = (items, above, below) => {
    const cols = [];
    for (const it of items) {
      const a = +it.from - pad(above);
      const b = +it.to + pad(below);
      let c = cols.findIndex(col => col.every(([x, y]) => b <= x || a >= y));
      if (c === -1) { c = cols.length; cols.push([]); }
      cols[c].push([a, b]);
      it.col = c;
    }
    return cols.length;
  };
  const order = (a, b) => (a.verdict === 'worse') - (b.verdict === 'worse') || a.from - b.from;
  const left = strands.filter(s => s.type === 'before').sort(order);
  const right = [...strands.filter(s => s.type !== 'before').sort(order), ...[...ghosts].sort(order)];
  const nl = pack(left, 18, 4);
  const nr = pack(right, 4, 18);
  const cap = Math.max(2, Math.floor((W - AX - MIN_MAIN - 12) / STEP));
  const lc = Math.min(nl, cap - Math.min(nr, Math.floor(cap / 2)));
  const rc = Math.min(nr, cap - lc);
  // The lane takes what the strands leave, so its blocks can say more.
  const MAIN = Math.min(150, W - AX - 12 - (lc + rc) * STEP);
  const shownL = left.filter(s => s.col < lc);
  const shownR = right.filter(s => s.col < rc);
  const hidden = left.length + right.length - shownL.length - shownR.length;

  const items = [...shownL, ...shownR];
  const t0 = Math.min(+j.start, ...items.map(i => +i.from)) - 14 * MIN;
  const t1 = Math.max(+j.end, ...items.map(i => +i.to)) + 16 * MIN;
  const px = t => ((t - t0) / MIN) * ppm;
  const mainX = items.length ? AX + lc * STEP + 6 : AX + (W - AX - MAIN) / 2;
  const leftX = c => mainX - 6 - COL - c * STEP;
  const rightX = c => mainX + MAIN + 6 + c * STEP;

  let grid = '';
  const Q = 15 * MIN;
  const every = ppm * 15 >= 40 ? 15 : 30;
  for (let t = Math.ceil(t0 / Q) * Q; t <= t1; t += Q) {
    const label = fmtTime(new Date(t));
    const mm = +label.slice(3);
    grid += `<div class="b-gl${mm === 0 ? ' hour' : ''}" style="top:${px(t).toFixed(1)}px">${mm % every === 0 ? `<span>${label}</span>` : ''}</div>`;
  }

  // The route itself: its rides to scale, waits as dotted lines, the spare time at each change,
  // and a tick wherever a strand gets on or off at a stop along the way.
  const names = legNames(j);
  const ticks = new Map();
  for (const s of items) {
    for (const [p, end] of [[s.join, 0], [s.leave, -1]]) {
      if (!p || sameStop(p.stop, j.legs[p.k].stops.at(end))) continue;
      const list = ticks.get(p.k) || [];
      if (!list.some(x => sameStop(x.stop, p.stop))) list.push(p);
      ticks.set(p.k, list);
    }
  }
  let main = '';
  j.legs.forEach((l, k) => {
    const y = px(+l.dep);
    const h = Math.max(px(+l.arr) - y, 2);
    const next = j.legs[k + 1];
    if (next && next.dep - l.arr >= MIN) main += `<div class="b-wait" style="left:${(mainX + MAIN / 2 - 1).toFixed(1)}px;top:${px(+l.arr).toFixed(1)}px;height:${(px(+next.dep) - px(+l.arr)).toFixed(1)}px"></div>`;
    // Changes and short walks are a thin bar; a walk that is a stretch of its own gets a block.
    if (l.kind === 'walk' && !isSegment(l, j)) {
      main += `<div class="bb walk" style="${box(mainX + MAIN / 2 - 4, y, 8, h)}" title="${esc(legTitle(l, names[k], j))}"></div>`;
      return;
    }
    const delay = l.kind === 'transit' && !l.cancelled && l.delay >= 1 ? ` <em class="dly">+${l.delay}</em>` : '';
    const head = h >= 15 ? `<span class="l1">${l.kind === 'transit' ? `<b>${esc(l.route.display_name || '')}</b> ${fmtTime(l.dep)}` : `${icon(l.kind === 'other' ? 'pin' : l.kind)} ${shortDur(l.dur)}`}${delay}</span>` : '';
    const sub = h >= 30 && names[k].to ? `<span class="l2">→ ${esc(names[k].to)}</span>` : '';
    const marks = (ticks.get(k) || []).map(p => `<i class="b-tick" style="top:${(px(+p.t) - y).toFixed(1)}px"><span>${esc(p.stop.name)} ${fmtTime(p.t)}</span></i>`).join('');
    main += `<div class="bb ${l.kind}${l.kind === 'walk' ? ' wide' : ''}${l.kmh ? ' fast' : ''}${l.cancelled ? ' cancelled' : ''}" style="${box(mainX, y, MAIN, h)}${blockColors(l)}" title="${esc(legTitle(l, names[k], j))}">${head}${sub}${marks}</div>`;
  });
  // At each change, the time it gives you; orange when that's all walking (MOTIS pads a change
  // to the station's minimum transfer time with a walk), red when it's too short.
  j.transit.forEach((b, i) => {
    if (!i) return;
    const a = j.transit[i - 1];
    const walk = sum(j.legs.slice(j.legs.indexOf(a) + 1, j.legs.indexOf(b)).filter(l => l.kind === 'walk').map(l => l.dur));
    const gap = (b.dep - a.arr) / 1000;
    const cls = b.atRisk || gap < walk ? 'bad' : gap - walk < 300 ? 'tight' : 'ok';
    const y = (px(+a.arr) + px(+b.dep)) / 2 - 7;
    const title = `${mins(Math.max(0, gap))} to change${walk >= 60 ? `, ${mins(walk)} of it walking` : ''}`;
    main += `<span class="b-spare ${cls}" style="left:${(mainX + MAIN / 2 + 6).toFixed(1)}px;top:${y.toFixed(1)}px" title="${title}">${b.atRisk ? '⚠' : mins(Math.max(0, gap))}</span>`;
  });
  main += `<span class="b-cap main" style="left:${mainX.toFixed(1)}px;width:${MAIN}px;top:${(px(+j.start) - 18).toFixed(1)}px">${fmtTime(j.start)}</span>`;
  main += `<span class="b-cap main${j.delay >= 1 ? ' dly' : ''}" style="left:${mainX.toFixed(1)}px;width:${MAIN}px;top:${(px(+j.end) + 3).toFixed(1)}px">${fmtTime(j.end)}</span>`;

  const strandHTML = (s, x) => {
    const y0 = px(+s.from);
    const len = Math.max(px(+s.to) - y0, 6);
    const nm = legNames(s.o);
    const lead = s.legs.filter(l => l.kind !== 'walk').sort((a, b) => b.dur - a.dur)[0] || [...s.legs].sort((a, b) => b.dur - a.dur)[0];
    let inner = '';
    const stops = [+s.from, ...s.legs.flatMap(l => [+l.dep, +l.arr]), +s.to];
    for (let i = 0; i < stops.length; i += 2) {
      if (stops[i + 1] - stops[i] >= MIN) inner += `<i class="b-wait" style="left:${COL / 2 - 1}px;top:${(px(stops[i]) - y0).toFixed(1)}px;height:${(px(stops[i + 1]) - px(stops[i])).toFixed(1)}px"></i>`;
    }
    for (const l of s.legs) {
      const y = px(+l.dep) - y0;
      const h = Math.max(px(+l.arr) - px(+l.dep), 2);
      const label = l === lead && h >= 22 ? `<span class="s-lbl">${braidLabel(l, nm[s.o.legs.indexOf(l)], h < 60)}</span>` : '';
      inner += l.kind === 'walk' && l !== lead
        ? `<span class="bb walk" style="${box(COL / 2 - 3, y, 6, h)}"></span>`
        : `<span class="bb ${l.kind}${l.kind === 'walk' ? ' wide' : ''}" style="${box(0, y, COL, h)}${blockColors(l)}">${label}</span>`;
    }
    // Dashed links to the lane, at the moment the strand gets on or off one of its rides.
    const link = (p, side) => {
      const y = px(+p.t) - y0;
      return side === 'in'
        ? `<i class="b-conn in" style="left:${COL}px;width:${(mainX - x - COL).toFixed(1)}px;top:${y.toFixed(1)}px"></i>`
        : `<i class="b-conn out" style="left:${(mainX + MAIN - x).toFixed(1)}px;width:${(x - mainX - MAIN).toFixed(1)}px;top:${y.toFixed(1)}px"></i>`;
    };
    if (s.type === 'before') inner += link(s.join, 'in');
    else {
      inner += link(s.leave, 'out');
      if (s.join) inner += link(s.join, 'out');
    }
    if (s.type === 'before') inner += `<span class="s-cap" style="bottom:${len.toFixed(1)}px">${fmtTime(s.o.start)}</span>`;
    if (s.type === 'after') inner += `<span class="s-cap ${s.verdict}" style="top:${len.toFixed(1)}px">${fmtTime(s.o.end)}</span>`;
    const what = s.legs.filter(l => l.kind !== 'walk' || s.legs.length === 1).map(l => braidLabel(l, nm[s.o.legs.indexOf(l)])).join(', ');
    const label = `${what}: leave ${fmtTime(s.o.start)}, home ${fmtTime(s.o.end)}. Open this route`;
    return `<button type="button" class="strand ${s.type} ${s.verdict}" data-variant="${esc(s.o.key)}" style="${box(x, y0, COL, len)}" title="${label}" aria-label="${label}">${inner}</button>`;
  };
  const ghostHTML = (g, x) => {
    const y0 = px(+g.from);
    const len = Math.max(px(+g.to) - y0, 24);
    const label = `${braidLabel(g.leg)} from ${esc(g.leg.stops[0].name)}: home ${fmtTime(g.o.end)}. Open this route`;
    return `<button type="button" class="ghost ${g.verdict}" data-variant="${esc(g.o.key)}" style="${box(x, y0, COL, len)}--c:${esc(g.leg.route.ui_color || g.leg.route.color || '#555')}" title="${label}" aria-label="${label}">`
      + `<span class="s-lbl">${braidLabel(g.leg, null, len < 60)}</span><span class="s-cap ${g.verdict}" style="top:${len.toFixed(1)}px">${fmtTime(g.o.end)}</span></button>`;
  };
  const side = shownL.map(s => strandHTML(s, leftX(s.col))).join('')
    + shownR.map(s => (s.leg ? ghostHTML(s, rightX(s.col)) : strandHTML(s, rightX(s.col)))).join('');

  const note = items.length
    ? `Strands: other ways to do part of this route. Dashed: other rides from its stations. Faded: you'd leave earlier and get home no sooner. Tap one to open it.${hidden ? ` ${hidden} more don't fit.` : ''}`
    : 'No other route in this search shares a ride or a station with this one. Earlier and later routes you load in the timetable show up here too.';
  return `<p class="b-note">${note}</p><div class="braid" style="height:${px(t1).toFixed(0)}px">${grid}${main}${side}</div>`;
}

// ---------- ride mode ----------
//
// While you travel: one big countdown to your next move, a strip from now to arrival, and
// for every change still ahead the spare time (grey = the walk between platforms) and what
// to do if you miss it. Live times refresh every RIDE_REFRESH; plan B and bike counts every
// few minutes. The phone buzzes only when one of three facts changes (next move, spare time
// at the next change, arrival), plus once when it's time to get off.

const RIDE_REFRESH = 30 * 1000;
const RIDE_EXTRAS_EVERY = 3 * MIN;
const SPARE_SCALE_MIN = 15; // a spare bar's full width
const GET_OFF_WARN = 2 * MIN;

const rideState = { extras: null, extrasAt: 0, extrasKey: '', loading: false, facts: null, change: null, warned: new Set() };

function currentRide() {
  if (!state.rideId) return null;
  const entry = savedEntries().find(s => s.id === state.rideId);
  return entry ? savedJourney(entry) : null;
}

const legLabel = l => (l.kind === 'transit' ? `${l.route.display_name || ''}${l.headsign ? ` to ${l.headsign}` : ''}`
  : l.rental ? (l.rental.systemName || l.rental.providerId) : { bike: 'your bike', walk: 'walk', scooter: 'scooter' }[l.kind] || l.kind);

// Where you are in the journey (by the clock) and what comes next.
function rideModel(j, now = Date.now()) {
  const legs = j.legs;
  const names = legNames(j);
  const transit = legs.map((l, k) => k).filter(k => legs[k].kind === 'transit');
  const cur = legs.findIndex(l => now < +l.arr);
  const between = (a, b) => sum(legs.slice(a + 1, b).map(l => l.dur));
  const after = k => {
    const n = legs.slice(k + 1).find(l => !l.transfer && (l.kind !== 'walk' || l.dur >= 3 * 60));
    if (!n) return k === legs.length - 1 ? 'you\'ve arrived' : '';
    if (n.kind === 'transit') {
      const walk = between(k, legs.indexOf(n));
      return `${walk >= 60 ? `walk ${Math.round(walk / 60)}′ to the` : 'change to the'} ${n.route.display_name || 'next ride'}`;
    }
    return `${n.kind === 'walk' ? 'walk' : n.rental ? `${n.rental.systemName || 'shared bike'}` : 'ride'} ${Math.round(n.dur / 60)}′`;
  };
  let next;
  if (now < +j.start) {
    const l = legs[0];
    next = { at: j.start, title: l.kind === 'transit' ? `Board the ${legLabel(l)}` : `Leave ${j.from.name}`,
      sub: l.kind === 'transit' ? names[0].from : `${l.kind === 'walk' ? 'walk' : 'ride'} ${Math.round(l.dur / 60)}′ to ${names[0].to || 'the station'}` };
  } else if (cur === -1) {
    next = { at: null, title: `Arrived at ${j.to.name}`, sub: '' };
  } else {
    const l = legs[cur];
    if (l.kind === 'transit' && now >= +l.dep) {
      next = { at: l.arr, title: `Get off at ${names[cur].to}`, sub: after(cur), getOff: cur };
    } else {
      const t = l.kind === 'transit' ? cur : transit.find(k => k > cur);
      next = t != null
        ? { at: legs[t].dep, title: `Board the ${legLabel(legs[t])}`, sub: [names[t].from, legs[t].live?.platform && `track ${legs[t].live.platform}`].filter(Boolean).join(' · ') }
        : { at: j.end, title: `Arrive at ${j.to.name}`, sub: `${legLabel(l)} ${Math.round((j.end - now) / MIN)}′` };
    }
  }
  // Changes still ahead: arriving with leg a, leaving with leg b.
  const changes = [];
  for (let i = 1; i < transit.length; i++) {
    const a = transit[i - 1];
    const b = transit[i];
    if (+legs[b].dep < now) continue;
    const walk = between(a, b);
    changes.push({ a, b, place: names[b].from, walk, spare: (legs[b].dep - legs[a].arr) / 1000 - walk, leg: legs[b] });
  }
  // A shared bike after the last ride: will there be one?
  const lastT = transit[transit.length - 1] ?? -1;
  const pickup = legs.findIndex((l, k) => k > lastT && l.rental);
  return { next, changes, pickup: pickup >= 0 && +legs[pickup].dep > now - 5 * MIN ? pickup : -1, cur };
}

const spareCls = s => (s < 120 ? 'bad' : s < 300 ? 'tight' : 'ok');
const mins = sec => `${Math.round(sec / 60)}′`;

function countdownHTML(at, now) {
  if (!at) return '<div class="r-count done">✓</div>';
  const m = Math.max(0, Math.round((at - now) / MIN));
  if (m >= 60) return `<div class="r-count"><b>${Math.floor(m / 60)}</b><span>h</span><b>${String(m % 60).padStart(2, '0')}</b></div>`;
  return `<div class="r-count"><b>${m < 1 ? 'now' : m}</b>${m < 1 ? '' : '<span>min</span>'}</div>`;
}

// The rest of the journey to scale, from now (or the start) to arrival.
function rideStripHTML(j, now) {
  const t0 = Math.max(now, +j.start);
  const total = Math.max(+j.end - t0, MIN);
  const segs = j.legs.filter(l => +l.arr > t0).map(l => {
    const a = Math.max(+l.dep, t0);
    const left = ((a - t0) / total) * 100;
    const width = Math.max(((l.arr - a) / total) * 100, 0.8);
    return `<span class="seg-b ${l.kind}${l.kmh ? ' fast' : ''}" style="left:${left.toFixed(2)}%;width:${width.toFixed(2)}%;${blockColors(l)}"></span>`;
  }).join('');
  const hours = [];
  for (let h = Math.ceil(t0 / (60 * MIN)) * 60 * MIN; h < +j.end; h += 60 * MIN) hours.push(`<i style="left:${((h - t0) / total * 100).toFixed(1)}%">${fmtTime(new Date(h))}</i>`);
  return `<div class="strip r-strip">${segs}</div><div class="r-strip-lbl"><i style="left:0">${now < +j.start ? fmtTime(j.start) : 'now'}</i>${hours.join('')}<i style="right:0">${fmtTime(j.end)}</i></div>`;
}

function changeCardHTML(ch, k, fb) {
  const scale = Math.max(SPARE_SCALE_MIN * 60, ch.walk + Math.max(ch.spare, 0));
  const walkW = (ch.walk / scale) * 100;
  const spareW = (Math.max(ch.spare, 0) / scale) * 100;
  const l = ch.leg;
  const delay = l.delay >= 1 ? ` <em class="dly">+${l.delay}</em>` : '';
  const spareTxt = l.cancelled ? 'cancelled' : ch.spare < 0 ? `${mins(-ch.spare)} short` : `${mins(ch.spare)} spare`;
  let plan;
  if (fb === undefined) plan = '<span class="muted">Looking for a plan B…</span>';
  else if (!fb) plan = '<span class="muted">No other way on from here tonight</span>';
  else {
    const first = fb.transit[0];
    const late = Math.round((fb.end - currentRide().end) / MIN);
    plan = `<button type="button" class="r-planb" data-planb="${k}">If missed: ${first ? `${esc(first.route.display_name)} ${fmtTime(first.dep)}` : esc(legLabel(fb.legs[0]))} → arrive ${fmtTime(fb.end)}
      <b class="${late >= 15 ? 'bad' : 'tight'}">${late >= 0 ? '+' : ''}${late}′</b></button>`;
  }
  return `<div class="r-card ${l.cancelled ? 'bad' : spareCls(ch.spare)}">
    <div class="r-card-head"><span><b>${fmtTime(ch.leg.dep)} ${esc(ch.place || '')}</b> → ${pill(l.route)}${delay}${l.live?.platform ? ` <span class="trk">${esc(l.live.platform)}</span>` : ''}</span><b class="r-spare">${spareTxt}</b></div>
    <div class="r-bar"><span class="walk" style="width:${walkW.toFixed(1)}%"></span><span class="spare" style="width:${spareW.toFixed(1)}%"></span></div>
    <div class="r-card-foot">${plan}</div></div>`;
}

function pickupCardHTML(j, k, info) {
  const l = j.legs[k];
  const r = l.rental;
  const bikes = info === undefined ? '' : !info ? '<b class="r-spare bad">no station found</b>'
    : `<b class="r-spare ${info.bikes > 2 ? 'ok' : info.bikes > 0 ? 'tight' : 'bad'}">${info.bikes} bike${info.bikes === 1 ? '' : 's'}</b>`;
  const rule = r.returnConstraint === 'ROUNDTRIP_STATION' ? 'bring it back here' : r.toStationName ? `park at ${esc(r.toStationName)}` : '';
  return `<div class="r-card"><div class="r-card-head"><span><b>${fmtTime(l.dep)} ${esc(r.fromStationName || legNames(j)[k].from || '')}</b> → ${esc(r.systemName || r.providerId)} ${mins(l.dur)}</span>${bikes}</div>
    ${rule ? `<div class="r-card-foot muted">${rule}</div>` : ''}</div>`;
}

// If a change ahead can't be made any more, you'll arrive with its plan B (when known).
function lostChange(m, ex) {
  const k = m.changes.findIndex(ch => ch.spare < 0 || ch.leg.cancelled);
  return k < 0 ? null : { k, ch: m.changes[k], fb: ex?.fallbacks[k] || null };
}

function renderRide(j) {
  const now = Date.now();
  const m = rideModel(j, now);
  const ex = rideState.extras && rideState.extrasKey === j.key ? rideState.extras : null;
  const curLeg = j.legs[m.cur];
  const names = legNames(j);
  const on = m.cur < 0 ? 'Arrived'
    : now < +j.start ? 'Before you leave'
    : curLeg.kind === 'transit' ? (now >= +curLeg.dep ? `On the ${legLabel(curLeg)}` : `At ${names[m.cur].from}`)
    : curLeg.kind === 'walk' ? 'Walking' : curLeg.rental ? `On a ${legLabel(curLeg)}` : 'Riding';
  const lost = lostChange(m, ex);
  const arrive = lost?.fb
    ? `<s>${fmtTime(j.end)}</s> <b class="dly">${fmtTime(lost.fb.end)}</b>`
    : `<b class="${j.delay >= 1 ? 'dly' : ''}">${fmtTime(j.end)}</b>`;
  const live = j.transit.some(l => l.live?.isLive);
  const cards = m.changes.map((ch, k) => changeCardHTML(ch, k, ex ? ex.fallbacks[k] : undefined)).join('')
    + (m.pickup >= 0 ? pickupCardHTML(j, m.pickup, ex ? ex.rentals[0] : undefined) : '');
  const notifyBtn = 'Notification' in window && Notification.permission === 'default'
    ? '<button type="button" class="r-notify" data-action="ride-notify">Allow alerts</button>' : '';
  $('#ride').innerHTML = `<div class="r-top"><span>${esc(on)} · ${fmtTime(new Date(now))} · arrive ${arrive}</span>
      ${live ? '<span class="r-live"><span class="live-dot"></span>live</span>' : '<span class="r-live muted">timetable</span>'}</div>
    <div class="r-next">${countdownHTML(m.next.at, now)}<div><b>${esc(m.next.title)}</b>${m.next.sub ? `<small>${esc(m.next.sub)}</small>` : ''}</div></div>
    ${rideStripHTML(j, now)}
    ${rideState.change ? `<div class="r-alert">${esc(rideState.change)}</div>` : ''}
    ${lost ? `<div class="r-alert bad"><b>${lost.ch.leg.cancelled ? `The ${esc(lost.ch.leg.route.display_name)} is cancelled` : `You'll miss the ${esc(lost.ch.leg.route.display_name)} at ${esc(lost.ch.place)}`}</b>
      ${lost.fb ? `<button type="button" class="r-take" data-planb="${lost.k}">Take plan B: arrive ${fmtTime(lost.fb.end)}</button>` : ' Looking for plan B…'}</div>`
      : j.cancelled ? '<div class="r-alert bad">A ride on this route is cancelled. Replan from here.</div>' : ''}
    <div class="r-cards">${cards || '<div class="muted r-empty">No more changes. Enjoy the ride.</div>'}</div>
    <div class="r-actions">${notifyBtn}<button type="button" data-action="ride-route">Route &amp; map</button><button type="button" data-action="ride-replan">Replan from here</button><button type="button" class="r-end" data-action="ride-end">End ride</button></div>`;
  maybeAlert(j, m, now, lost);
}

// The three facts (plus "get off now"); a buzz when one changes.
function maybeAlert(j, m, now, lost) {
  const ch = m.changes[0];
  const facts = {
    key: j.key,
    action: m.next.title,
    spare: ch ? Math.round(ch.spare / 60) : null,
    arrive: Math.round((lost?.fb?.end || j.end) / MIN),
    cancelled: j.cancelled,
    lost: lost ? lost.ch.leg.tripId : null,
  };
  const old = rideState.facts;
  rideState.facts = facts;
  if (m.next.getOff != null && m.next.at - now <= GET_OFF_WARN && !rideState.warned.has(`off${m.next.getOff}@${j.key}`)) {
    rideState.warned.add(`off${m.next.getOff}@${j.key}`);
    const planB = lost?.fb && lost.ch.a === m.next.getOff;
    notify(m.next.title.replace('Get off', 'Get off soon'), planB ? `Then plan B: arrive ${fmtTime(lost.fb.end)}` : m.next.sub ? `Then ${m.next.sub}` : '');
  }
  if (!old || old.key !== facts.key) return;
  const msgs = [];
  if (facts.lost && facts.lost !== old.lost) {
    msgs.push(`You'll miss the ${lost.ch.leg.route.display_name} at ${lost.ch.place}`);
    if (lost.fb) msgs.push(`Plan B: ${lost.fb.transit[0] ? `${lost.fb.transit[0].route.display_name} ${fmtTime(lost.fb.transit[0].dep)}, ` : ''}arrive ${fmtTime(lost.fb.end)}`);
  } else if (facts.cancelled && !old.cancelled) msgs.push('A ride on your route is cancelled');
  if (Math.abs(facts.arrive - old.arrive) >= 2) msgs.push(`Arrival now ${fmtTime(new Date(facts.arrive * MIN))} (${facts.arrive > old.arrive ? '+' : ''}${facts.arrive - old.arrive}′)`);
  if (!facts.lost && ch && old.spare != null && facts.spare !== old.spare && (Math.abs(facts.spare - old.spare) >= 3 || (facts.spare < 2) !== (old.spare < 2))) {
    msgs.push(`Spare at ${ch.place} ${old.spare}′ → ${facts.spare}′${facts.spare < 2 ? ': tight!' : ''}`);
  }
  if (facts.action !== old.action) msgs.unshift(`Next: ${facts.action}`);
  if (!msgs.length) return;
  rideState.change = `${fmtTime(new Date(now))} · ${msgs.join(' · ')}`;
  notify(msgs[0], msgs.slice(1).join(' · ') || m.next.sub || '');
  renderRideSoon();
}

let rideRenderTimer = 0;
const renderRideSoon = () => { clearTimeout(rideRenderTimer); rideRenderTimer = setTimeout(() => { const j = currentRide(); if (j && document.body.dataset.mode === 'ride') renderRide(j); }, 50); };

async function notify(title, body) {
  if (!('Notification' in window) || Notification.permission !== 'granted') return;
  const opts = { body, tag: 'overstap-ride', renotify: true, vibrate: [180, 80, 180], icon: 'icon.svg', badge: 'icon.svg' };
  try {
    const reg = await navigator.serviceWorker?.getRegistration();
    if (reg) await reg.showNotification(title, opts); else new Notification(title, opts);
  } catch { /* in-page banner only */ }
}

// Plan B per change and bike counts at the pickup; cached for RIDE_EXTRAS_EVERY.
async function loadRideExtras(j, force = false) {
  if (rideState.loading || (!force && rideState.extrasKey === j.key && Date.now() - rideState.extrasAt < RIDE_EXTRAS_EVERY)) return;
  const m = rideModel(j);
  const body = {
    mode: j.tab,
    to: ll(j.to.coords),
    bike: j.bike && j.bike !== 'here' ? j.bike : undefined,
    changes: m.changes.map(ch => {
      const prev = j.legs[ch.a];
      const stop = prev.stops[prev.stops.length - 1];
      return { from: stop?.id || ll(stop.coords), after: ch.leg.dep.toISOString(), avoidTrip: ch.leg.tripId };
    }),
    rentals: [],
  };
  if (m.pickup >= 0) {
    const raw = j.raw.legs[m.pickup];
    body.rentals = [{ lat: raw.from.lat, lng: raw.from.lon, provider: j.legs[m.pickup].rental.providerId }];
  }
  rideState.loading = true;
  try {
    const d = await api('/api/ride', {}, body);
    rideState.extras = { fallbacks: d.fallbacks.map(f => (f ? normJourney(f, null, j.tab === 'folding') : null)), rentals: d.rentals };
    rideState.extrasKey = j.key;
    rideState.extrasAt = Date.now();
  } catch { /* try again next tick */ }
  rideState.loading = false;
  if (document.body.dataset.mode === 'ride') renderRideSoon();
}

// Where a route leaves your own bike: at the station you ride it to, or at the destination
// when you ride it there. Taking the route moves it there, so the next search knows.
function moveBikeAlong(j) {
  if (!j.bike) return;
  const k = j.legs.map(l => l.kind === 'bike' && !l.rental).lastIndexOf(true);
  if (k < 0) return;
  const raw = j.raw.legs[k];
  setBike('parked', k === j.legs.length - 1 ? slim(j.to) : { name: legNames(j)[k].to || 'the station', address: null, coords: [raw.to.lat, raw.to.lon] });
}

function startRide(j) {
  if (!isSaved(j.key)) saveJourney(j);
  moveBikeAlong(j);
  state.rideId = j.key;
  state.savedId = null;
  Object.assign(rideState, { facts: null, change: null, extras: null, extrasKey: '' });
  if ('Notification' in window && Notification.permission === 'default') Notification.requestPermission().then(renderRideSoon);
  writeHash();
  render();
  const r = currentRide();
  if (r) { loadRideExtras(r, true); liveThenRender([r]); }
}

// Switch to plan B for change k: everything up to the stop you'd be stranded at, then plan B.
function takePlanB(k) {
  const j = currentRide();
  const ex = rideState.extras;
  const fb = ex?.fallbacks[k];
  if (!j || !fb) return;
  const ch = rideModel(j).changes[k];
  const raw = { legs: [...j.raw.legs.slice(0, ch.a + 1), ...fb.raw.legs], label: 'Plan B', notes: [`Plan B from ${ch.place}, made ${fmtTime(new Date())}.`] };
  const nj = normJourney(raw, null, j.tab === 'folding');
  if (!nj) return;
  Object.assign(nj, { key: `${journeyKey(nj, j.tab)}|b`, from: j.from, to: j.to, tab: j.tab, bike: j.bike });
  saveJourney(nj);
  unsaveJourney(j.key);
  startRide(nj);
  rideState.change = `${fmtTime(new Date())} · Switched to plan B: arrive ${fmtTime(nj.end)}`;
}

function endRide() {
  state.rideId = null;
  rideState.facts = null;
  writeHash();
  render();
}

setInterval(async () => {
  const j = currentRide();
  if (!j || document.body.dataset.mode !== 'ride') return;
  await refreshLive([j]);
  renderRide(j);
  loadRideExtras(j);
}, RIDE_REFRESH);

if ('serviceWorker' in navigator) navigator.serviceWorker.register('sw.js').catch(() => {});

// ---------- events ----------

document.addEventListener('click', e => {
  const t = e.target;
  const menu = $('#savedMenu');
  if (!menu.hidden && !t.closest('#savedMenu, #saved')) { menu.hidden = true; renderSavedBar(); }
  const savedOpen = t.closest('[data-saved]');
  if (savedOpen) { openSaved(savedOpen.dataset.saved); return; }
  const unsave = t.closest('[data-unsave]');
  if (unsave) { unsaveJourney(unsave.dataset.unsave); writeHash(); render(); return; }
  const tab = t.closest('[data-tab]');
  if (tab) { state.tab = tab.dataset.tab; state.savedId = null; plan(); return; }
  const more = t.closest('[data-more]');
  if (more) {
    if (state.paging[more.dataset.more] === 'error') state.paging[more.dataset.more] = 'idle';
    loadMore(more.dataset.more);
    return;
  }
  const show = t.closest('button[data-show]');
  if (show) { state.detailView = show.dataset.show; writeHash(); render(); return; }
  const variant = t.closest('[data-variant]');
  if (variant) { openDetail(variant.dataset.variant); return; }
  const route = t.closest('[data-key]');
  if (route) { openDetail(route.dataset.key); return; }
  const z = t.closest('[data-zoom]');
  if (z) { zoom(+z.dataset.zoom); return; }
  const trip = t.closest('[data-trip]');
  if (trip) {
    const saved = store.get('trips', [])[+trip.dataset.trip];
    if (saved) { setEndpoint('from', saved.from, { replan: false }); setEndpoint('to', saved.to); }
    return;
  }
  if (t.closest('#summary')) { state.editing = true; render(); return; }
  if (t.closest('#done')) { state.editing = false; render(); return; }
  if (t.closest('#pill')) { setHeader(true); return; }
  const action = t.closest('[data-action]')?.dataset.action;
  if (action === 'retry') plan();
  if (action === 'back') {
    if (state.savedId) state.savedId = null; else state.selected = null;
    writeHash();
    render();
  }
  if (action === 'saved-menu') { menu.hidden = !menu.hidden; renderSavedBar(); }
  if (action === 'ride-start') { const j = currentDetail(); if (j) startRide(j); return; }
  if (action === 'ride-end') { endRide(); return; }
  if (action === 'ride-route') { state.savedId = state.rideId; state.rideId = null; writeHash(); render(); return; }
  if (action === 'ride-notify') { Notification.requestPermission().then(renderRideSoon); return; }
  if (action === 'ride-replan') { endRide(); locate('from'); return; }
  const planb = t.closest('[data-planb]');
  if (planb) { takePlanB(+planb.dataset.planb); return; }
  if (action === 'toggle-all') { state.showAll = !state.showAll; render(); }
  if (action === 'toggle-save') {
    const j = currentDetail();
    if (!j) return;
    if (isSaved(j.key)) unsaveJourney(j.key); else saveJourney(j);
    writeHash();
    render();
  }
});

const schedEl = $('#schedule');
schedEl.addEventListener('pointerover', e => { if (e.pointerType === 'mouse') setHot(e.target.closest('[data-key]')?.dataset.key ?? null); });
schedEl.addEventListener('pointerleave', () => setHot(null));
schedEl.addEventListener('keydown', e => {
  const route = e.target.closest('.route[data-key]');
  if (route && (e.key === 'Enter' || e.key === ' ')) { e.preventDefault(); openDetail(route.dataset.key); }
});

// Scrolling forward (down or right) hides the search bar; scrolling back a bit shows it again.
let travel = 0;
schedEl.addEventListener('scroll', () => {
  const s = state.sched;
  const x = schedEl.scrollLeft;
  const y = schedEl.scrollTop;
  const ours = s.expect && x === s.expect.x && y === s.expect.y;
  const delta = (x - s.last.x) + (y - s.last.y);
  s.last = { x, y };
  s.expect = null;
  if (ours || !s.o) return;
  const p = s.o === 'h' ? x : y;
  s.anchor = s.t0 + ((p - s.E0) / s.ppm) * MIN;

  // Scrolling the timetable means you're done with the search fields for now.
  if (state.editing) { state.editing = false; document.body.classList.remove('editing'); }
  if ($('#top').contains(document.activeElement)) document.activeElement.blur();
  travel = Math.sign(delta) === Math.sign(travel) ? travel + delta : delta;
  if (travel > 40) setHeader(false);
  else if (travel < -90) setHeader(true);
}, { passive: true });
$('#top').addEventListener('focusin', () => setHeader(true));

let lastWheelZoom = 0;
schedEl.addEventListener('wheel', e => {
  const s = state.sched;
  if (!s.o) return;
  if (e.ctrlKey) {
    e.preventDefault();
    if (performance.now() - lastWheelZoom > 140) { lastWheelZoom = performance.now(); zoom(e.deltaY < 0 ? 1 : -1); }
    return;
  }
  // A plain mouse wheel scrolls the time axis when the lanes already fit vertically.
  if (s.o === 'h' && !e.shiftKey && Math.abs(e.deltaY) > Math.abs(e.deltaX) && schedEl.scrollHeight <= schedEl.clientHeight + 1) {
    e.preventDefault();
    schedEl.scrollLeft += e.deltaY;
  }
}, { passive: false });

// Only a width change can alter the layout (orientation, column widths); height changes come
// from the search bar sliding and are absorbed by CSS.
let resizeFrame = 0;
let lastWidth = 0;
new ResizeObserver(() => {
  cancelAnimationFrame(resizeFrame);
  resizeFrame = requestAnimationFrame(() => {
    const w = schedEl.clientWidth;
    if (document.body.dataset.stage !== 'schedule' || (w === lastWidth && state.sched.o)) return;
    lastWidth = w;
    renderSchedule();
  });
}).observe(schedEl);
narrowQuery.addEventListener('change', () => render());

const topEl = $('#top');
new ResizeObserver(() => document.documentElement.style.setProperty('--top-h', `${topEl.offsetHeight}px`)).observe(topEl);

// Move the "now" line along.
setInterval(() => { if (document.body.dataset.stage === 'schedule' && state.status === 'ok') renderSchedule(); }, 30000);
// Live delays and tracks: every minute, and straight away when you come back to the tab.
setInterval(liveTick, 60000);
document.addEventListener('visibilitychange', () => { if (!document.hidden) liveTick(); });

$('#swap').innerHTML = icon('swap');
$('#swap').addEventListener('click', () => {
  const { from, to } = state;
  setEndpoint('from', to, { replan: false });
  setEndpoint('to', from);
});
$('.locate').innerHTML = icon('locate');
$('.refresh').innerHTML = icon('refresh');
$('.locate').addEventListener('click', () => locate('from'));
$('.bike-ico').innerHTML = icon('bike');
{
  const b = store.get('bike', null);
  if (b && ['none', 'here', 'parked'].includes(b.mode)) setBike(b.mode === 'parked' && !b.at ? 'none' : b.mode, b.at || null);
}
$('#bike-mode').addEventListener('change', e => {
  setBike(e.target.value);
  state.savedId = null;
  if (state.bike === 'parked' && !state.bikeAt) { renderTabs(); $('#bikeAt').focus(); return; }
  plan();
});

const timeMode = $('#time-mode');
const timeInput = $('#time');
timeMode.addEventListener('change', () => {
  state.timeMode = timeMode.value;
  timeInput.hidden = state.timeMode === 'now';
  if (!timeInput.hidden && !timeInput.value) timeInput.value = defaultTimeValue();
  state.time = state.timeMode === 'now' ? '' : timeInput.value;
  state.savedId = null;
  plan();
});
timeInput.addEventListener('change', () => { state.time = timeInput.value; if (state.time) { state.savedId = null; plan(); } });

addEventListener('keydown', e => {
  if (e.key === 'Escape' && currentDetail() && !e.target.closest('input')) {
    if (state.savedId) state.savedId = null; else state.selected = null;
    writeHash();
    render();
  }
});

// ---------- boot ----------

function applyHash() {
  readHash();
  timeMode.value = state.timeMode;
  timeInput.hidden = state.timeMode === 'now';
  timeInput.value = state.time || '';
  for (const role of ['from', 'to']) { $('#' + role).value = state[role]?.name || ''; placeMarker(role); }
}

setupField('from');
setupField('to');
setupField('bikeAt');
pruneSaved();
applyHash();
// Our own hash writes use replaceState, so this only fires for pasted or bookmarked links.
addEventListener('hashchange', () => { applyHash(); plan(); });

(async () => {
  try {
    const cfg = await api('/api/config');
    state.config = cfg;
    try { timeFmt = new Intl.DateTimeFormat('en-GB', { hour: '2-digit', minute: '2-digit', timeZone: cfg.timezone }); } catch { /* keep local */ }
    const b = cfg.bounds;
    if (!state.from && !state.to && b) map.fitBounds([b.southwest, b.northeast]);
  } catch { /* defaults are fine */ }
  plan();
  if (state.savedId) liveThenRender([currentDetail()]);
  if (state.rideId) { const r = currentRide(); if (r) { liveThenRender([r]); loadRideExtras(r, true); } }
})();
