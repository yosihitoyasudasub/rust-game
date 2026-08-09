//! Ownership - a puzzle prototype where Rust's move semantics are the rules of the world.
//!
//! A value is a physical crate. Handing it to another scope is a move: you no longer have it.
//! Whether the crate survives a scope is written on the door - its function signature.
//! The panel on the right is the same program as source, written line by line as you play.
//!
//! UI language toggles with L. Prose uses a CJK face when set to Japanese; anything that is
//! Rust source stays in the code face, because the source itself is not translated.

use macroquad::prelude::*;

// Everything is authored in this design space and scaled to whatever the window actually is,
// so the framebuffer is native resolution on a HiDPI monitor instead of an upscaled blur.
const DW: f32 = 1600.0;
const DH: f32 = 900.0;

const RW: f32 = 228.0;
const RH: f32 = 96.0;

const PANEL_X: f32 = 1150.0;
const PANEL_W: f32 = 420.0;
const CODE_H: f32 = 24.0;
const COMMENT_H: f32 = 19.0;

// ------------------------------------------------------------------ palette

const BG: Color = Color::new(0.059, 0.071, 0.098, 1.0);
const PANEL: Color = Color::new(0.082, 0.102, 0.141, 1.0);
const ROOM: Color = Color::new(0.106, 0.133, 0.188, 1.0);
const LINE: Color = Color::new(0.184, 0.227, 0.302, 1.0);
const INK: Color = Color::new(0.753, 0.792, 0.961, 1.0);
const DIM: Color = Color::new(0.420, 0.459, 0.600, 1.0);
const FAINT: Color = Color::new(0.267, 0.302, 0.404, 1.0);
const BLUE: Color = Color::new(0.478, 0.635, 0.969, 1.0);
const GREEN: Color = Color::new(0.620, 0.808, 0.416, 1.0);
const AMBER: Color = Color::new(0.878, 0.686, 0.408, 1.0);
const RED: Color = Color::new(0.969, 0.463, 0.557, 1.0);
const COMMENT: Color = Color::new(0.365, 0.502, 0.443, 1.0);

fn alpha(c: Color, a: f32) -> Color {
    Color { a, ..c }
}

// ------------------------------------------------------------------- locale

#[derive(Clone, Copy, PartialEq)]
enum Lang {
    En,
    Ja,
}

/// Which typeface a run of text belongs to. `Sans` follows the UI language;
/// `Mono` is always the code face, since Rust source is never translated.
#[derive(Clone, Copy, PartialEq)]
enum Face {
    Sans,
    Mono,
}

/// An English/Japanese pair. Everything player-facing goes through one of these.
struct S(&'static str, &'static str);

impl S {
    fn get(&self, l: Lang) -> &'static str {
        match l {
            Lang::En => self.0,
            Lang::Ja => self.1,
        }
    }
}

// -------------------------------------------------------------------- level

#[derive(Clone)]
struct Room {
    name: String,
    sig: String,
    binding: String,
    /// Does the value leave this scope alive? (i.e. does the signature return it)
    returns: bool,
    is_goal: bool,
    pos: Vec2,
}

impl Room {
    fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, RW, RH)
    }
    fn center(&self) -> Vec2 {
        self.pos + vec2(RW / 2.0, RH / 2.0)
    }
    fn slot(&self) -> Rect {
        Rect::new(self.pos.x + RW / 2.0 - 44.0, self.pos.y + RH - 38.0, 88.0, 28.0)
    }
    /// The source line that defines this scope, derived from the signature so the
    /// board and the code can never disagree.
    fn def_line(&self) -> String {
        if self.is_goal || !self.returns {
            format!("{} {{ }}", self.sig)
        } else {
            format!("{} {{ s }}", self.sig)
        }
    }
    /// The line main writes when it hands the crate over.
    fn call_line(&self) -> String {
        if self.returns && !self.is_goal {
            format!("    let data = {}(data);", self.name)
        } else {
            format!("    {}(data);", self.name)
        }
    }
}

fn room(name: &str, sig: &str, binding: &str, returns: bool, x: f32, y: f32) -> Room {
    Room {
        name: name.into(),
        sig: sig.into(),
        binding: binding.into(),
        returns,
        is_goal: false,
        pos: vec2(x, y),
    }
}

fn goal(name: &str, sig: &str, binding: &str, x: f32, y: f32) -> Room {
    Room { is_goal: true, ..room(name, sig, binding, true, x, y) }
}

struct Level {
    title: S,
    /// What problem the language is solving by having this rule at all. Without
    /// this the levels teach a rule the player has no reason to want.
    stakes: [S; 2],
    /// The Rust idea this level exists to teach.
    concept: S,
    /// Why that idea matters, shown before the player touches anything.
    why: [S; 2],
    /// What winning looks like, kept on screen for the whole level.
    goal: S,
    /// How to operate the board.
    hint: S,
    /// Restated on the clear screen, once the player has actually done it.
    takeaway: S,
    rooms: Vec<Room>,
    /// Directed: you may hand the crate from .0 to .1
    edges: Vec<(usize, usize)>,
}

