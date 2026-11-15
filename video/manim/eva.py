"""Eva by WearMeds — explainer in the style of a math-animation video, built with Manim Community.
Original vector assets only (drawn here in code). Run: manim -r 1920,1080 --fps 30 eva.py <Scene>"""
from manim import *
import numpy as np

config.background_color = "#000000"
CORAL = "#FF6A3D"; CREAM = "#F4F3F0"; CYAN = "#7FD4FF"; AMBER = "#FFB347"; MUTED = "#8A8986"; RED = "#FF2A1F"; YEL = "#FFD84A"
FONT = "Inter"
import os
SFX = os.path.join(os.path.dirname(os.path.abspath(__file__)), "sfx")
def sfx(scene, name, gain=-8):
    """Earshot bank sounds (MIT, synthesised from code), placed at the current animation time."""
    scene.add_sound(os.path.join(SFX, name + ".wav"), gain=gain)

def T(s, size=36, color=CREAM, **kw):
    return Text(s, font=FONT, font_size=size, color=color, **kw)

def wristband(scale=1.0):
    band = RoundedRectangle(width=3.2, height=1.0, corner_radius=0.5, fill_color="#2a2a2e", fill_opacity=1, stroke_width=0)
    face = RoundedRectangle(width=1.7, height=0.78, corner_radius=0.2, fill_color="#0d0d10", fill_opacity=1, stroke_width=0)
    scr = RoundedRectangle(width=1.5, height=0.6, corner_radius=0.14, fill_color=CYAN, fill_opacity=0.9, stroke_width=0)
    pulse = VMobject(stroke_color="#0d0d10", stroke_width=4).set_points_as_corners(
        [[-0.6, 0, 0], [-0.25, 0, 0], [-0.12, 0.2, 0], [0.0, -0.22, 0], [0.12, 0.14, 0], [0.22, 0, 0], [0.6, 0, 0]])
    return VGroup(band, face, scr, pulse).scale(scale)

def harness(scale=1.0):
    s = dict(fill_opacity=1, stroke_width=0)
    l = Polygon([-1.1, 1.4, 0], [-0.75, 1.4, 0], [-0.05, 0.1, 0], [-0.35, -0.05, 0], fill_color="#3a3a40", **s)
    r = l.copy().flip(UP)
    belt = RoundedRectangle(width=2.6, height=0.36, corner_radius=0.18, fill_color="#3a3a40", **s).shift(DOWN * 0.55)
    core = RoundedRectangle(width=0.9, height=0.9, corner_radius=0.2, fill_color=CREAM, **s).shift(DOWN * 0.05)
    light = Circle(0.13, fill_color=CORAL, **s).move_to(core)
    padL = RoundedRectangle(width=0.55, height=0.8, corner_radius=0.2, fill_color=CYAN, **s).move_to([-0.95, 0.75, 0]).rotate(-0.4)
    padR = padL.copy().move_to([0.95, 0.75, 0]).rotate(0.8)
    return VGroup(l, r, belt, padL, padR, core, light).scale(scale)

class S1_Title(Scene):
    def construct(self):
        stars = VGroup(*[Dot(np.array([np.random.uniform(-7.2, 7.2), np.random.uniform(-4, 4), 0]), radius=np.random.uniform(.008, .022),
                             color=WHITE, fill_opacity=np.random.uniform(.2, .8)) for _ in range(160)])
        self.add(stars)
        eva = T("Eva", 150); by = T("by WearMeds", 44, MUTED).next_to(eva, DOWN, buff=0.25)
        g = VGroup(eva, by).shift(LEFT * 2.2)
        sfx(self, 'logo_sting_soft_bloom', -6); self.play(Write(eva, run_time=1.6, rate_func=smooth))
        self.play(FadeIn(by, shift=UP * 0.2), run_time=0.8)
        line = T("Care that guides, records and warns", 40).next_to(g, DOWN, buff=0.7).align_to(g, LEFT)
        line2 = T("on the way to Mars.", 40).next_to(line, DOWN, buff=0.18).align_to(line, LEFT)
        for w in ["guides", "records", "warns"]:
            line[line.text.index(w) - line.text[:line.text.index(w)].count(" "):][:len(w)].set_color(CORAL)
        sfx(self, 'pro_shimmer_m1', -14); self.play(LaggedStart(FadeIn(line, shift=UP * .15), FadeIn(line2, shift=UP * .15), lag_ratio=.4), run_time=1.4)
        self.wait(3.5)  # mascot is composited on the right here
        self.play(FadeOut(VGroup(g, line, line2), shift=UP * .3), FadeOut(stars), run_time=1)

