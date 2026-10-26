"""Regenerates docs/showcase/*.svg with `python scripts/showcase.py`: the animated README showcase (SMIL only, so they animate inside GitHub's <img> sandbox)."""
import os

OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "docs", "showcase")
os.makedirs(OUT, exist_ok=True)

FONT = "'Segoe UI Variable Text','Segoe UI',Inter,system-ui,-apple-system,'Helvetica Neue',Arial,sans-serif"
ISLAND = "#120d0b"
RAISED = "#1f1714"
BORDER = "#3a2c25"
TEXT = "#f8efe6"
MUTED = "#bba99c"
SUBTLE = "#8a786c"
ACCENT = "#eaa660"
SUCCESS = "#74d3a9"
CLAUDE = "#d97757"
EASE = "0.32 0.72 0 1"
LINEAR = "0 0 1 1"


def kt(*fractions):
    return ";".join(f"{value:.4f}".rstrip("0").rstrip(".") if value not in (0, 1) else str(int(value)) for value in fractions)


def fade(begin_in, end_in, begin_out, end_out, dur):
    """Opacity keyframes: hidden, fades in over [begin_in, end_in], holds, fades out over [begin_out, end_out]."""
    times = [0, begin_in / dur, end_in / dur, begin_out / dur, end_out / dur, 1]
    return f'<animate attributeName="opacity" values="0;0;1;1;0;0" keyTimes="{kt(*times)}" dur="{dur}s" repeatCount="indefinite"/>'