fn levels() -> Vec<Level> {
    vec![
        Level {
            title: S("1 / 3   move", "1 / 3   ムーブ"),
            stakes: [
                S(
                    "In C, freeing memory is your job - double frees and use-after-free follow.",
                    "C言語では解放の責任が人間側にあり、二重解放や解放後の参照がバグになる。",
                ),
                S(
                    "Rust allows exactly one owner, which makes both impossible at compile time.",
                    "Rustは持ち主を常に1人に限ることで、それをコンパイル時に不可能にした。",
                ),
            ],
            concept: S("Move", "ムーブ"),
            why: [
                S(
                    "In Rust, handing a value to a function moves ownership with it.",
                    "Rustでは、値を関数に渡すと所有権ごと移動する。",
                ),
                S(
                    "It is not a copy. The caller does not have it any more.",
                    "コピーではない。渡した側には、もう無い。",
                ),
            ],
            goal: S(
                "Get the crate to `finish`.",
                "箱を `finish` まで届ける。",
            ),
            hint: S(
                "Drag the crate along an arrow to hand it to the next scope.",
                "矢印をたどって、箱を次のスコープに手渡す。",
            ),
            takeaway: S(
                "A variable you handed over cannot be used again. That is error E0382.",
                "渡した変数は二度と使えない。これがエラー E0382 の正体。",
            ),
            rooms: vec![
                room("main", "fn main()", "data", true, 110.0, 400.0),
                room("takes", "fn takes(s: String) -> String", "s", true, 450.0, 400.0),
                goal("finish", "fn finish(s: String)", "s", 790.0, 400.0),
            ],
            edges: vec![(0, 1), (1, 2)],
        },
        Level {
            title: S("2 / 3   read the signature", "2 / 3   シグネチャを読む"),
            stakes: [
                S(
                    "You want to call someone else's function without reading its body.",
                    "他人が書いた関数は、中身を読まずに安全に呼べるのが望ましい。",
                ),
                S(
                    "Rust puts ownership in the signature, so the declaration alone tells you.",
                    "Rustは所有権の行方をシグネチャに書かせる。だから宣言だけで判断できる。",
                ),
            ],
            concept: S("The signature is the contract", "シグネチャは契約書"),
            why: [
                S(
                    "Whether a function gives the value back is visible in its signature alone.",
                    "値を返してくれるかどうかは、シグネチャだけを見れば分かる。",
                ),
                S(
                    "You never need to read the body to know where ownership ends up.",
                    "所有権の行き先を知るのに、関数の中身を読む必要はない。",
                ),
            ],
            goal: S(
                "Reach `deliver`, avoiding any scope that will not return the value.",
                "値を返さないスコープを避けて `deliver` まで届ける。",
            ),
            hint: S(
                "A scope that does not return the value drops it when it ends.",
                "値を返さないスコープは、終わりでその値を破棄する。",
            ),
            takeaway: S(
                "Look for `-> String`. That one glance tells you if the value survives.",
                "`-> String` があるかを見る。それだけで値が生き残るか分かる。",
            ),
            rooms: vec![
                room("main", "fn main()", "data", true, 110.0, 400.0),
                room("log", "fn log(s: String)", "s", false, 450.0, 215.0),
                room("tag", "fn tag(s: String) -> String", "s", true, 450.0, 585.0),
                goal("deliver", "fn deliver(s: String)", "s", 790.0, 400.0),
            ],
            edges: vec![(0, 1), (0, 2), (1, 3), (2, 3)],
        },
        Level {
            title: S("3 / 3   one path", "3 / 3   一本道"),
            stakes: [
                S(
                    "Most real memory bugs come from losing track of who owns a value right now.",
                    "実際のバグの多くは「今この値を誰が持っているか」を見失うことから起きる。",
                ),
                S(
                    "Rust keeps that thread single, so once it compiles you cannot have lost it.",
                    "Rustはその線を1本に保つ。だからコンパイルが通れば、見失いようがない。",
                ),
            ],
            concept: S("Follow ownership end to end", "所有権を最後まで追う"),
            why: [
                S(
                    "In real code a value passes through many functions in a row.",
                    "実際のコードでは、値はいくつもの関数を次々に通り抜けていく。",
                ),
                S(
                    "Chain the signatures together and the whole route is decided.",
                    "シグネチャをつなげて読めば、通れる経路はそれだけで決まる。",
                ),
            ],
            goal: S(
                "Route the crate through two scopes to `ship`.",
                "2つのスコープを経由して `ship` まで届ける。",
            ),
            hint: S(
                "Only one route keeps the value alive all the way to the end.",
                "最後まで値が生き残るルートは、ただ1つ。",
            ),
            takeaway: S(
                "Ownership is a single thread: no forks, no copies. Reading Rust is following it.",
                "所有権は1本の線。分岐も複製もしない。Rustを読むとは、この線を追うこと。",
            ),
            rooms: vec![
                room("main", "fn main()", "data", true, 30.0, 380.0),
                room("peek", "fn peek(s: String)", "s", false, 315.0, 200.0),
                room("wrap", "fn wrap(s: String) -> String", "s", true, 315.0, 560.0),
                room("drain", "fn drain(s: String)", "s", false, 600.0, 200.0),
                room("seal", "fn seal(s: String) -> String", "s", true, 600.0, 560.0),
                goal("ship", "fn ship(s: String)", "s", 885.0, 380.0),
            ],
            edges: vec![(0, 1), (0, 2), (1, 3), (1, 4), (2, 3), (2, 4), (3, 5), (4, 5)],
        },
    ]
}

// -------------------------------------------------------------------- state

enum Status {
    /// Why this level exists, shown before the board is live.
    Briefing,
    Playing,
    /// The crate landed in a scope that will not give it back. t counts up.
    Dropping(f32),
    Failed,
    Cleared(f32),
}