class S2_Distance(Scene):
    def construct(self):
        AU = 1.45
        sun = Dot(ORIGIN, radius=0.18, color=AMBER)
        eo = Circle(AU, stroke_color=MUTED, stroke_width=1.2, stroke_opacity=.5); mo = Circle(AU * 1.524, stroke_color=MUTED, stroke_width=1.2, stroke_opacity=.5)
        orbits = VGroup(sun, eo, mo).shift(LEFT * 4.5 + DOWN * .55)
        C = orbits[0].get_center()
        dth = ValueTracker(170 * DEGREES)
        earth = always_redraw(lambda: Dot(C + AU * np.array([np.cos(0.0), np.sin(0.0), 0]), radius=0.11, color="#4F8FFF"))
        mars = always_redraw(lambda: Dot(C + AU * 1.524 * np.array([np.cos(dth.get_value()), np.sin(dth.get_value()), 0]), radius=0.09, color=CORAL))
        link = always_redraw(lambda: DashedLine(earth.get_center(), mars.get_center(), dash_length=.08, stroke_width=2, color=CREAM, stroke_opacity=.6))
        def dist_au(): return np.sqrt(1 + 1.524 ** 2 - 2 * 1.524 * np.cos(dth.get_value()))
        km = always_redraw(lambda: T(f"{dist_au() * 149.6:,.0f} million km", 38).move_to(LEFT * 1.5 + UP * 1.2, aligned_edge=LEFT))
        lt = always_redraw(lambda: T(f"one-way radio delay  {dist_au() * 499 / 60:4.1f} min", 32, CYAN).next_to(km, DOWN, aligned_edge=LEFT, buff=.3))
        eL = T("Earth", 24, MUTED).add_updater(lambda m: m.next_to(earth, DOWN, buff=.12))
        mL = T("Mars", 24, MUTED).add_updater(lambda m: m.next_to(mars, UP, buff=.12))
        title = T("Help from Earth is minutes away", 46).to_corner(UL, buff=.7)
        sfx(self, 'whoosh_fast_crisp', -12); self.play(FadeIn(title, shift=DOWN * .2), Create(eo), Create(mo), GrowFromCenter(sun), run_time=1.5)
        self.add(earth, mars, link, eL, mL); self.play(FadeIn(km), FadeIn(lt))
        self.play(dth.animate.set_value(15 * DEGREES), run_time=5, rate_func=there_and_back_with_pause if False else smooth)
        self.play(dth.animate.set_value(120 * DEGREES), run_time=3, rate_func=smooth)
        note = T("Circular orbits, simplified. Real range: about 3 to 22 min each way.", 18, MUTED).next_to(lt, DOWN, aligned_edge=LEFT, buff=.3)
        self.play(FadeIn(note))
        # a question and its answer
        q = T("\"Is she breathing?\"", 30, CREAM)
        pkt = Dot(radius=.07, color=CYAN)
        pkt.move_to(mars.get_center()); q.next_to(mars, UP, buff=.3).align_to(mars, LEFT)
        self.play(FadeIn(q, shift=LEFT * .1), run_time=.6)
        sfx(self, 'pro_tap_glass', -10); self.play(MoveAlongPath(pkt, Line(mars.get_center(), earth.get_center())), run_time=2.2, rate_func=linear)
        a = T("The answer arrives one round trip later.", 26, CYAN).next_to(note, DOWN, aligned_edge=LEFT, buff=.9)
        self.play(MoveAlongPath(pkt, Line(earth.get_center(), mars.get_center())), FadeIn(a), run_time=2.2, rate_func=linear)
        key = T("Some emergencies are decided in seconds.", 28, CORAL).next_to(a, DOWN, aligned_edge=LEFT, buff=.3)
        sfx(self, 'pro_impact_short', -8); self.play(Write(key), run_time=1.4); self.wait(2.2)
        self.play(*[FadeOut(m) for m in self.mobjects])