def hero():
    dur = 12
    w, h = 880, 400
    pill = dict(x=290, y=304, width=300, height=44, rx=22)
    card = dict(x=180, y=52, width=520, height=296, rx=30)
    times = kt(0, 2.0 / dur, 2.5 / dur, 6.0 / dur, 6.5 / dur, 1)
    splines = f"{LINEAR};{EASE};{LINEAR};{EASE};{LINEAR}"

    def morph(attribute):
        a, b = pill[attribute], card[attribute]
        return (
            f'<animate attributeName="{attribute}" values="{a};{a};{b};{b};{a};{a}" keyTimes="{times}" '
            f'calcMode="spline" keySplines="{splines}" dur="{dur}s" repeatCount="indefinite"/>'
        )

    island_rect = "".join(morph(attribute) for attribute in ("x", "y", "width", "height", "rx"))

    bars = []
    for index, (period, low, high) in enumerate([(0.9, 4, 14), (1.1, 6, 18), (0.7, 3, 12), (1.3, 5, 16), (0.8, 4, 11), (1.0, 6, 15)]):
        x = 552 + index * 5
        bars.append(
            f'<rect x="{x}" width="3" rx="1.5" fill="{ACCENT}">'
            f'<animate attributeName="height" values="{low};{high};{low + 2};{high - 3};{low}" dur="{period}s" repeatCount="indefinite"/>'
            f'<animate attributeName="y" values="{334 - low};{334 - high};{334 - low - 2};{334 - high + 3};{334 - low}" dur="{period}s" repeatCount="indefinite"/>'
            f"</rect>"
        )

    lyrics = [
        "We melted down the city lights",
        "and poured them into something sweet",
        "you said the night was ours to keep",
        "so I kept it in my coat pocket",
        "warm until the morning came",
    ]
    lyric_switch = kt(0, 4.0 / dur, 4.3 / dur, 1)
    lyric_lines = []
    for index, line in enumerate(lyrics):
        if index == 1:
            fill = f'<animate attributeName="fill" values="{TEXT};{TEXT};{MUTED};{MUTED}" keyTimes="{lyric_switch}" dur="{dur}s" repeatCount="indefinite"/>'
            weight, base = 600, TEXT
        elif index == 2:
            fill = f'<animate attributeName="fill" values="{MUTED};{MUTED};{TEXT};{TEXT}" keyTimes="{lyric_switch}" dur="{dur}s" repeatCount="indefinite"/>'
            weight, base = 600, MUTED
        else:
            fill, weight, base = "", 400, MUTED
        lyric_lines.append(
            f'<text x="440" y="{298 + index * 21}" text-anchor="middle" font-size="13" font-weight="{weight}" fill="{base}">{line}{fill}</text>'
        )

    svg = f"""<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}" viewBox="0 0 {w} {h}" font-family="{FONT}">
  <title>ChocolateBar: the island morphs from a pill into a media card, then cycles through Claude Code and battery activities</title>
  <defs>
    <linearGradient id="desk" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#2b2420"/>
      <stop offset="1" stop-color="#16110f"/>
    </linearGradient>
    <radialGradient id="blobA" cx="0.5" cy="0.5" r="0.5">
      <stop offset="0" stop-color="#eaa660" stop-opacity="0.22"/>
      <stop offset="1" stop-color="#eaa660" stop-opacity="0"/>
    </radialGradient>
    <radialGradient id="blobB" cx="0.5" cy="0.5" r="0.5">
      <stop offset="0" stop-color="#93acff" stop-opacity="0.14"/>
      <stop offset="1" stop-color="#93acff" stop-opacity="0"/>
    </radialGradient>
    <radialGradient id="sheen" cx="0.5" cy="0" r="0.9">
      <stop offset="0" stop-color="#eaa660" stop-opacity="0.09"/>
      <stop offset="1" stop-color="#eaa660" stop-opacity="0"/>
    </radialGradient>
    <linearGradient id="art" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#f4bd81"/>
      <stop offset="0.5" stop-color="#b8643a"/>
      <stop offset="1" stop-color="#3b1c10"/>
    </linearGradient>
    <linearGradient id="lyricFade" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#fff" stop-opacity="0"/>
      <stop offset="0.3" stop-color="#fff" stop-opacity="1"/>
      <stop offset="0.7" stop-color="#fff" stop-opacity="1"/>
      <stop offset="1" stop-color="#fff" stop-opacity="0"/>
    </linearGradient>
    <mask id="lyricMask"><rect x="180" y="276" width="520" height="62" fill="url(#lyricFade)"/></mask>
    <filter id="shadow" x="-20%" y="-20%" width="140%" height="160%">
      <feDropShadow dx="0" dy="10" stdDeviation="14" flood-color="#000" flood-opacity="0.55"/>
    </filter>
  </defs>

  <rect width="{w}" height="{h}" fill="url(#desk)"/>
  <circle cx="160" cy="90" r="220" fill="url(#blobA)"/>
  <circle cx="760" cy="300" r="240" fill="url(#blobB)"/>

  <rect x="0" y="364" width="{w}" height="36" fill="#0f2b2b" opacity="0.92"/>
  <g opacity="0.85">
    <rect x="374" y="372" width="20" height="20" rx="5" fill="#3aa0e0"/>
    <rect x="404" y="372" width="20" height="20" rx="5" fill="#6d5ae6"/>
    <rect x="434" y="372" width="20" height="20" rx="5" fill="#f0b429"/>
    <rect x="464" y="372" width="20" height="20" rx="5" fill="#e8562a"/>
    <rect x="494" y="372" width="20" height="20" rx="10" fill="#1ed760"/>
  </g>

  <g filter="url(#shadow)">
    <rect x="{pill['x']}" y="{pill['y']}" width="{pill['width']}" height="{pill['height']}" rx="{pill['rx']}" fill="{ISLAND}">{island_rect}</rect>
  </g>
  <rect x="{pill['x']}" y="{pill['y']}" width="{pill['width']}" height="{pill['height']}" rx="{pill['rx']}" fill="url(#sheen)" stroke="#ffecdc" stroke-opacity="0.08">{island_rect}</rect>

  <g id="media-pill">
    <animate attributeName="opacity" values="1;1;0;0;1;1" keyTimes="{kt(0, 1.9 / dur, 2.0 / dur, 10.6 / dur, 10.8 / dur, 1)}" dur="{dur}s" repeatCount="indefinite"/>
    <rect x="296" y="310" width="32" height="32" rx="9" fill="url(#art)"/>
    <text x="338" y="324" font-size="13" font-weight="600" fill="{TEXT}">Midnight Cocoa</text>
    <text x="338" y="339" font-size="11" fill="{MUTED}">The Wrappers</text>
    {"".join(bars)}
  </g>

  <g id="media-card" opacity="0">
    {fade(2.6, 2.9, 5.8, 6.0, dur)}
    <path d="M206 72v-9l8-2v9" fill="none" stroke="{ACCENT}" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>
    <circle cx="204" cy="72" r="2.4" fill="{ACCENT}"/><circle cx="212" cy="70" r="2.4" fill="{ACCENT}"/>
    <text x="224" y="75" font-size="11" font-weight="600" letter-spacing="0.9" fill="{SUBTLE}">SPOTIFY</text>
    <path d="M587 66l-4 4 4 4M651 66l4 4-4 4M672 67l4 4 4-4" fill="none" stroke="{MUTED}" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>
    <text x="619" y="74" text-anchor="middle" font-size="11" fill="{SUBTLE}">1 / 3</text>

    <rect x="212" y="100" width="76" height="76" rx="13" fill="url(#art)"/>
    <circle cx="250" cy="138" r="18" fill="none" stroke="#fff3e4" stroke-opacity="0.35" stroke-width="2"/>
    <text x="304" y="127" font-size="18" font-weight="600" fill="{TEXT}">Midnight Cocoa</text>
    <text x="304" y="150" font-size="13" fill="{MUTED}">The Wrappers</text>
    <text x="304" y="169" font-size="12" fill="{SUBTLE}">Sweet Static</text>

    <text x="212" y="204" font-size="11" fill="{MUTED}">1:06</text>
    <rect x="246" y="198" width="390" height="4" rx="2" fill="{BORDER}"/>
    <rect x="246" y="198" width="120" height="4" rx="2" fill="{TEXT}">
      <animate attributeName="width" values="118;118;178;178" keyTimes="{kt(0, 2.5 / dur, 6.0 / dur, 1)}" dur="{dur}s" repeatCount="indefinite"/>
    </rect>
    <text x="668" y="204" text-anchor="end" font-size="11" fill="{MUTED}">3:31</text>

    <g fill="none" stroke="{ACCENT}" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round">
      <path d="M332 234h4l10 12h4M332 246h4l10-12h4M347 231l3 3-3 3M347 243l3 3-3 3"/>
    </g>
    <circle cx="341" cy="254" r="1.6" fill="{ACCENT}"/>
    <path d="M384 232v16M397 232l-11 8 11 8z" fill="{TEXT}" stroke="{TEXT}" stroke-width="2" stroke-linejoin="round"/>
    <circle cx="440" cy="240" r="23" fill="{TEXT}"/>
    <rect x="432" y="231" width="5" height="18" rx="1.5" fill="{ISLAND}"/>
    <rect x="443" y="231" width="5" height="18" rx="1.5" fill="{ISLAND}"/>
    <path d="M496 232v16M483 232l11 8-11 8z" fill="{TEXT}" stroke="{TEXT}" stroke-width="2" stroke-linejoin="round"/>
    <path d="M532 236h14l-3-3M548 244h-14l3 3" fill="none" stroke="{SUBTLE}" stroke-width="1.7" stroke-linecap="round" stroke-linejoin="round"/>

    <g mask="url(#lyricMask)">
      <g>
        <animateTransform attributeName="transform" type="translate" values="0 0;0 0;0 -21;0 -21" keyTimes="{lyric_switch}" calcMode="spline" keySplines="{LINEAR};{EASE};{LINEAR}" dur="{dur}s" repeatCount="indefinite"/>
        {"".join(lyric_lines)}
      </g>
    </g>
  </g>

  <g id="claude-pill" opacity="0">
    {fade(6.6, 6.8, 8.4, 8.6, dur)}
    <circle cx="312" cy="326" r="4" fill="{CLAUDE}" opacity="0.6">
      <animate attributeName="r" values="4;10" dur="1.2s" repeatCount="indefinite"/>
      <animate attributeName="opacity" values="0.6;0" dur="1.2s" repeatCount="indefinite"/>
    </circle>
    <circle cx="312" cy="326" r="4" fill="{CLAUDE}"/>
    <text x="326" y="331" font-size="13" font-weight="600" fill="{TEXT}">ChocolateBar</text>
    <text x="414" y="331" font-size="13" fill="{MUTED}">Thinking…</text>
    <rect x="520" y="315" width="56" height="22" rx="11" fill="{RAISED}"/>
    <text x="548" y="330" text-anchor="middle" font-size="11" fill="{MUTED}">opus</text>
  </g>

  <g id="battery-pill" opacity="0">
    {fade(8.6, 8.8, 10.4, 10.6, dur)}
    <rect x="308" y="319" width="30" height="15" rx="4" fill="none" stroke="{SUCCESS}" stroke-opacity="0.75" stroke-width="2"/>
    <rect x="339.5" y="323.5" width="3" height="6" rx="1" fill="{SUCCESS}" fill-opacity="0.75"/>
    <rect x="311" y="322" width="21" height="9" rx="2" fill="{SUCCESS}">
      <animate attributeName="opacity" values="1;0.65;1" dur="1.8s" repeatCount="indefinite"/>
    </rect>
    <text x="352" y="331" font-size="14" font-weight="600" fill="{TEXT}">84%</text>
    <text x="574" y="330" text-anchor="end" font-size="11" fill="{MUTED}">Charging</text>
  </g>
</svg>
"""
    return svg