/// Diagnostics are stored as data, not as a formatted string, so the language
/// toggle re-renders them instead of leaving a stale translation on screen.
enum Diag {
    NoEdge(usize, usize),
    Moved(usize),
}

/// One rendered line of the source panel, tied back to the room it describes.
struct CodeLine {
    text: String,
    /// Plain-language gloss shown under the line. Deliberately paraphrases the
    /// syntax rather than announcing the outcome, so it scaffolds reading the
    /// signature instead of solving the level for the player.
    comment: Option<S>,
    room: Option<usize>,
    kind: LineKind,
}

#[derive(PartialEq)]
enum LineKind {
    Def,
    Body,
    /// The line that would be written if the player dropped the crate right now.
    Preview,
    Structure,
    Dead,
}

/// The gloss for a call line. Until the crate is actually dropped, a call into a
/// non-returning scope reads the same as any other hand-off - saying "dropped"
/// early would answer the puzzle before the player commits.
fn call_comment(r: &Room, dead: bool) -> S {
    if dead {
        S("moved in; never returned - dropped here", "ムーブ。返さない関数 → ここで破棄")
    } else if r.returns && !r.is_goal {
        S("move in; rebind what comes back", "ムーブ。返り値を data に再束縛")
    } else {
        S("move in; let it go", "ムーブ。ここで手放す")
    }
}

struct Game {
    levels: Vec<Level>,
    idx: usize,
    holder: usize,
    /// Scopes the value has already been moved out of.
    ghosts: Vec<bool>,
    /// Rooms visited, in order - this is what the source panel replays.
    path: Vec<usize>,
    dragging: bool,
    status: Status,
    err: Option<(Diag, f32)>,
    hover_room: Option<usize>,
    /// Room highlighted because the pointer is over its source line.
    hover_from_code: Option<usize>,
    t: f32,
}

impl Game {
    fn new() -> Self {
        let levels = levels();
        let n = levels[0].rooms.len();
        Game {
            levels,
            idx: 0,
            holder: 0,
            ghosts: vec![false; n],
            path: vec![0],
            dragging: false,
            status: Status::Briefing,
            err: None,
            hover_room: None,
            hover_from_code: None,
            t: 0.0,
        }
    }

    fn lvl(&self) -> &Level {
        &self.levels[self.idx]
    }

    fn reset(&mut self) {
        let n = self.levels[self.idx].rooms.len();
        self.holder = 0;
        self.ghosts = vec![false; n];
        self.path = vec![0];
        self.dragging = false;
        self.status = Status::Playing;
        self.err = None;
    }

    fn next_level(&mut self) {
        if self.idx + 1 < self.levels.len() {
            self.idx += 1;
            self.reset();
            // a new theme deserves its briefing; a retry of the same one does not
            self.status = Status::Briefing;
        }
    }

    fn try_move(&mut self, to: usize) {
        let from = self.holder;
        if to == from {
            return;
        }
        if !self.lvl().edges.contains(&(from, to)) {
            self.err = Some((Diag::NoEdge(from, to), 3.0));
            return;
        }

        self.ghosts[from] = true;
        self.holder = to;
        self.path.push(to);

        let r = &self.lvl().rooms[to];
        if r.is_goal {
            self.status = Status::Cleared(0.0);
        } else if !r.returns {
            self.status = Status::Dropping(0.0);
        }
    }

    /// Build the source panel from the current run. `preview` is the room the
    /// player is currently dragging the crate over, if the move would be legal.
    fn source(&self, preview: Option<usize>) -> Vec<CodeLine> {
        let mut out = Vec::new();
        let rooms = &self.lvl().rooms;

        for (i, r) in rooms.iter().enumerate().skip(1) {
            // a mechanical restatement of the signature - the reading skill the
            // player is meant to acquire, not the conclusion they must draw
            let c = if r.returns && !r.is_goal {
                S("takes a String, gives a String back", "String を受け取り、String を返す")
            } else {
                S("takes a String, gives nothing back", "String を受け取る。返さない")
            };
            out.push(CodeLine {
                text: r.def_line(),
                comment: Some(c),
                room: Some(i),
                kind: LineKind::Def,
            });
        }
        out.push(CodeLine { text: String::new(), comment: None, room: None, kind: LineKind::Structure });
        out.push(CodeLine {
            text: "fn main() {".into(),
            comment: None,
            room: Some(0),
            kind: LineKind::Structure,
        });
        out.push(CodeLine {
            text: "    let data = String::from(\"payload\");".into(),
            comment: Some(S("create the value; `data` owns it", "値を作る。所有者は data")),
            room: Some(0),
            kind: LineKind::Body,
        });

        let dead = matches!(self.status, Status::Dropping(_) | Status::Failed);
        for (n, &i) in self.path.iter().enumerate().skip(1) {
            let is_dead = dead && n == self.path.len() - 1;
            out.push(CodeLine {
                text: rooms[i].call_line(),
                comment: Some(call_comment(&rooms[i], is_dead)),
                room: Some(i),
                kind: if is_dead { LineKind::Dead } else { LineKind::Body },
            });
        }

        if let Some(p) = preview {
            out.push(CodeLine {
                text: rooms[p].call_line(),
                comment: Some(call_comment(&rooms[p], false)),
                room: Some(p),
                kind: LineKind::Preview,
            });
        }

        out.push(CodeLine {
            text: "}".into(),
            comment: None,
            room: Some(0),
            kind: LineKind::Structure,
        });
        out
    }
}

// ----------------------------------------------------------------- ui / draw

