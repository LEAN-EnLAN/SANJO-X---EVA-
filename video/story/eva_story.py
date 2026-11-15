"""Eva by WearMeds: a story-first explainer for non-technical viewers, in English and Spanish.
Set EVA_LANG=en|es. Pictograms and diagrams are drawn in code; narration comes from narr/<lang>/<id>.wav."""
from manim import *
import json, os, numpy as np

HERE = os.path.dirname(os.path.abspath(__file__))
LANG = os.environ.get("EVA_LANG", "en")
SCRIPT = {l["id"]: l for l in json.load(open(os.path.join(HERE, "script.json")))}
DUR = json.load(open(os.path.join(HERE, "durations.json")))[LANG]
SFX = next(p for p in (os.path.join(HERE, "..", "sfx"), os.path.join(HERE, "..", "manim", "sfx")) if os.path.isdir(p))
config.background_color = "#000000"
CORAL = "#FF6A3D"; CREAM = "#F4F3F0"; CYAN = "#7FD4FF"; AMBER = "#FFB347"; MUTED = "#8A8986"; RED = "#FF2A1F"; YEL = "#FFD84A"; GREEN = "#5BE38A"; SKIN = "#E9B38F"; SUIT = "#E7E5E0"
FONT = "Inter"
L = {
 "wristband": {"en": "Wristband", "es": "Brazalete"}, "harness": {"en": "Harness", "es": "Arnés"},
 "inner": {"en": "Inner part: sensors", "es": "Parte de adentro: sensores"}, "outer": {"en": "Outer part: screen + battery", "es": "Parte de afuera: pantalla + batería"},
 "magnets": {"en": "Magnets", "es": "Imanes"}, "suit": {"en": "Suit", "es": "Traje"}, "skin": {"en": "Skin", "es": "Piel"},
 "own": {"en": "own small battery", "es": "batería chica propia"},
 "go": {"en": "Go help Ana", "es": "Andá a ayudar a Ana"}, "s1": {"en": "Hands here", "es": "Manos acá"}, "s2": {"en": "Push, follow the beat", "es": "Presioná al ritmo"},
 "s3": {"en": "Stand back", "es": "Alejate"}, "shelter": {"en": "SHELTER", "es": "REFUGIO"}, "toshelter": {"en": "Go to the shelter", "es": "Andá al refugio"},
 "tagline": {"en": "guides, records and warns", "es": "guía, registra y avisa"}, "by": {"en": "by WearMeds", "es": "de WearMeds"},
 "team": {"en": "Team Sanjo-X · NASA Space Apps Challenge 2026 · Rosario", "es": "Equipo Sanjo-X · NASA Space Apps Challenge 2026 · Rosario"},
 "only": {"en": "Shocks come only from validated medical devices.", "es": "Las descargas las dan solo dispositivos médicos validados."},
 "months": {"en": "7 months", "es": "7 meses"}, "nodoc": {"en": "no doctor", "es": "sin médico"}, "people": {"en": "15 people", "es": "15 personas"},
 "earth": {"en": "Earth", "es": "Tierra"}, "mars": {"en": "Mars", "es": "Marte"}, "upto": {"en": "up to 20 min", "es": "hasta 20 min"},
}
def tr(k): return L[k][LANG]
def T(s, size=36, color=CREAM, **kw): return Text(s, font=FONT, font_size=size, color=color, **kw)

def wrap(s, n=44):
    out, line = [], ""
    for w in s.split():
        if len(line) + len(w) + 1 > n: out.append(line); line = w
        else: line = (line + " " + w).strip()
    out.append(line); return "\n".join(out)