def positions():
    hold, move = 0.8, 0.45
    stops = [("bottom", "center"), ("bottom", "left"), ("middle", "left"), ("top", "left"), ("top", "center"),
             ("top", "right"), ("middle", "right"), ("bottom", "right"), ("middle", "center"), ("bottom", "center")]
    segment = hold + move
    dur = round(segment * (len(stops) - 1), 2)

    screen = dict(x=300, y=30, w=400, h=226)
    taskbar = 16
    pw, ph, gap = 92, 18, 8
    work_bottom = screen["y"] + screen["h"] - taskbar
    xs = {"left": screen["x"] + gap, "center": screen["x"] + (screen["w"] - pw) / 2, "right": screen["x"] + screen["w"] - gap - pw}
    ys = {"top": screen["y"] + gap, "middle": screen["y"] + (screen["h"] - taskbar - ph) / 2, "bottom": work_bottom - gap - ph}

    cell_w, cell_h, cell_gap, gx, gy = 64, 40, 6, 40, 70
    cols = {"left": 0, "center": 1, "right": 2}
    rows = {"top": 0, "middle": 1, "bottom": 2}
    bar_w, bar_h = 30, 7

    def marker(vertical, horizontal):
        cx = gx + cols[horizontal] * (cell_w + cell_gap)
        cy = gy + rows[vertical] * (cell_h + cell_gap)
        x = {"left": cx + 6, "center": cx + (cell_w - bar_w) / 2, "right": cx + cell_w - 6 - bar_w}[horizontal]
        y = {"top": cy + 6, "middle": cy + (cell_h - bar_h) / 2, "bottom": cy + cell_h - 6 - bar_h}[vertical]
        return x, y

    times, pill_x, pill_y, mark_x, mark_y = [], [], [], [], []
    for index, (vertical, horizontal) in enumerate(stops):
        start = index * segment
        for t in ([start] if index == len(stops) - 1 else [start, start + hold]):
            times.append(t / dur)
            pill_x.append(xs[horizontal]); pill_y.append(ys[vertical])
            mx, my = marker(vertical, horizontal)
            mark_x.append(mx); mark_y.append(my)
    splines = ";".join(LINEAR if i % 2 == 0 else EASE for i in range(len(times) - 1))

    def anim(attribute, values):
        joined = ";".join(f"{value:g}" for value in values)
        return (f'<animate attributeName="{attribute}" values="{joined}" keyTimes="{kt(*times)}" calcMode="spline" '
                f'keySplines="{splines}" dur="{dur}s" repeatCount="indefinite"/>')

    cells = []
    for vertical in rows:
        for horizontal in cols:
            cx = gx + cols[horizontal] * (cell_w + cell_gap)
            cy = gy + rows[vertical] * (cell_h + cell_gap)
            mx, my = marker(vertical, horizontal)
            cells.append(f'<rect x="{cx}" y="{cy}" width="{cell_w}" height="{cell_h}" rx="8" fill="{RAISED}" fill-opacity="0.55"/>')
            cells.append(f'<rect x="{mx:g}" y="{my:g}" width="{bar_w}" height="{bar_h}" rx="3.5" fill="{BORDER}"/>')

    mx0, my0 = mark_x[0], mark_y[0]
    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="740" height="290" viewBox="0 0 740 290" font-family="{FONT}">
  <title>Position picker: the island hops between nine anchors on the screen</title>
  <defs>
    <linearGradient id="bg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#1b1411"/>
      <stop offset="1" stop-color="#120d0b"/>
    </linearGradient>
    <linearGradient id="wall" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#3a2f28"/>
      <stop offset="1" stop-color="#1d1714"/>
    </linearGradient>
    <filter id="glow" x="-50%" y="-100%" width="200%" height="300%">
      <feDropShadow dx="0" dy="0" stdDeviation="4" flood-color="{ACCENT}" flood-opacity="0.6"/>
    </filter>
    <filter id="lift" x="-30%" y="-80%" width="160%" height="300%">
      <feDropShadow dx="0" dy="4" stdDeviation="5" flood-color="#000" flood-opacity="0.6"/>
    </filter>
  </defs>
  <rect width="740" height="290" rx="20" fill="url(#bg)"/>
  <text x="40" y="46" font-size="15" font-weight="600" fill="{TEXT}">Position</text>
  <text x="40" y="250" font-size="12" fill="{SUBTLE}">Nine anchors plus nudge offsets</text>
  {"".join(cells)}
  <rect x="{mx0:g}" y="{my0:g}" width="{bar_w}" height="{bar_h}" rx="3.5" fill="{ACCENT}" filter="url(#glow)">{anim("x", mark_x)}{anim("y", mark_y)}</rect>

  <rect x="{screen['x']}" y="{screen['y']}" width="{screen['w']}" height="{screen['h']}" rx="12" fill="url(#wall)" stroke="{BORDER}"/>
  <rect x="{screen['x']}" y="{work_bottom}" width="{screen['w']}" height="{taskbar}" fill="#0f2b2b" opacity="0.9"/>
  <g opacity="0.8">
    <rect x="474" y="{work_bottom + 4}" width="8" height="8" rx="2" fill="#3aa0e0"/>
    <rect x="488" y="{work_bottom + 4}" width="8" height="8" rx="2" fill="#f0b429"/>
    <rect x="502" y="{work_bottom + 4}" width="8" height="8" rx="2" fill="#e8562a"/>
    <rect x="516" y="{work_bottom + 4}" width="8" height="8" rx="4" fill="#1ed760"/>
  </g>
  <g filter="url(#lift)">
    <rect x="{pill_x[0]:g}" y="{pill_y[0]:g}" width="{pw}" height="{ph}" rx="9" fill="{ISLAND}" stroke="#ffecdc" stroke-opacity="0.12">{anim("x", pill_x)}{anim("y", pill_y)}</rect>
    <circle cx="{pill_x[0] + 10:g}" cy="{pill_y[0] + 9:g}" r="2.5" fill="{ACCENT}">{anim("cx", [x + 10 for x in pill_x])}{anim("cy", [y + 9 for y in pill_y])}</circle>
    <rect x="{pill_x[0] + 18:g}" y="{pill_y[0] + 7:g}" width="40" height="4" rx="2" fill="{MUTED}">{anim("x", [x + 18 for x in pill_x])}{anim("y", [y + 7 for y in pill_y])}</rect>
    <rect x="{pill_x[0] + 66:g}" y="{pill_y[0] + 7:g}" width="16" height="4" rx="2" fill="{TEXT}">{anim("x", [x + 66 for x in pill_x])}{anim("y", [y + 7 for y in pill_y])}</rect>
  </g>