struct Ui {
    s: f32,
    ox: f32,
    oy: f32,
    sans: Option<Font>,
    mono: Option<Font>,
    lang: Lang,
    comments: bool,
}

impl Ui {
    fn sync(&mut self) {
        let (w, h) = (screen_width(), screen_height());
        self.s = (w / DW).min(h / DH);
        self.ox = (w - DW * self.s) / 2.0;
        self.oy = (h - DH * self.s) / 2.0;
    }
    fn px(&self, x: f32) -> f32 {
        self.ox + x * self.s
    }
    fn py(&self, y: f32) -> f32 {
        self.oy + y * self.s
    }
    /// Screen pointer -> design space, so hit tests stay in authored coordinates.
    fn mouse(&self) -> Vec2 {
        let (mx, my) = mouse_position();
        vec2((mx - self.ox) / self.s, (my - self.oy) / self.s)
    }
    fn tr(&self, s: &S) -> &'static str {
        s.get(self.lang)
    }
    /// Prose sitting inside the code panel: the code face reads better in English,
    /// but Consolas has no kana, so Japanese has to fall back to the CJK face.
    fn gloss_face(&self) -> Face {
        match self.lang {
            Lang::Ja => Face::Sans,
            Lang::En => Face::Mono,
        }
    }
    fn font(&self, f: Face) -> Option<&Font> {
        match f {
            Face::Mono => self.mono.as_ref(),
            Face::Sans => self.sans.as_ref(),
        }
    }
    /// The language toggle only makes sense if the face that has kana loaded.
    fn bilingual(&self) -> bool {
        self.sans.is_some()
    }
    fn text(&self, txt: &str, x: f32, y: f32, size: f32, col: Color, f: Face) {
        if txt.is_empty() {
            return;
        }
        draw_text_ex(
            txt,
            self.px(x),
            self.py(y),
            TextParams {
                font: self.font(f),
                font_size: (size * self.s).round().max(1.0) as u16,
                font_scale: 1.0,
                color: col,
                ..Default::default()
            },
        );
    }
    /// Width in design units.
    fn width(&self, txt: &str, size: f32, f: Face) -> f32 {
        measure_text(txt, self.font(f), (size * self.s).round().max(1.0) as u16, 1.0).width / self.s
    }
    fn text_center(&self, txt: &str, cx: f32, y: f32, size: f32, col: Color, f: Face) {
        self.text(txt, cx - self.width(txt, size, f) / 2.0, y, size, col, f);
    }
    fn text_right(&self, txt: &str, rx: f32, y: f32, size: f32, col: Color, f: Face) {
        self.text(txt, rx - self.width(txt, size, f), y, size, col, f);
    }

    fn rect(&self, x: f32, y: f32, w: f32, h: f32, col: Color) {
        draw_rectangle(self.px(x), self.py(y), w * self.s, h * self.s, col);
    }
    fn round_rect(&self, x: f32, y: f32, w: f32, h: f32, r: f32, col: Color) {
        let (x, y, w, h, r) = (self.px(x), self.py(y), w * self.s, h * self.s, r * self.s);
        let r = r.min(w / 2.0).min(h / 2.0);
        draw_rectangle(x + r, y, w - 2.0 * r, h, col);
        draw_rectangle(x, y + r, r, h - 2.0 * r, col);
        draw_rectangle(x + w - r, y + r, r, h - 2.0 * r, col);
        for (cx, cy) in [
            (x + r, y + r),
            (x + w - r, y + r),
            (x + r, y + h - r),
            (x + w - r, y + h - r),
        ] {
            draw_circle(cx, cy, r, col);
        }
    }
    /// Outline drawn as a slightly inflated rounded rect behind the fill - cheap,
    /// and the corners stay smooth under MSAA.
    fn round_frame(&self, x: f32, y: f32, w: f32, h: f32, r: f32, fill: Color, edge: Color, t: f32) {
        self.round_rect(x - t, y - t, w + 2.0 * t, h + 2.0 * t, r + t, edge);
        self.round_rect(x, y, w, h, r, fill);
    }
    fn line(&self, a: Vec2, b: Vec2, thick: f32, col: Color) {
        draw_line(self.px(a.x), self.py(a.y), self.px(b.x), self.py(b.y), thick * self.s, col);
        // round the caps so thick corridors do not end in a hard chisel
        draw_circle(self.px(a.x), self.py(a.y), thick * self.s / 2.0, col);
        draw_circle(self.px(b.x), self.py(b.y), thick * self.s / 2.0, col);
    }
    fn tri(&self, a: Vec2, b: Vec2, c: Vec2, col: Color) {
        draw_triangle(
            vec2(self.px(a.x), self.py(a.y)),
            vec2(self.px(b.x), self.py(b.y)),
            vec2(self.px(c.x), self.py(c.y)),
            col,
        );
    }
}

/// Where a ray from the centre of an axis-aligned box leaves it.
fn exit_point(c: Vec2, hw: f32, hh: f32, dir: Vec2) -> Vec2 {
    let tx = if dir.x.abs() > 1e-4 { hw / dir.x.abs() } else { f32::MAX };
    let ty = if dir.y.abs() > 1e-4 { hh / dir.y.abs() } else { f32::MAX };
    c + dir * tx.min(ty)
}