class S3_Minutes(Scene):
    def construct(self):
        ax = Axes(x_range=[0, 12, 2], y_range=[0, 100, 25], x_length=6.4, y_length=4.0, tips=False,
                  axis_config=dict(color=MUTED, stroke_width=1.5)).shift(DOWN * 1.0 + LEFT * 2.55)
        nums = VGroup(*[T(str(v), 22, MUTED).next_to(ax.c2p(v, 0), DOWN, buff=.15) for v in range(0, 13, 2)], *[T(str(v), 22, MUTED).next_to(ax.c2p(0, v), LEFT, buff=.15) for v in range(25, 101, 25)])
        xl = T("minutes without a shock", 24, MUTED).next_to(ax.x_axis, DOWN, buff=.45)
        yl = T("chance of survival (%)", 22, MUTED).next_to(ax.y_axis, UP, buff=.25).align_to(ax.y_axis, LEFT)
        title = T("Cardiac arrest: every minute counts", 46).to_corner(UL, buff=.6)
        sfx(self, 'whoosh_fast_crisp', -12); self.play(FadeIn(title, shift=DOWN * .2), Create(ax), FadeIn(nums), FadeIn(xl), FadeIn(yl), run_time=1.6)
        hi = ax.plot(lambda t: max(0, 100 - 7 * t), x_range=[0, 12], color=CORAL, stroke_width=3)
        lo = ax.plot(lambda t: max(0, 100 - 10 * t), x_range=[0, 10], color=CORAL, stroke_width=3)
        band = ax.get_area(hi, x_range=[0, 10], bounded_graph=lo, color=CORAL, opacity=.25)
        self.play(Create(lo), Create(hi), run_time=2.4, rate_func=smooth); self.play(FadeIn(band))
        lab = T("Without a shock, survival falls about 7–10% every minute.", 26, CORAL).next_to(title, DOWN, aligned_edge=LEFT, buff=.3)
        self.play(FadeIn(lab, shift=UP * .1))
        rt = Arrow(ax.c2p(9.5, 92), ax.c2p(12.4, 92), buff=0, color=CYAN, stroke_width=3)
        rtl = T("Earth's answer:\n6–44 min round trip", 22, CYAN).next_to(rt, DOWN, buff=.12).align_to(rt, RIGHT)
        self.play(GrowArrow(rt), FadeIn(rtl), run_time=1.2)
        zone = ax.get_area(ax.plot(lambda t: 100, x_range=[0, 3]), x_range=[0, 3], color=CYAN, opacity=.14)
        zl = T("on board,\nnow", 24, CYAN).move_to(ax.c2p(1.5, 14))
        self.play(FadeIn(zone), FadeIn(zl)); self.wait(.6)
        steps = VGroup(
            T("1  Harness detects the rhythm (validated algorithm)", 26),
            T("2  Eva alerts the nearest trained passenger", 26),
            T("3  Wristband guides CPR, step by step", 26),
            T("4  Harness advises and delivers the shock", 26),
        ).arrange(DOWN, aligned_edge=LEFT, buff=.3).scale(.74).to_edge(RIGHT, buff=.3).shift(DOWN * 1.45)
        bg = BackgroundRectangle(steps, color=BLACK, fill_opacity=.85, buff=.25)
        for k in range(4): self.add_sound(os.path.join(SFX, 'pro_typing_key_soft.wav'), time_offset=k * .62, gain=-12)
        self.play(FadeIn(bg), LaggedStart(*[FadeIn(s, shift=LEFT * .2) for s in steps], lag_ratio=.35), run_time=2.6)
        self.wait(3)
        self.play(*[FadeOut(m) for m in self.mobjects])