</svg>
"""


def hub():
    dur = 10
    info, raspberry, berry = "#93acff", "#ff7a85", "#c7a1ff"

    def at(*seconds):
        return kt(*(second / dur for second in seconds))

    def loop(attribute, values, times, extra=""):
        return f'<animate attributeName="{attribute}" values="{values}" keyTimes="{times}" dur="{dur}s" repeatCount="indefinite"{extra}/>'

    def avatar(x, y, letters, hue):
        return (f'<rect x="{x}" y="{y}" width="28" height="28" rx="8" fill="hsl({hue} 45% 26%)" stroke="{ISLAND}" stroke-width="2"/>'
                f'<text x="{x + 14}" y="{y + 18}" text-anchor="middle" font-size="10" font-weight="700" fill="hsl({hue} 80% 82%)">{letters}</text>')

    def glyph(x, y, area):
        frame = f'<rect x="{x}" y="{y}" width="14" height="10" rx="2" fill="none" stroke="{ACCENT}" stroke-opacity="0.5"/>'
        ax, ay, aw, ah = area
        return frame + f'<rect x="{x + ax}" y="{y + ay}" width="{aw}" height="{ah}" rx="1" fill="{ACCENT}"/>'

    def window(name, color, floating, snapped, appear):
        fx, fy, fw, fh = floating
        sx, sy, sw, sh = snapped
        times = at(0, appear, appear + 0.35, appear + 0.75, 9.3, 9.7, 10)
        splines = f' calcMode="spline" keySplines="{LINEAR};{LINEAR};{EASE};{LINEAR};{LINEAR};{LINEAR}" additive="sum"'

        def transform(kind, a, b):
            values = f"{a};{a};{a};{b};{b};{b};{a}"
            return (f'<animateTransform attributeName="transform" type="{kind}" values="{values}" keyTimes="{times}" '
                    f'dur="{dur}s" repeatCount="indefinite"{splines}/>')

        floating_scale = f"{fw / 180:.4f} {fh / 214:.4f}"
        snapped_scale = f"{sw / 180:.4f} {sh / 214:.4f}"
        return (
            f'<g opacity="0">{loop("opacity", "0;0;1;1;1;0;0", times)}'
            f'<g>{transform("translate", f"{fx} {fy}", f"{sx} {sy}")}'
            f'<g>{transform("scale", floating_scale, snapped_scale)}'
            f'<rect x="1" y="1" width="178" height="212" rx="6" fill="#f3ede8" stroke="#000" stroke-opacity="0.25"/>'
            f'<rect x="1" y="1" width="178" height="22" rx="6" fill="{color}"/><rect x="1" y="14" width="178" height="9" fill="{color}"/>'
            f'<text x="10" y="16" font-size="10" font-weight="600" fill="#fff">{name}</text>'
            f'<rect x="14" y="38" width="110" height="7" rx="3.5" fill="#c9bfb7"/><rect x="14" y="54" width="150" height="5" rx="2.5" fill="#ddd4cd"/>'
            f'<rect x="14" y="66" width="140" height="5" rx="2.5" fill="#ddd4cd"/><rect x="14" y="78" width="120" height="5" rx="2.5" fill="#ddd4cd"/>'
            f'<rect x="14" y="98" width="152" height="60" rx="5" fill="#e6ded7"/>'
            f'</g></g></g>'
        )

    shortcuts_opacity = loop("opacity", "1;1;0;0;1", at(0, 4.5, 4.8, 9.7, 10))
    clipboard_opacity = loop("opacity", "0;0;1;1;0;0", at(0, 4.8, 5.1, 9.4, 9.7, 10))
    tab_x = loop("x", "56;56;156;156;56", at(0, 4.5, 4.85, 9.6, 10), f' calcMode="spline" keySplines="{LINEAR};{EASE};{LINEAR};{EASE}"')
    pressed = loop("fill-opacity", "0;0;0.18;0.18;0;0", at(0, 0.9, 1.0, 1.3, 1.6, 10))
    spinner = loop("opacity", "0;0;1;1;0;0", at(0, 1.0, 1.1, 2.7, 2.8, 10))
    push_down = (f'<animateTransform attributeName="transform" type="translate" values="0 0;0 0;0 58;0 58;0 0" '
                 f'keyTimes="{at(0, 6.0, 6.4, 9.7, 10)}" calcMode="spline" keySplines="{LINEAR};{EASE};{LINEAR};{LINEAR}" dur="{dur}s" repeatCount="indefinite"/>')
    new_entry = loop("opacity", "0;0;1;1;0;0", at(0, 6.15, 6.5, 9.4, 9.7, 10))
    copied = loop("opacity", "0;0;1;1;0;0", at(0, 7.6, 7.75, 8.9, 9.1, 10))

    def clip_row(y, body):
        return f'<rect x="52" y="{y}" width="396" height="50" rx="14" fill="#ffecdc" fill-opacity="0.045" stroke="#ffecdc" stroke-opacity="0.07"/>{body}'

    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="900" height="400" viewBox="0 0 900 400" font-family="{FONT}">
  <title>The hub: one tap on a shortcut opens Chrome and Word side by side, and the clipboard collects everything you copy</title>
  <defs>
    <linearGradient id="hubBg" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#1b1411"/>
      <stop offset="1" stop-color="#110c0a"/>
    </linearGradient>
    <linearGradient id="hubWall" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#3b4a5c"/>
      <stop offset="1" stop-color="#2a2226"/>
    </linearGradient>
    <linearGradient id="thumb" x1="0" y1="0" x2="1" y2="1">
      <stop offset="0" stop-color="#93acff"/>
      <stop offset="1" stop-color="#c7a1ff"/>
    </linearGradient>
    <clipPath id="hubCard"><rect x="30" y="84" width="440" height="276" rx="20"/></clipPath>
    <filter id="hubShadow" x="-10%" y="-10%" width="120%" height="130%">
      <feDropShadow dx="0" dy="10" stdDeviation="14" flood-color="#000" flood-opacity="0.5"/>
    </filter>
  </defs>
  <rect width="900" height="400" rx="20" fill="url(#hubBg)"/>

  <rect x="30" y="30" width="440" height="340" rx="28" fill="{ISLAND}" stroke="#ffecdc" stroke-opacity="0.08" filter="url(#hubShadow)"/>
  <rect x="52" y="44" width="262" height="32" rx="16" fill="{RAISED}"/>
  <rect x="56" y="48" width="98" height="24" rx="12" fill="{TEXT}">{tab_x}</rect>
  <g font-size="12" font-weight="600" text-anchor="middle">
    <text x="105" y="64" fill="{ISLAND}">Shortcuts{loop("fill", f"{ISLAND};{ISLAND};{MUTED};{MUTED};{ISLAND}", at(0, 4.5, 4.7, 9.7, 10))}</text>
    <text x="205" y="64" fill="{MUTED}">Clipboard{loop("fill", f"{MUTED};{MUTED};{ISLAND};{ISLAND};{MUTED}", at(0, 4.6, 4.8, 9.6, 10))}</text>
    <text x="283" y="64" fill="{MUTED}">Clock</text>
  </g>
  <path d="M422 56l5 5 5-5" fill="none" stroke="{MUTED}" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round"/>

  <g clip-path="url(#hubCard)">{shortcuts_opacity}
    <rect x="52" y="96" width="194" height="100" rx="16" fill="#ffecdc" fill-opacity="0.045" stroke="#ffecdc" stroke-opacity="0.07"/>
    <rect x="52" y="96" width="194" height="100" rx="16" fill="{ACCENT}" fill-opacity="0" stroke="{ACCENT}" stroke-opacity="0.6">{pressed}</rect>
    {avatar(66, 110, "GC", 200)}{avatar(88, 110, "W", 230)}
    <g opacity="0">{spinner}
      <circle cx="226" cy="124" r="7" fill="none" stroke="{ACCENT}" stroke-width="2" stroke-dasharray="30 14" stroke-linecap="round">
        <animateTransform attributeName="transform" type="rotate" from="0 226 124" to="360 226 124" dur="0.9s" repeatCount="indefinite"/>
      </circle>
    </g>
    <text x="66" y="166" font-size="14" font-weight="600" fill="{TEXT}">Work</text>
    <text x="66" y="184" font-size="11" fill="{MUTED}">2 apps</text>
    {glyph(110, 176, (1.5, 1.5, 5, 7))}{glyph(128, 176, (7.5, 1.5, 5, 7))}

    <rect x="254" y="96" width="194" height="100" rx="16" fill="#ffecdc" fill-opacity="0.045" stroke="#ffecdc" stroke-opacity="0.07"/>
    {avatar(268, 110, "D", 265)}{avatar(290, 110, "MD", 150)}
    <text x="268" y="166" font-size="14" font-weight="600" fill="{TEXT}">Gaming</text>
    <text x="268" y="184" font-size="11" fill="{MUTED}">2 apps</text>
    {glyph(312, 176, (1.5, 1.5, 11, 7))}

    <rect x="52" y="204" width="194" height="70" rx="16" fill="none" stroke="{BORDER}" stroke-dasharray="4 4"/>
    <text x="149" y="244" text-anchor="middle" font-size="12" fill="{MUTED}">+ New or edit</text>
  </g>

  <g opacity="0" clip-path="url(#hubCard)">{clipboard_opacity}
    <g opacity="0">{new_entry}
      {clip_row(96, f'<text x="66" y="117" font-size="13" fill="{TEXT}">Meeting notes: ship v0.2 on Friday,</text><text x="66" y="134" font-size="13" fill="{TEXT}">then update the README showcase</text><text x="434" y="112" text-anchor="end" font-size="10" fill="{SUBTLE}">now</text>')}
    </g>
    <g>{push_down}
      {clip_row(96, f'<rect x="62" y="104" width="64" height="34" rx="7" fill="url(#thumb)"/><text x="138" y="125" font-size="12" fill="{MUTED}">Image · 1920×1080</text><text x="434" y="112" text-anchor="end" font-size="10" fill="{SUBTLE}">2m</text><g opacity="0">{copied}<rect x="356" y="102" width="70" height="18" rx="9" fill="{SUCCESS}" fill-opacity="0.15"/><text x="391" y="115" text-anchor="middle" font-size="10" font-weight="600" fill="{SUCCESS}">Copied</text></g>')}
      {clip_row(154, f'<rect x="62" y="162" width="34" height="34" rx="8" fill="{RAISED}"/><path d="M73 170h8l5 5v13h-13z" fill="none" stroke="{ACCENT}" stroke-width="1.6" stroke-linejoin="round"/><text x="108" y="176" font-size="13" font-weight="600" fill="{TEXT}">report.docx</text><text x="108" y="191" font-size="11" fill="{MUTED}">C:\\Users\\you\\Documents</text><text x="434" y="170" text-anchor="end" font-size="10" fill="{SUBTLE}">9m</text>')}
      {clip_row(212, f'<text x="66" y="241" font-size="13" fill="{TEXT}">https://github.com/AbdulghaniKM/ChocolateBar</text><text x="434" y="228" text-anchor="end" font-size="10" fill="{SUBTLE}">32m</text>')}
      {clip_row(270, f'<text x="66" y="299" font-size="13" fill="{TEXT}">npm run build &amp;&amp; npm run preview</text><text x="434" y="286" text-anchor="end" font-size="10" fill="{SUBTLE}">1h</text>')}
    </g>
  </g>

  <rect x="500" y="40" width="370" height="244" rx="12" fill="url(#hubWall)" stroke="{BORDER}"/>
  <rect x="500" y="268" width="370" height="16" fill="#0f2b2b"/>
  <g opacity="0.85">
    <rect x="660" y="272" width="8" height="8" rx="2" fill="#3aa0e0"/><rect x="674" y="272" width="8" height="8" rx="4" fill="#4285f4"/>
    <rect x="688" y="272" width="8" height="8" rx="2" fill="#2b579a"/><rect x="702" y="272" width="8" height="8" rx="2" fill="#5865f2"/>
  </g>
  {window("Google Chrome", "#4285f4", (585, 90, 120, 140), (500, 40, 185, 228), 1.3)}
  {window("Word", "#2b579a", (665, 100, 120, 140), (685, 40, 185, 228), 2.1)}
  <text x="685" y="320" text-anchor="middle" font-size="12" fill="{SUBTLE}">One tap opens every app and snaps each window into place</text>
</svg>
"""