fn draw_arrow(ui: &Ui, a: Vec2, b: Vec2, col: Color, thick: f32) {
    let dir = (b - a).normalize();
    let a = exit_point(a, RW / 2.0, RH / 2.0, dir) + dir * 10.0;
    let b = exit_point(b, RW / 2.0, RH / 2.0, -dir) - dir * 10.0;
    if (b - a).dot(dir) <= 4.0 {
        return;
    }
    let head = b - dir * 13.0;
    ui.line(a, head, thick, col);
    let n = vec2(-dir.y, dir.x);
    ui.tri(b, head + n * 8.0, head - n * 8.0, col);
}

fn draw_crate(ui: &Ui, r: Rect, label: &str, scale: f32, a: f32, glow: f32) {
    let cx = r.x + r.w / 2.0;
    let cy = r.y + r.h / 2.0;
    let w = r.w * scale;
    let h = r.h * scale;
    let (x, y) = (cx - w / 2.0, cy - h / 2.0);

    if glow > 0.0 {
        ui.round_rect(x - 6.0, y - 6.0, w + 12.0, h + 12.0, 12.0, alpha(AMBER, 0.14 * glow * a));
    }
    ui.round_rect(x + 1.0, y + 3.0, w, h, 7.0, alpha(BLACK, 0.35 * a));
    ui.round_rect(x, y, w, h, 7.0, alpha(AMBER, a));
    // a lighter band across the top reads as a lid without needing a sprite
    ui.round_rect(
        x + 3.0,
        y + 3.0,
        w - 6.0,
        h * 0.36,
        4.0,
        alpha(Color::new(1.0, 0.85, 0.60, 1.0), 0.35 * a),
    );
    if a > 0.4 {
        ui.text_center(
            label,
            cx,
            cy + h * 0.17,
            17.0,
            alpha(Color::new(0.10, 0.08, 0.04, 1.0), a),
            Face::Mono,
        );
    }
}

fn draw_room(ui: &Ui, g: &Game, i: usize, reachable: bool, lit: bool) {
    let r = &g.lvl().rooms[i];
    let rc = r.rect();

    let edge = if r.is_goal {
        GREEN
    } else if reachable || lit {
        BLUE
    } else {
        LINE
    };
    let t = if reachable || lit { 2.2 } else { 1.2 };

    ui.round_rect(rc.x + 2.0, rc.y + 6.0, rc.w, rc.h, 12.0, alpha(BLACK, 0.35));
    if lit {
        ui.round_rect(rc.x - 7.0, rc.y - 7.0, rc.w + 14.0, rc.h + 14.0, 18.0, alpha(BLUE, 0.10));
    }
    ui.round_frame(rc.x, rc.y, rc.w, rc.h, 11.0, ROOM, edge, t);

    // the scope name and its signature are both Rust, so both stay in the code face
    ui.text_center(&r.name, rc.x + rc.w / 2.0, rc.y + 27.0, 20.0, INK, Face::Mono);
    // JetBrains Mono is wider than the system faces this was first sized against;
    // the longest signature has to clear the box, so it is shrunk to fit.
    ui.text_center(&r.sig, rc.x + rc.w / 2.0, rc.y + 48.0, 11.5, DIM, Face::Mono);

    let slot = r.slot();
    ui.round_frame(slot.x, slot.y, slot.w, slot.h, 6.0, alpha(BLACK, 0.18), alpha(LINE, 0.9), 1.0);

    if g.ghosts[i] && g.holder != i {
        let c = alpha(FAINT, 0.85);
        ui.text_center(&r.binding, slot.x + slot.w / 2.0, slot.y + 19.0, 16.0, c, Face::Mono);
        let w = ui.width(&r.binding, 16.0, Face::Mono);
        let cx = slot.x + slot.w / 2.0;
        ui.line(vec2(cx - w / 2.0, slot.y + 14.0), vec2(cx + w / 2.0, slot.y + 14.0), 1.2, c);
    }
}

// --------------------------------------------------------------------- panel

/// The compiler-style diagnostic, as (text, face, colour) rows.
fn diag_rows(ui: &Ui, g: &Game, d: &Diag) -> Vec<(String, Face, Color)> {
    let rooms = &g.lvl().rooms;
    match *d {
        Diag::NoEdge(a, b) => {
            let (a, b) = (&rooms[a].name, &rooms[b].name);
            vec![
                (
                    match ui.lang {
                        Lang::En => format!("no call from `{a}` to `{b}`"),
                        Lang::Ja => format!("`{a}` から `{b}` を呼ぶ経路はない"),
                    },
                    Face::Sans,
                    RED,
                ),
                (
                    ui.tr(&S("follow an arrow", "矢印をたどること")).into(),
                    Face::Sans,
                    DIM,
                ),
            ]
        }
        Diag::Moved(i) => {
            let b = &rooms[i].binding;
            let culprit = g.path.get(1).map(|&j| rooms[j].call_line()).unwrap_or_default();
            vec![
                (format!("error[E0382]: use of moved value: `{b}`"), Face::Mono, RED),
                (culprit.trim().to_string(), Face::Mono, DIM),
                (
                    ui.tr(&S(
                        "moved out here - it cannot be used again",
                        "ここでムーブされた。この値はもう使えない",
                    ))
                    .into(),
                    Face::Sans,
                    DIM,
                ),
            ]
        }
    }
}