class S4_System(Scene):
    def construct(self):
        title = T("One system on every passenger", 46).to_corner(UL, buff=.6)
        wb = wristband(.8).move_to(LEFT * 4.2 + UP * .6); hn = harness(.9).move_to(UP * .5)
        hub = VGroup(RoundedRectangle(width=1.8, height=1.2, corner_radius=.18, fill_color="#1c1c20", fill_opacity=1, stroke_width=0),
                     *[Rectangle(width=1.2, height=.08, fill_color=CYAN if i == 0 else "#3a3a40", fill_opacity=1, stroke_width=0).shift(UP * (.3 - .2 * i)) for i in range(4)]).move_to(RIGHT * 4.2 + UP * .6)
        labs = VGroup(T("Wristband", 30).next_to(wb, DOWN, buff=.35), T("Harness", 30).next_to(hn, DOWN, buff=.35), T("Ship hub", 30).next_to(hub, DOWN, buff=.35))
        subs = VGroup(T("screen ≥100 mm · vitals · guidance", 22, MUTED).next_to(labs[0], DOWN, buff=.15),
                      T("defibrillation · injectable · ECG", 22, MUTED).next_to(labs[1], DOWN, buff=.15),
                      T("records · ship data · Earth link", 22, MUTED).next_to(labs[2], DOWN, buff=.15))
        sfx(self, 'whoosh_fast_crisp', -12); self.play(FadeIn(title, shift=DOWN * .2))
        for o, l, s in zip([wb, hn, hub], labs, subs):
            self.play(FadeIn(o, scale=.85), FadeIn(l, shift=UP * .1), FadeIn(s), run_time=.9)
        self.wait(1.6)
        self.play(FadeOut(VGroup(wb, hn, hub, labs, subs)), run_time=.8)
        # deck plan with 15 passengers on a LoRa mesh
        deck = RoundedRectangle(width=11, height=4.6, corner_radius=1.2, stroke_color=MUTED, stroke_width=1.5).shift(DOWN * .4)
        rng = np.random.default_rng(7)
        pts = [np.array([x, y, 0]) for x, y in zip(rng.uniform(-4.6, 4.6, 15), rng.uniform(-2.2, 1.4, 15))]
        dots = VGroup(*[Dot(p, radius=.12, color=CREAM) for p in pts])
        hubd = Square(.4, fill_color=CYAN, fill_opacity=1, stroke_width=0).move_to(DOWN * .4)
        edges = VGroup(*[Line(pts[i], pts[j], stroke_width=1.2, color=CYAN, stroke_opacity=.35)
                         for i in range(15) for j in range(i + 1, 15) if np.linalg.norm(pts[i] - pts[j]) < 2.6])
        ml = T("LoRa mesh: each wristband relays for its neighbours", 28, CYAN).to_edge(DOWN, buff=.35)
        sfx(self, 'pro_pop_in_round', -12); self.play(Create(deck), LaggedStart(*[GrowFromCenter(d) for d in dots], lag_ratio=.06), FadeIn(hubd), run_time=1.8)
        self.play(Create(edges, lag_ratio=.02), FadeIn(ml), run_time=1.8)
        # an event: passenger 3 goes down; ripple; nearest trained passenger
        v = 3; vic = dots[v]
        sfx(self, 'notify_m2', -6); self.play(vic.animate.set_color(RED).scale(1.5), run_time=.4)
        for k in range(3):
            self.play(Circle(.2, color=RED, stroke_width=3).move_to(vic).animate.scale(14).set_stroke(opacity=0), run_time=.7, rate_func=rush_from)
        d = [np.linalg.norm(pts[i] - pts[v]) if i != v else 99 for i in range(15)]; h = int(np.argmin(d))
        helper = dots[h]
        tag = T("nearest trained passenger", 24, YEL).next_to(helper, DOWN, buff=.25)
        sfx(self, 'pro_notify_ding_1930', -10); self.play(helper.animate.set_color(YEL).scale(1.4), FadeIn(tag), run_time=.6)
        path = Arrow(helper.get_center(), vic.get_center(), buff=.2, color=YEL, stroke_width=4)
        self.play(GrowArrow(path)); self.play(helper.animate.move_to(vic.get_center() + (helper.get_center() - vic.get_center()) * .25), run_time=1.2, rate_func=smooth)
        info = T("Other passengers get an informative alert", 26, MUTED).next_to(ml, UP, buff=.2)
        self.play(FadeIn(info)); self.wait(2)
        self.play(*[FadeOut(m) for m in self.mobjects])