def palette():
    swatches = [("Island", ISLAND), ("Raised", RAISED), ("Cream", TEXT), ("Caramel", ACCENT),
                ("Mint", SUCCESS), ("Raspberry", "#ff7a85"), ("Blueberry", "#93acff"), ("Berry", "#c7a1ff")]
    parts = []
    for index, (name, color) in enumerate(swatches):
        x = 24 + index * 88
        delay = index * 0.12
        parts.append(
            f'<g opacity="0"><animate attributeName="opacity" from="0" to="1" begin="{delay:.2f}s" dur="0.5s" fill="freeze"/>'
            f'<rect x="{x}" y="20" width="76" height="56" rx="14" fill="{color}" stroke="#ffecdc" stroke-opacity="0.12"/>'
            f'<text x="{x}" y="96" font-size="12" font-weight="600" fill="{TEXT}">{name}</text>'
            f'<text x="{x}" y="112" font-size="11" fill="{SUBTLE}">{color}</text></g>'
        )
    return f"""<svg xmlns="http://www.w3.org/2000/svg" width="740" height="132" viewBox="0 0 740 132" font-family="{FONT}">
  <title>ChocolateBar palette</title>
  <rect width="740" height="132" rx="20" fill="#140e0b"/>
  {"".join(parts)}
</svg>
"""


for name, content in (("hero.svg", hero()), ("positions.svg", positions()), ("palette.svg", palette()), ("hub.svg", hub())):
    with open(os.path.join(OUT, name), "w", encoding="utf-8", newline="\n") as file:
        file.write(content)
    print(name, len(content))