class Story(Scene):
    sid = ""
    def setup(self):
        self.d = DUR[self.sid]
        self.add_sound(os.path.join(HERE, "narr", LANG, self.sid + ".wav"), time_offset=0.35, gain=-3)
        cap = T(wrap(SCRIPT[self.sid][LANG], 62), 32, CREAM, line_spacing=0.9)
        box = RoundedRectangle(width=cap.width + .6, height=cap.height + .4, corner_radius=.18, fill_color=BLACK, fill_opacity=.72, stroke_width=0)
        self.cap = VGroup(box, cap).to_edge(DOWN, buff=.3); cap.move_to(box)
        self.add_foreground_mobject(self.cap)
        self.t0 = 0.0
    def sfx(self, name, gain=-12): self.add_sound(os.path.join(SFX, name + ".wav"), gain=gain)
    def hold(self):
        """wait so the scene lasts at least narration + tail"""
        el = self.renderer.time
        if el < self.d + 0.9: self.wait(self.d + 0.9 - el)

def person(color=CREAM, band=True, s=1.0):
    head = Circle(.2, fill_color=color, fill_opacity=1, stroke_width=0).shift(UP * .56)
    body = RoundedRectangle(width=.56, height=.66, corner_radius=.24, fill_color=color, fill_opacity=1, stroke_width=0)
    g = VGroup(body, head)
    if band: g.add(RoundedRectangle(width=.16, height=.09, corner_radius=.03, fill_color=CYAN, fill_opacity=1, stroke_width=0).move_to([.27, -.12, 0]))
    return g.scale(s)

def heart(color=CORAL, s=1.0):
    h = VMobject(fill_color=color, fill_opacity=1, stroke_width=0)
    t = np.linspace(0, TAU, 80); x = 16 * np.sin(t) ** 3; y = 13 * np.cos(t) - 5 * np.cos(2 * t) - 2 * np.cos(3 * t) - np.cos(4 * t)
    h.set_points_smoothly([np.array([a, b, 0]) / 40 for a, b in zip(x, y)]); return h.scale(s)

def ecg(width=3, flat=False, color=GREEN):
    pts = []; n = 3
    for k in range(n):
        x0 = -width / 2 + k * width / n; w = width / n
        if flat: pts += [[x0, 0, 0], [x0 + w, 0, 0]]
        else: pts += [[x0, 0, 0], [x0 + w * .35, 0, 0], [x0 + w * .42, .12, 0], [x0 + w * .48, -.1, 0], [x0 + w * .55, .55, 0], [x0 + w * .62, -.3, 0], [x0 + w * .7, 0, 0], [x0 + w, 0, 0]]
    return VMobject(stroke_color=color, stroke_width=5).set_points_as_corners(pts)

def envelope(color=CYAN, s=1.0):
    r = Rectangle(width=.7, height=.46, fill_color=color, fill_opacity=1, stroke_width=0)
    f = Polygon([-.35, .23, 0], [.35, .23, 0], [0, -.04, 0], fill_color="#0d0d10", fill_opacity=1, stroke_width=0)
    return VGroup(r, f).scale(s)

def planet(color, r, s=None):
    c = Circle(r, fill_color=color, fill_opacity=1, stroke_width=0)
    shade = Circle(r, fill_color=BLACK, fill_opacity=.35, stroke_width=0).shift(RIGHT * r * .35 + DOWN * r * .2)
    shade = Intersection(c, shade, fill_color=BLACK, fill_opacity=.3, stroke_width=0)
    return VGroup(c, shade)

def stars(n=140):
    rng = np.random.default_rng(3)
    return VGroup(*[Dot([rng.uniform(-7.2, 7.2), rng.uniform(-4, 4), 0], radius=rng.uniform(.008, .022), color=WHITE, fill_opacity=rng.uniform(.25, .8)) for _ in range(n)])

def cabin_bg():
    hull = RoundedRectangle(width=12.6, height=4.6, corner_radius=1.3, fill_color="#0e0f12", fill_opacity=1, stroke_color="#2a2a2e", stroke_width=2).shift(UP * .95)
    wins = VGroup(*[Circle(.28, fill_color="#05070d", fill_opacity=1, stroke_color="#3a3a40", stroke_width=3).move_to([x, 2.6, 0]) for x in np.linspace(-5, 5, 6)])
    sp = VGroup(*[Dot(w.get_center() + np.array([np.cos(a) * .15, np.sin(a) * .12, 0]), radius=.015, color=WHITE) for w in wins for a in (0.4, 2.2, 4.1)])
    return VGroup(hull, wins, sp)