class S5_Solar(Scene):
    def construct(self):
        title = T("Solar storm: the warning window", 46).to_corner(UL, buff=.6)
        ax = Axes(x_range=[0, 120, 20], y_range=[0, 4, 1], x_length=10, y_length=4.2, tips=False,
                  axis_config=dict(color=MUTED, stroke_width=1.5)).shift(DOWN * .35)
        xn = VGroup(*[T(str(v), 22, MUTED).next_to(ax.c2p(v, 0), DOWN, buff=.15) for v in range(0, 121, 20)])
        ylabels = VGroup(*[T(s, 22, MUTED).next_to(ax.c2p(0, i), LEFT, buff=.2) for i, s in enumerate(["1", "10", "100", "1,000", "10,000"])])
        xl = T("minutes after the flare", 24, MUTED).next_to(ax.x_axis, DOWN, buff=.45)
        yl = T("proton flux (pfu, ≥10 MeV)", 24, MUTED).next_to(ax.y_axis, UP, buff=.25).align_to(ax.y_axis, LEFT)
        sfx(self, 'whoosh_fast_crisp', -12); self.play(FadeIn(title, shift=DOWN * .2), Create(ax), FadeIn(ylabels), FadeIn(xn), FadeIn(xl), FadeIn(yl), run_time=1.6)
        thr = VGroup()
        for i, s in [(1, "S1"), (2, "S2"), (3, "S3")]:
            l = DashedLine(ax.c2p(0, i), ax.c2p(120, i), color=RED, stroke_width=1.2, stroke_opacity=.5)
            thr.add(VGroup(l, T(s, 22, RED).next_to(l, RIGHT, buff=.15)))
        self.play(FadeIn(thr))
        xr = ax.plot(lambda t: 0.3 + 3.2 * np.exp(-((t - 2) / 3) ** 2), x_range=[0, 30], color=YEL, stroke_width=3)
        xrl = T("X-rays arrive with the light", 24, YEL).next_to(ax.c2p(6, 3.4), RIGHT, buff=.1)
        self.play(Create(xr), FadeIn(xrl), run_time=1.4)
        def prot(t): return 0.3 + 3.2 / (1 + np.exp(-(t - 55) / 9))
        pr = ax.plot(prot, x_range=[0, 120], color=CORAL, stroke_width=4)
        prl = T("protons follow", 26, CORAL).move_to(ax.c2p(96, 3.75))
        t_s1 = 55 - 9 * np.log(3.2 / 0.7 - 1)
        win = ax.get_area(ax.plot(lambda t: 4, x_range=[3, t_s1]), x_range=[3, t_s1], color=CYAN, opacity=.18)
        wl = T("time to reach\nthe shelter", 26, CYAN).move_to(ax.c2p((3 + t_s1) / 2 + 4, 1.6))
        sfx(self, 'riser_tension', -10); self.play(Create(pr), FadeIn(prl), run_time=3, rate_func=linear)
        self.play(FadeIn(win), Write(wl), run_time=1.2)
        src = T("Eva reads NOAA GOES and NASA DONKI. Onset is often tens of minutes after the flare.", 22, MUTED).to_edge(DOWN, buff=.2)
        self.play(FadeIn(src)); self.wait(1.5)
        # crew count to shelter
        cnt = ValueTracker(0)
        ct = always_redraw(lambda: T(f"in shelter  {int(cnt.get_value())}/15", 34, CYAN).to_corner(UR, buff=.6).shift(DOWN * .9))
        sfx(self, 'notify_m2', -8); self.add(ct); self.play(cnt.animate.set_value(15), run_time=2.5, rate_func=smooth); self.wait(2)
        self.play(*[FadeOut(m) for m in self.mobjects])