/// Draws the source panel and returns the room whose line is under the pointer.
fn draw_panel(ui: &Ui, g: &Game, lines: &[CodeLine], m: Vec2) -> Option<usize> {
    ui.round_frame(PANEL_X, 100.0, PANEL_W, 720.0, 12.0, PANEL, LINE, 1.0);
    ui.text("main.rs", PANEL_X + 20.0, 130.0, 14.0, FAINT, Face::Mono);
    ui.line(
        vec2(PANEL_X + 16.0, 142.0),
        vec2(PANEL_X + PANEL_W - 16.0, 142.0),
        1.0,
        alpha(LINE, 0.8),
    );

    let x0 = PANEL_X + 20.0;
    let mut hovered = None;
    let mut y = 172.0;

    let live_idx = lines
        .iter()
        .rposition(|x| matches!(x.kind, LineKind::Body | LineKind::Dead));

    for (n, l) in lines.iter().enumerate() {
        let row = Rect::new(PANEL_X + 6.0, y - 17.0, PANEL_W - 12.0, CODE_H);
        let over = row.contains(m) && !l.text.is_empty();
        if over {
            hovered = l.room;
        }
        let live = live_idx == Some(n);

        if live || over {
            let c = match l.kind {
                LineKind::Dead => alpha(RED, 0.13),
                _ if live => alpha(BLUE, 0.11),
                _ => alpha(BLUE, 0.06),
            };
            ui.round_rect(row.x, row.y, row.w, row.h, 5.0, c);
        }
        if live {
            let bar = if l.kind == LineKind::Dead { RED } else { BLUE };
            ui.round_rect(PANEL_X + 6.0, row.y + 3.0, 3.0, CODE_H - 6.0, 1.5, bar);
        }

        let col = match l.kind {
            LineKind::Def => DIM,
            LineKind::Structure => FAINT,
            LineKind::Preview => alpha(AMBER, 0.55),
            LineKind::Dead => RED,
            LineKind::Body => INK,
        };
        ui.text(&l.text, x0, y, 15.0, col, Face::Mono);
        y += CODE_H;

        if ui.comments {
            if let Some(ref c) = l.comment {
                let tint = match l.kind {
                    LineKind::Dead => alpha(RED, 0.65),
                    LineKind::Preview => alpha(AMBER, 0.4),
                    _ => COMMENT,
                };
                ui.text(
                    &format!("// {}", ui.tr(c)),
                    x0 + 12.0,
                    y + 1.0,
                    13.0,
                    tint,
                    ui.gloss_face(),
                );
                y += COMMENT_H;
            }
        }
    }

    // rustc-style diagnostic, anchored under the code
    if let Some((ref d, t)) = g.err {
        let a = (t / 0.5).min(1.0);
        let rows = diag_rows(ui, g, d);
        let y = y + 34.0;
        let h = rows.len() as f32 * 21.0 + 20.0;
        ui.round_frame(
            PANEL_X + 12.0,
            y - 21.0,
            PANEL_W - 24.0,
            h,
            8.0,
            alpha(RED, 0.07),
            alpha(RED, 0.35 * a),
            1.0,
        );
        for (n, (txt, face, col)) in rows.iter().enumerate() {
            ui.text(txt, PANEL_X + 26.0, y + n as f32 * 21.0, 14.0, alpha(*col, a), *face);
        }
    }

    hovered
}

// ------------------------------------------------------------------ briefing

/// Answers "why am I being asked to do this" before the level starts, and again
/// on demand. Covers the board only, so the source panel stays readable.
fn draw_briefing(ui: &Ui, l: &Level) {
    let bw = PANEL_X - 20.0;
    ui.rect(0.0, 0.0, bw, DH, alpha(BG, 0.94));

    let (x, y, w, h) = (105.0, 165.0, 920.0, 550.0);
    ui.round_rect(x + 2.0, y + 8.0, w, h, 16.0, alpha(BLACK, 0.4));
    ui.round_frame(x, y, w, h, 15.0, PANEL, LINE, 1.0);

    let px = x + 48.0;
    let rule = |ui: &Ui, yy: f32| {
        ui.line(vec2(px, yy), vec2(x + w - 48.0, yy), 1.0, alpha(LINE, 0.8));
    };

    // why the language works this way at all - the frame everything else hangs on
    ui.round_rect(px - 18.0, y + 34.0, 3.0, 62.0, 1.5, alpha(AMBER, 0.7));
    ui.text(
        ui.tr(&S("WHY RUST IS LIKE THIS", "なぜ Rust はこうなっているのか")),
        px,
        y + 48.0,
        13.0,
        FAINT,
        Face::Sans,
    );
    ui.text(ui.tr(&l.stakes[0]), px, y + 76.0, 17.0, INK, Face::Sans);
    ui.text(ui.tr(&l.stakes[1]), px, y + 102.0, 17.0, INK, Face::Sans);
    rule(ui, y + 132.0);

    // the rule this level drills
    ui.text(ui.tr(&S("THEME", "テーマ")), px, y + 170.0, 13.0, FAINT, Face::Sans);
    ui.text(ui.tr(&l.concept), px, y + 210.0, 30.0, INK, Face::Sans);
    ui.text(ui.tr(&l.why[0]), px, y + 246.0, 17.0, DIM, Face::Sans);
    ui.text(ui.tr(&l.why[1]), px, y + 272.0, 17.0, DIM, Face::Sans);
    rule(ui, y + 302.0);

    // what to actually do
    ui.text(ui.tr(&S("GOAL", "目的")), px, y + 340.0, 13.0, FAINT, Face::Sans);
    ui.text(ui.tr(&l.goal), px, y + 376.0, 21.0, GREEN, Face::Sans);
    ui.text(ui.tr(&l.hint), px, y + 408.0, 16.0, DIM, Face::Sans);

    ui.text_center(
        ui.tr(&S(
            "space to begin        H shows this again",
            "Space で開始        H でいつでも再表示",
        )),
        x + w / 2.0,
        y + h - 34.0,
        16.0,
        FAINT,
        Face::Sans,
    );
}