# ------------------------------------------------------------------ scenes
class N01_Ship(Story):
    sid = "01_ship"
    def construct(self):
        self.add(stars()); bg = cabin_bg(); self.play(FadeIn(bg), run_time=.6)
        seats = VGroup(*[person(CREAM, s=.62).move_to([-4.4 + (i % 5) * 2.2, 1.45 - (i // 5) * 1.05, 0]) for i in range(15)])
        self.play(LaggedStart(*[FadeIn(p, shift=UP * .2, scale=.8) for p in seats], lag_ratio=.07), run_time=1.6)
        icons = VGroup(VGroup(person(CREAM, False, .6), T(tr("people"), 30)).arrange(RIGHT, buff=.2),
                       VGroup(Square(.5, fill_color=CORAL, fill_opacity=1, stroke_width=0), T(tr("months"), 30)).arrange(RIGHT, buff=.2),
                       VGroup(VGroup(Circle(.26, stroke_color=RED, stroke_width=5), Line(LEFT * .18 + DOWN * .18, RIGHT * .18 + UP * .18, color=RED, stroke_width=5)), T(tr("nodoc"), 30)).arrange(RIGHT, buff=.2)
                       ).arrange(RIGHT, buff=1.0).move_to(DOWN * 1.75)
        icons.set_z_index(5)
        ib = BackgroundRectangle(icons, color=BLACK, fill_opacity=.8, buff=.15)
        self.play(FadeIn(ib), LaggedStart(*[FadeIn(i, shift=DOWN * .2) for i in icons], lag_ratio=.5), run_time=2.2)
        self.hold()

class N02_Earth(Story):
    sid = "02_earth"
    def construct(self):
        self.add(stars())
        earth = planet("#3F7DFF", .9).move_to(LEFT * 5 + UP * .6); mars = planet(CORAL, .55).move_to(RIGHT * 5 + UP * .6)
        el = T(tr("earth"), 28, MUTED).next_to(earth, DOWN); ml = T(tr("mars"), 28, MUTED).next_to(mars, DOWN)
        path = DashedLine(mars.get_left(), earth.get_right(), dash_length=.12, color=MUTED, stroke_opacity=.6).shift(UP * .0)
        self.play(FadeIn(earth, scale=.8), FadeIn(mars, scale=.8), FadeIn(el), FadeIn(ml), Create(path), run_time=1.0)
        env = envelope(s=.8).move_to(mars.get_left() + LEFT * .5)
        clock = VGroup(Circle(.35, stroke_color=CREAM, stroke_width=4), Line(ORIGIN, UP * .25, color=CREAM, stroke_width=4)).move_to(UP * 2.2)
        hand = clock[1]; up = T(tr("upto"), 30, CYAN).next_to(clock, RIGHT, buff=.25)
        self.play(FadeIn(env, scale=.6), FadeIn(clock), FadeIn(up), run_time=.5); self.sfx("pro_tap_glass")
        half = max(1.2, (self.d - 1.6) / 2)
        self.play(env.animate.move_to(earth.get_right() + RIGHT * .5), Rotate(hand, -TAU * 2, about_point=clock[0].get_center()), run_time=half, rate_func=linear)
        env2 = envelope(color=YEL, s=.8).move_to(env)
        self.play(ReplacementTransform(env, env2), run_time=.3)
        self.play(env2.animate.move_to(mars.get_left() + LEFT * .5), Rotate(hand, -TAU * 2, about_point=clock[0].get_center()), run_time=half, rate_func=linear)
        self.hold()

class N03_Collapse(Story):
    sid = "03_collapse"
    def construct(self):
        bg = cabin_bg(); self.add(bg)
        a = person(CREAM, s=1.3).move_to(LEFT * 3 + UP * .3); v = person(CORAL, s=1.3).move_to(UP * .3); b = person(CREAM, s=1.3).move_to(RIGHT * 3 + UP * .3)
        self.play(FadeIn(VGroup(a, v, b), shift=UP * .2), run_time=.6)
        h = heart(s=.9).next_to(v, UP, buff=.5); line = ecg(2.4).next_to(h, RIGHT, buff=.3)
        self.play(FadeIn(h), Create(line), run_time=.8)
        self.sfx("notify_m2", -6)
        flat = ecg(2.4, flat=True, color=RED).move_to(line)
        self.play(Rotate(v, -PI / 2.3, about_point=v.get_bottom()), v.animate.shift(DOWN * .35), Transform(line, flat), h.animate.set_color(RED), run_time=1.0, rate_func=rush_into)
        for _ in range(2): self.play(h.animate.scale(1.2), run_time=.2); self.play(h.animate.scale(1 / 1.2), run_time=.2)
        env = envelope(s=.7).move_to(RIGHT * 4.6 + UP * 2.3); q = T("?", 46, CYAN).next_to(env, RIGHT, buff=.15)
        x = VGroup(Line(UL * .4, DR * .4, color=RED, stroke_width=8), Line(UR * .4, DL * .4, color=RED, stroke_width=8)).move_to(env)
        self.play(FadeIn(env), FadeIn(q)); self.play(Create(x), run_time=.5)
        self.hold()

class N04_Meet(Story):
    sid = "04_meet"
    def construct(self):
        p = person(CREAM, band=False, s=3.2).shift(UP * .5)
        strap = VGroup(Line([-.75, 1.3, 0], [.15, -.2, 0]), Line([.75, 1.3, 0], [-.15, -.2, 0])).set_stroke("#3a3a40", 22).shift(UP * .3)
        core = RoundedRectangle(width=.55, height=.55, corner_radius=.12, fill_color="#3a3a40", fill_opacity=1, stroke_width=0).move_to(UP * .2)
        led = Circle(.09, fill_color=CORAL, fill_opacity=1, stroke_width=0).move_to(core)
        band = RoundedRectangle(width=.62, height=.32, corner_radius=.1, fill_color="#1c1c20", fill_opacity=1, stroke_width=0).move_to([1.02, -.2, 0])
        scr = RoundedRectangle(width=.46, height=.2, corner_radius=.05, fill_color=CYAN, fill_opacity=1, stroke_width=0).move_to(band)
        self.play(FadeIn(p, scale=.9), run_time=.7)
        self.sfx("pro_pop_in_round"); self.play(FadeIn(band, scale=.5), FadeIn(scr, scale=.5), run_time=.5)
        wl = T(tr("wristband"), 32, CYAN).next_to(band, RIGHT, buff=.5); wa = Arrow(wl.get_left(), band.get_right(), buff=.1, color=CYAN, stroke_width=3)
        self.play(FadeIn(wl), GrowArrow(wa), run_time=.6)
        self.sfx("pro_pop_in_round"); self.play(Create(strap), FadeIn(core), FadeIn(led), run_time=.8)
        hl = T(tr("harness"), 32, CORAL).next_to(core, LEFT, buff=2.0); ha = Arrow(hl.get_right(), core.get_left(), buff=.1, color=CORAL, stroke_width=3)
        self.play(FadeIn(hl), GrowArrow(ha), run_time=.6)
        self.play(led.animate.scale(1.5).set_opacity(.6), rate_func=there_and_back, run_time=.6)
        self.hold()

class N05_Band(Story):
    sid = "05_band"
    def construct(self):
        d = self.d; X = 2.6
        def lab(txt, y, col=CREAM): return T(txt, 26, col).move_to([-6.6, y, 0], aligned_edge=LEFT)
        skin = Rectangle(width=7.2, height=.6, fill_color=SKIN, fill_opacity=1, stroke_width=0).move_to([X, -.25, 0])
        sl = lab(tr("skin"), -.25, SKIN)
        inner = RoundedRectangle(width=4.2, height=.45, corner_radius=.2, fill_color="#d9d8d4", fill_opacity=1, stroke_width=0).next_to(skin, UP, buff=0)
        leds = VGroup(*[Circle(.07, fill_color=c, fill_opacity=1, stroke_width=0) for c in (RED, "#8a1f1f", GREEN, RED)]).arrange(RIGHT, buff=.16).move_to(inner.get_bottom() + UP * .12)
        cell = RoundedRectangle(width=.5, height=.22, corner_radius=.05, fill_color=GREEN, fill_opacity=1, stroke_width=0).move_to(inner.get_center() + RIGHT * 1.4)
        il = lab(tr("inner"), inner.get_y())
        self.play(FadeIn(skin), FadeIn(sl), run_time=.5)
        self.play(FadeIn(inner, shift=DOWN * .3), FadeIn(leds), FadeIn(cell), FadeIn(il), run_time=.8)
        rays = VGroup(*[Line(l.get_bottom(), l.get_bottom() + DOWN * .5 + RIGHT * (i - 1.5) * .12, color=l.get_fill_color(), stroke_width=4) for i, l in enumerate(leds)])
        self.play(LaggedStart(*[Create(r) for r in rays], lag_ratio=.15), run_time=.8)
        self.play(rays.animate.set_opacity(.25), rate_func=there_and_back, run_time=.6)
        outer = RoundedRectangle(width=4.6, height=.75, corner_radius=.25, fill_color="#1c1c20", fill_opacity=1, stroke_color="#3a3a40", stroke_width=2).move_to([X, 3.0, 0])
        scr = RoundedRectangle(width=3.6, height=.45, corner_radius=.1, fill_color=CYAN, fill_opacity=1, stroke_width=0).move_to(outer)
        batt = RoundedRectangle(width=.9, height=.26, corner_radius=.06, fill_color=GREEN, fill_opacity=1, stroke_width=0).next_to(outer, UP, buff=.12).shift(LEFT * 1.4)
        ol = lab(tr("outer"), 3.0)
        top = VGroup(outer, scr, batt)
        self.play(FadeIn(top, shift=DOWN * .4), FadeIn(ol), run_time=.8)
        mags = VGroup(*[VGroup(Rectangle(width=.28, height=.2, fill_color="#c33", fill_opacity=1, stroke_width=0), Rectangle(width=.28, height=.2, fill_color="#777", fill_opacity=1, stroke_width=0).shift(DOWN * .2)) for _ in range(2)])
        mags[0].move_to(outer.get_bottom() + DOWN * .22 + LEFT * 1.8); mags[1].move_to(outer.get_bottom() + DOWN * .22 + RIGHT * 1.8)
        ml = T(tr("magnets"), 24, MUTED).next_to(mags[1], RIGHT, buff=.25)
        self.play(FadeIn(mags), FadeIn(ml), run_time=.5)
        grp = VGroup(top, mags, ml)
        gap = (mags.get_bottom()[1]) - inner.get_top()[1]
        self.sfx("pro_pop_in_round", -8)
        self.play(grp.animate.shift(DOWN * gap), ol.animate.move_to([-6.4, inner.get_top()[1] + 1.0, 0], aligned_edge=LEFT), run_time=.7, rate_func=rate_functions.ease_out_back)
        spark = VGroup(*[Line(ORIGIN, UP * .25).rotate(a).shift(inner.get_top() + RIGHT * k) for a in np.linspace(-1, 1, 3) for k in (-1.8, 1.8)]).set_stroke(YEL, 3)
        self.play(FadeIn(spark), run_time=.2); self.play(FadeOut(spark), run_time=.3)
        self.wait(max(.3, d * .45 - 4.9))
        self.play(grp.animate.shift(UP * .5), ol.animate.shift(UP * .5), run_time=.6, rate_func=smooth)
        suit = Rectangle(width=7.2, height=.4, fill_color=SUIT, fill_opacity=1, stroke_width=0).move_to(inner.get_top() + UP * .2)
        stl = lab(tr("suit"), suit.get_y(), SUIT)
        self.play(FadeIn(suit, shift=RIGHT * 2), FadeIn(stl), run_time=.8)
        self.play(grp.animate.shift(DOWN * .1), ol.animate.shift(DOWN * .1), run_time=.5, rate_func=rate_functions.ease_out_back)
        own = T(tr("own"), 24, GREEN).next_to(skin, DOWN, buff=.12).align_to(skin, RIGHT)
        self.play(cell.animate.scale(1.3), FadeIn(own), run_time=.4); self.play(cell.animate.scale(1 / 1.3), run_time=.3)
        self.hold()

class N06_Helper(Story):
    sid = "06_helper"
    def construct(self):
        deck = RoundedRectangle(width=12, height=4.8, corner_radius=1.3, fill_color="#0e0f12", fill_opacity=1, stroke_color="#2a2a2e", stroke_width=2).shift(UP * .9)
        rng = np.random.default_rng(11)
        pts = [np.array([x, y, 0]) for x, y in zip(rng.uniform(-5, 5, 15), rng.uniform(-.9, 2.6, 15))]
        ppl = VGroup(*[person(CREAM, s=.55).move_to(p) for p in pts])
        self.add(deck); self.play(LaggedStart(*[FadeIn(p, scale=.7) for p in ppl], lag_ratio=.04), run_time=.8)
        v = 4; vic = ppl[v]; self.sfx("notify_m2", -6)
        self.play(vic.animate.set_color(RED), run_time=.3)
        for _ in range(2):
            self.play(Circle(.3, color=RED, stroke_width=4).move_to(vic).animate.scale(9).set_stroke(opacity=0), run_time=.6, rate_func=rush_from)
        h = int(np.argmin([np.linalg.norm(pts[i] - pts[v]) if i != v else 99 for i in range(15)]))
        hp = ppl[h]; self.sfx("pro_notify_ding_1930", -8)
        gt = T(tr("go"), 26, BLACK); bub = VGroup(RoundedRectangle(width=gt.width + .5, height=.6, corner_radius=.2, fill_color=YEL, fill_opacity=1, stroke_width=0), gt)
        bub[1].move_to(bub[0]); bub.next_to(hp, UP, buff=.2)
        self.play(hp.animate.set_color(YEL).scale(1.3), FadeIn(bub, shift=UP * .1), run_time=.5)
        self.play(hp.animate.move_to(vic.get_center() + LEFT * .55), bub.animate.shift(vic.get_center() + LEFT * .55 - hp.get_center()), run_time=1.3, rate_func=smooth)
        self.hold()

class N07_CPR(Story):
    sid = "07_cpr"
    def construct(self):
        d = self.d
        band = RoundedRectangle(width=7.4, height=3.6, corner_radius=.6, fill_color="#1c1c20", fill_opacity=1, stroke_width=0).shift(UP * .7 + LEFT * 2.2)
        scr = RoundedRectangle(width=6.6, height=2.9, corner_radius=.35, fill_color="#0b1820", fill_opacity=1, stroke_color=CYAN, stroke_width=3).move_to(band)
        self.play(FadeIn(band, scale=.9), FadeIn(scr, scale=.9), run_time=.6)
        hands = VGroup(RoundedRectangle(width=1.0, height=.45, corner_radius=.2, fill_color=CREAM, fill_opacity=1, stroke_width=0),
                       RoundedRectangle(width=1.0, height=.45, corner_radius=.2, fill_color=CREAM, fill_opacity=.7, stroke_width=0).shift(UP * .28 + RIGHT * .1))
        steps = [VGroup(hands.copy(), T("1  " + tr("s1"), 34)), VGroup(VGroup(*[Circle(.25 + .2 * k, stroke_color=CYAN, stroke_width=4, stroke_opacity=1 - .3 * k) for k in range(3)]), T("2  " + tr("s2"), 34)), VGroup(person(CREAM, False, .7), T("3  " + tr("s3"), 34))]
        for s in steps: s.arrange(RIGHT, buff=.5).move_to(scr)
        cur = steps[0]; self.sfx("pro_typing_key_soft"); self.play(FadeIn(cur, shift=UP * .2), run_time=.4)
        per = max(.9, (d - 3.2) / 3)
        self.wait(per * .6)
        self.sfx("pro_typing_key_soft"); self.play(ReplacementTransform(cur, steps[1]), run_time=.4); cur = steps[1]
        for _ in range(3): self.play(cur[0].animate.scale(1.15), run_time=.15); self.play(cur[0].animate.scale(1 / 1.15), run_time=.15)
        self.sfx("pro_typing_key_soft"); self.play(ReplacementTransform(cur, steps[2]), run_time=.4)
        # harness checks and shocks
        core = RoundedRectangle(width=1.4, height=1.4, corner_radius=.3, fill_color="#3a3a40", fill_opacity=1, stroke_width=0).move_to(RIGHT * 4.6 + UP * .7)
        hl = T(tr("harness"), 28, CORAL).next_to(core, UP, buff=.2)
        bolt = Polygon([0, .5, 0], [-.25, 0, 0], [0, 0, 0], [-.1, -.5, 0], [.25, .05, 0], [0, .05, 0], fill_color=YEL, fill_opacity=1, stroke_width=0).move_to(core)
        self.play(FadeIn(core), FadeIn(hl), run_time=.4)
        self.sfx("pro_impact_short", -6); self.play(FadeIn(bolt, scale=1.6), Flash(core, color=YEL, line_length=.4, num_lines=12), run_time=.5)
        h = heart(GREEN, .7).next_to(core, DOWN, buff=.4); beat = ecg(2.2).next_to(h, DOWN, buff=.2)
        self.play(FadeIn(h), Create(beat), run_time=.8)
        self.hold()

class N08_Sun(Story):
    sid = "08_sun"
    def construct(self):
        self.add(stars())
        sun = Circle(1.3, fill_color=AMBER, fill_opacity=1, stroke_width=0).shift(UP * .7)
        glow = VGroup(*[Circle(1.3 + .25 * k, fill_color=AMBER, fill_opacity=.12, stroke_width=0).move_to(sun) for k in range(1, 6)])
        self.play(FadeIn(glow), FadeIn(sun), run_time=.6)
        rays = VGroup(*[Line(sun.get_center() + 1.4 * np.array([np.cos(a), np.sin(a), 0]), sun.get_center() + 6 * np.array([np.cos(a), np.sin(a), 0]), color=YEL, stroke_width=3) for a in np.linspace(0, TAU, 28, endpoint=False)])
        self.sfx("riser_tension", -10)
        self.play(LaggedStart(*[Create(r) for r in rays], lag_ratio=.03), sun.animate.set_color("#FFF1C2"), run_time=1.2)
        self.hold()

class N09_Shelter(Story):
    sid = "09_shelter"
    def construct(self):
        deck = RoundedRectangle(width=12, height=4.8, corner_radius=1.3, fill_color="#0e0f12", fill_opacity=1, stroke_color="#2a2a2e", stroke_width=2).shift(UP * .9)
        room = Rectangle(width=3.2, height=3.0, fill_color="#1a2a33", fill_opacity=1, stroke_color=CYAN, stroke_width=10).move_to(RIGHT * 3.8 + UP * .9)
        rl = T(tr("shelter"), 28, CYAN).next_to(room, UP, buff=.12)
        rng = np.random.default_rng(5)
        pts = [np.array([x, y, 0]) for x, y in zip(rng.uniform(-5.2, 1.2, 15), rng.uniform(-.8, 2.6, 15))]
        ppl = VGroup(*[person(CREAM, s=.5).move_to(p) for p in pts])
        self.add(deck, room, rl, ppl)
        self.sfx("notify_m2", -6)
        alert = VGroup(RoundedRectangle(width=4.6, height=.7, corner_radius=.2, fill_color=RED, fill_opacity=1, stroke_width=0), T(tr("toshelter"), 30, WHITE)); alert[1].move_to(alert[0]); alert.move_to(LEFT * 2.5 + DOWN * 1.85)
        self.play(*[p[2].animate.set_color(RED) for p in ppl], FadeIn(alert, shift=DOWN * .2), run_time=.5)
        slots = [room.get_center() + np.array([-1.0 + (i % 5) * .5, .9 - (i // 5) * .9, 0]) for i in range(15)]
        self.play(LaggedStart(*[p.animate.move_to(slots[i]) for i, p in enumerate(ppl)], lag_ratio=.06), run_time=max(1.8, self.d - 2.6), rate_func=smooth)
        parts = VGroup(*[Dot(radius=.05, color=YEL).move_to([rng.uniform(-6, 6), 3.6, 0]) for _ in range(40)])
        self.play(LaggedStart(*[d.animate.move_to([d.get_x() * .2 + 1, rng.uniform(-1, 2.5), 0]).set_opacity(0) for d in parts], lag_ratio=.02), run_time=1.2)
        self.hold()

class N10_Records(Story):
    sid = "10_records"
    def construct(self):
        self.add(stars())
        book = VGroup(RoundedRectangle(width=2.4, height=3.0, corner_radius=.15, fill_color=CREAM, fill_opacity=1, stroke_width=0))
        lines = VGroup(*[VGroup(Dot(radius=.07, color=GREEN), Rectangle(width=1.4, height=.1, fill_color="#9a9893", fill_opacity=1, stroke_width=0)).arrange(RIGHT, buff=.15) for _ in range(5)]).arrange(DOWN, buff=.3, aligned_edge=LEFT)
        book.add(lines.move_to(book)); book.move_to(LEFT * 4 + UP * .5)
        self.play(FadeIn(book[0], shift=UP * .2), run_time=.4)
        for l in lines: self.sfx("pro_typing_key_soft", -14); self.play(FadeIn(l, shift=RIGHT * .2), run_time=.25)
        earth = planet("#3F7DFF", .8).move_to(RIGHT * 4.6 + UP * .6)
        self.play(FadeIn(earth), run_time=.4)
        env = envelope(s=.8).move_to(book.get_right() + RIGHT * .6)
        self.sfx("pro_tap_glass"); self.play(FadeIn(env), run_time=.3)
        self.play(env.animate.move_to(earth.get_left() + LEFT * .5), run_time=1.4, rate_func=smooth)
        chk = VGroup(Circle(.3, fill_color=GREEN, fill_opacity=1, stroke_width=0), VMobject(stroke_color=BLACK, stroke_width=6).set_points_as_corners([[-.13, 0, 0], [-.03, -.1, 0], [.14, .1, 0]])).next_to(earth, UP, buff=.2)
        self.sfx("pro_notify_ding_1930", -10); self.play(FadeIn(chk, scale=.5), run_time=.4)
        self.hold()

class N11_Mars(Story):
    sid = "11_mars"
    def construct(self):
        self.add(stars()); m = planet(CORAL, .4).shift(UP * .6)
        self.play(m.animate.scale(4.5), run_time=max(1.5, self.d + .5), rate_func=smooth)
        self.hold()

class N12_Close(Story):
    sid = "12_close"
    def construct(self):
        self.remove_foreground_mobject(self.cap); self.remove(self.cap)
        wm = T("Eva", 140); tg = T(tr("tagline"), 44, CORAL).next_to(wm, DOWN, buff=.3); by = T(tr("by"), 34, MUTED).next_to(tg, DOWN, buff=.3)
        G = VGroup(wm, tg, by).shift(UP * .6 + LEFT * 2)
        self.sfx("logo_sting_soft_bloom", -6)
        self.play(FadeIn(wm, scale=.9), run_time=.8); self.play(FadeIn(tg, shift=UP * .1), FadeIn(by), run_time=.6)
        only = T(tr("only"), 26, MUTED).to_edge(DOWN, buff=1.0); team = T(tr("team"), 22, MUTED).to_edge(DOWN, buff=.45)
        self.play(FadeIn(only), FadeIn(team), run_time=.5)
        self.wait(max(1.5, self.d + 1.5 - self.renderer.time))
        self.play(FadeOut(VGroup(G, only, team)), run_time=.8)