class S6_Records(Scene):
    def construct(self):
        title = T("Everything recorded, even without Earth", 46).to_corner(UL, buff=.6)
        ship = T("Ship", 30).move_to(LEFT * 5 + UP * .3); earth = T("Earth", 30).move_to(RIGHT * 5 + UP * .3)
        line = Arrow(LEFT * 4.2 + UP * .3, RIGHT * 4.2 + UP * .3, buff=0, color=MUTED, stroke_width=2)
        sfx(self, 'whoosh_fast_crisp', -12); self.play(FadeIn(title, shift=DOWN * .2), FadeIn(ship), FadeIn(earth), GrowArrow(line))
        logs = ["14:02  rhythm alert", "14:02  helper assigned", "14:04  shock delivered", "14:06  pulse back"]
        items = VGroup(*[T(s, 24, CREAM) for s in logs]).arrange(DOWN, aligned_edge=LEFT, buff=.18).next_to(ship, DOWN, buff=.6).align_to(ship, LEFT)
        self.play(LaggedStart(*[FadeIn(i, shift=RIGHT * .2) for i in items], lag_ratio=.3), run_time=1.6)
        for i in items:
            sfx(self, 'pro_tap_glass', -14); p = Dot(radius=.08, color=CYAN).move_to(line.get_start())
            self.play(MoveAlongPath(p, line), run_time=.7, rate_func=linear); self.remove(p)
        ack = T("ACK · received on Earth, minutes later", 28, CYAN).next_to(earth, DOWN, buff=.6).align_to(earth, RIGHT)
        sfx(self, 'pro_notify_ding_1930', -10); self.play(FadeIn(ack, shift=LEFT * .2))
        n = T("Queued and signed on board first, sent when the link allows.", 26, MUTED).to_edge(DOWN, buff=.6)
        self.play(FadeIn(n)); self.wait(2.2)
        self.play(*[FadeOut(m) for m in self.mobjects])

class S7_Close(Scene):
    def construct(self):
        a = T("Eva guides, records and warns.", 52)
        b = T("Shocks and injections belong to validated devices,", 32, MUTED).next_to(a, DOWN, buff=.45)
        c = T("never to a chat model.", 32, MUTED).next_to(b, DOWN, buff=.15)
        g = VGroup(a, b, c).shift(UP * .6 + LEFT * 1.6)
        self.play(Write(a), run_time=1.6); self.play(FadeIn(b, shift=UP * .1), FadeIn(c, shift=UP * .1)); self.wait(2)
        self.play(FadeOut(g, shift=UP * .3))
        wm = T("Eva", 120); by = T("by WearMeds", 40, MUTED).next_to(wm, DOWN, buff=.2)
        team = T("Team Sanjo-X · NASA Space Apps Challenge 2026 · Rosario", 26, MUTED).next_to(by, DOWN, buff=.8)
        G = VGroup(wm, by, team).shift(LEFT * 1.8)
        sfx(self, 'logo_sting_soft_bloom', -6); self.play(FadeIn(wm, scale=.9), FadeIn(by), run_time=1.2); self.play(FadeIn(team)); self.wait(3.5)
        self.play(FadeOut(G), run_time=1)