// ---------------------------------------------------------------------- main

fn window_conf() -> Conf {
    Conf {
        window_title: "Ownership".into(),
        window_width: 1600,
        window_height: 900,
        // render at native device pixels instead of letting the OS upscale a small buffer
        high_dpi: true,
        sample_count: 4,
        window_resizable: true,
        ..Default::default()
    }
}

// Fonts are compiled in rather than read from the OS. Reading system paths worked on
// Windows but hung the whole game on wasm: `C:/...` parses as a URL scheme, the XHR
// never calls back, and the await before the main loop never resolves - so not a single
// frame was ever drawn. Embedding also makes every platform render identically.
// Subsetted to just the characters this game draws by tools/subset-font, which turns
// 4.8 MB of font into 130 KB. Re-run that tool after adding or changing any text.
const FONT_SANS: &[u8] = include_bytes!("../assets/jp-subset.ttf");
const FONT_MONO: &[u8] = include_bytes!("../assets/mono-subset.ttf");

fn embedded_font(kind: &str, bytes: &[u8]) -> Option<Font> {
    match load_ttf_font_from_bytes(bytes) {
        Ok(mut f) => {
            f.set_filter(FilterMode::Linear);
            Some(f)
        }
        Err(e) => {
            println!("font {kind}: failed to parse ({e})");
            None
        }
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    // Noto Sans JP carries Latin as well as kana and kanji, so one face serves both
    // languages; JetBrains Mono is Latin-only and is used strictly for Rust source.
    let sans = embedded_font("sans", FONT_SANS);
    let mono = embedded_font("mono", FONT_MONO);
    println!("fonts: sans={} mono={}", sans.is_some(), mono.is_some());

    let mut ui = Ui {
        s: 1.0,
        ox: 0.0,
        oy: 0.0,
        lang: if sans.is_some() { Lang::Ja } else { Lang::En },
        sans,
        mono,
        comments: true,
    };

    let mut g = Game::new();

    loop {
        let dt = get_frame_time();
        g.t += dt;
        ui.sync();
        let m = ui.mouse();

        // ------------------------------------------------------------ input
        if is_key_pressed(KeyCode::R) {
            g.reset();
        }
        if is_key_pressed(KeyCode::N) && matches!(g.status, Status::Cleared(_)) {
            g.next_level();
        }
        if is_key_pressed(KeyCode::L) && ui.bilingual() {
            ui.lang = if ui.lang == Lang::Ja { Lang::En } else { Lang::Ja };
        }
        if is_key_pressed(KeyCode::C) {
            ui.comments = !ui.comments;
        }

        if let Some((_, ref mut t)) = g.err {
            *t -= dt;
            if *t <= 0.0 {
                g.err = None;
            }
        }
        if let Status::Dropping(ref mut t) = g.status {
            *t += dt;
            if *t >= 0.9 {
                g.status = Status::Failed;
            }
        }
        if let Status::Cleared(ref mut t) = g.status {
            *t += dt;
        }

        g.hover_room = (0..g.lvl().rooms.len()).find(|&i| g.lvl().rooms[i].rect().contains(m));

        if matches!(g.status, Status::Playing) {
            if is_mouse_button_pressed(MouseButton::Left) {
                if g.lvl().rooms[g.holder].slot().contains(m) {
                    g.dragging = true;
                } else {
                    for i in 0..g.lvl().rooms.len() {
                        if g.ghosts[i] && i != g.holder && g.lvl().rooms[i].slot().contains(m) {
                            g.err = Some((Diag::Moved(i), 4.0));
                        }
                    }
                }
            }
            if is_mouse_button_released(MouseButton::Left) && g.dragging {
                g.dragging = false;
                if let Some(t) = g.hover_room {
                    g.try_move(t);
                }
            }
        }

        // briefing is dismissed after the play input block, so the click that closes
        // it cannot also be read as the start of a drag
        if matches!(g.status, Status::Briefing) {
            if is_key_pressed(KeyCode::Space)
                || is_key_pressed(KeyCode::Enter)
                || is_mouse_button_pressed(MouseButton::Left)
            {
                g.status = Status::Playing;
            }
        } else if is_key_pressed(KeyCode::H) && matches!(g.status, Status::Playing) {
            g.status = Status::Briefing;
            g.dragging = false;
        }

        // the room the crate is hovering over, if dropping it there would be legal
        let preview = if g.dragging {
            g.hover_room.filter(|&t| g.lvl().edges.contains(&(g.holder, t)))
        } else {
            None
        };

        // ----------------------------------------------------------- render
        clear_background(BG);
        ui.rect(0.0, 0.0, DW, DH, BG);

        let lines = g.source(preview);
        let code_hover = draw_panel(&ui, &g, &lines, m);
        g.hover_from_code = code_hover;

        let holder = g.holder;
        let playing = matches!(g.status, Status::Playing);
        let edges = g.lvl().edges.clone();
        for (a, b) in &edges {
            let live = *a == holder && playing;
            let col = if live { alpha(BLUE, 0.55) } else { alpha(LINE, 0.7) };
            draw_arrow(
                &ui,
                g.lvl().rooms[*a].center(),
                g.lvl().rooms[*b].center(),
                col,
                if live { 3.0 } else { 2.0 },
            );
        }

        for i in 0..g.lvl().rooms.len() {
            let reachable = playing && edges.contains(&(holder, i));
            draw_room(&ui, &g, i, reachable, code_hover == Some(i));
        }

        match g.status {
            Status::Dropping(t) => {
                let k = (t / 0.9).min(1.0);
                let r = &g.lvl().rooms[g.holder];
                draw_crate(&ui, r.slot(), &r.binding, 1.0 - k * 0.85, 1.0 - k, 0.0);
                let rc = r.rect();
                ui.round_frame(rc.x, rc.y, rc.w, rc.h, 11.0, alpha(BLACK, 0.0), alpha(RED, k), 2.2);
            }
            _ => {
                let pulse = 0.5 + 0.5 * (g.t * 2.4).sin();
                let r = &g.lvl().rooms[g.holder];
                if g.dragging {
                    draw_crate(
                        &ui,
                        Rect::new(m.x - 44.0, m.y - 14.0, 88.0, 28.0),
                        &r.binding,
                        1.0,
                        1.0,
                        1.0,
                    );
                } else {
                    draw_crate(&ui, r.slot(), &r.binding, 1.0, 1.0, if playing { pulse } else { 0.0 });
                }
            }
        }

        if matches!(g.status, Status::Briefing) {
            draw_briefing(&ui, g.lvl());
        }

        // ------------------------------------------------------------ chrome
        let bw = PANEL_X - 20.0;
        ui.text(ui.tr(&g.lvl().title), 40.0, 52.0, 24.0, INK, Face::Sans);
        // the objective stays on screen for the whole level, not just the briefing
        let label = ui.tr(&S("GOAL", "目的"));
        ui.text(label, 40.0, 80.0, 13.0, FAINT, Face::Sans);
        let gx = 40.0 + ui.width(label, 13.0, Face::Sans) + 14.0;
        ui.text(ui.tr(&g.lvl().goal), gx, 80.0, 16.0, DIM, Face::Sans);

        let moves = g.path.len() - 1;
        let m_txt = match ui.lang {
            Lang::En => format!("{moves} moves"),
            Lang::Ja => format!("{moves} 手"),
        };
        ui.text_right(&m_txt, PANEL_X + PANEL_W, 52.0, 16.0, DIM, Face::Sans);

        // one key bar along the bottom, out of the way of the board
        let cmt = if ui.comments {
            S("C  comments off", "C  コメントを消す")
        } else {
            S("C  comments on", "C  コメントを出す")
        };
        let mut kx = 40.0;
        let mut key = |ui: &Ui, s: &S| {
            let t = ui.tr(s);
            ui.text(t, kx, 866.0, 15.0, FAINT, Face::Sans);
            kx += ui.width(t, 15.0, Face::Sans) + 34.0;
        };
        key(&ui, &S("H  why this level", "H  このステージの狙い"));
        key(&ui, &cmt);
        key(&ui, &S("R  restart", "R  やり直し"));
        if ui.bilingual() {
            key(&ui, &S("L  日本語", "L  English"));
        }

        if matches!(g.status, Status::Failed) {
            let r = &g.lvl().rooms[g.holder];
            let msg = match ui.lang {
                Lang::En => format!("`{}` dropped at the end of `{}`", r.binding, r.name),
                Lang::Ja => format!("`{}` は `{}` の終わりで破棄された", r.binding, r.name),
            };
            ui.rect(0.0, 380.0, bw, 130.0, alpha(BG, 0.93));
            ui.text_center(&msg, bw / 2.0, 435.0, 24.0, RED, Face::Sans);
            ui.text_center(
                ui.tr(&S(
                    "that scope never returns it  -  press R",
                    "そのスコープは値を返さない  -  R でやり直し",
                )),
                bw / 2.0,
                470.0,
                17.0,
                DIM,
                Face::Sans,
            );
        }

        if let Status::Cleared(t) = g.status {
            let a = (t / 0.35).min(1.0);
            ui.rect(0.0, 0.0, bw, DH, alpha(BG, 0.9 * a));
            ui.text_center(
                ui.tr(&S("delivered", "配達完了")),
                bw / 2.0,
                348.0,
                42.0,
                alpha(GREEN, a),
                Face::Sans,
            );
            let sub = match ui.lang {
                Lang::En => format!("{moves} moves, nothing copied"),
                Lang::Ja => format!("{moves} 手  -  コピーは一度も起きていない"),
            };
            ui.text_center(&sub, bw / 2.0, 384.0, 18.0, alpha(DIM, a), Face::Sans);

            // the point of the level, restated now that the player has actually done it
            ui.text_center(
                ui.tr(&S("WHAT THAT WAS", "いま起きたこと")),
                bw / 2.0,
                452.0,
                13.0,
                alpha(FAINT, a),
                Face::Sans,
            );
            ui.text_center(ui.tr(&g.lvl().takeaway), bw / 2.0, 488.0, 20.0, alpha(INK, a), Face::Sans);
            ui.text_center(
                ui.tr(&S(
                    "the program you just wrote is on the right",
                    "いま書いたプログラムが右にある",
                )),
                bw / 2.0,
                528.0,
                16.0,
                alpha(FAINT, a),
                Face::Sans,
            );
            let tail = if g.idx + 1 < g.levels.len() {
                S("press N for the next theme", "N で次のテーマへ")
            } else {
                S("next: shared references, then &mut", "次は共有参照、そして &mut")
            };
            ui.text_center(ui.tr(&tail), bw / 2.0, 590.0, 20.0, alpha(GREEN, a), Face::Sans);
        }

        next_frame().await
    }
}
