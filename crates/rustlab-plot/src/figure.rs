use std::cell::RefCell;

/// Named or RGB color for a plot series.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SeriesColor {
    Blue, Red, Green, Cyan, Magenta, Yellow, Black, White,
    Rgb(u8, u8, u8),
}

impl SeriesColor {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "r" | "red"     => Some(Self::Red),
            "g" | "green"   => Some(Self::Green),
            "b" | "blue"    => Some(Self::Blue),
            "c" | "cyan"    => Some(Self::Cyan),
            "m" | "magenta" => Some(Self::Magenta),
            "y" | "yellow"  => Some(Self::Yellow),
            "k" | "black"   => Some(Self::Black),
            "w" | "white"   => Some(Self::White),
            _ => None,
        }
    }
    /// Default color cycle (matplotlib-like).
    pub fn cycle(idx: usize) -> Self {
        match idx % 6 {
            0 => Self::Cyan,
            1 => Self::Yellow,
            2 => Self::Green,
            3 => Self::Magenta,
            4 => Self::Red,
            _ => Self::Blue,
        }
    }
    pub fn to_plotters(&self) -> plotters::style::RGBColor {
        use plotters::style::RGBColor;
        match self {
            Self::Blue    => RGBColor(31, 119, 180),
            Self::Red     => RGBColor(214,  39,  40),
            Self::Green   => RGBColor( 44, 160,  44),
            Self::Cyan    => RGBColor( 23, 190, 207),
            Self::Magenta => RGBColor(148, 103, 189),
            Self::Yellow  => RGBColor(188, 189,  34),
            Self::Black   => RGBColor(  0,   0,   0),
            Self::White   => RGBColor(255, 255, 255),
            Self::Rgb(r,g,b) => RGBColor(*r, *g, *b),
        }
    }
    pub fn to_ratatui(&self) -> ratatui::style::Color {
        use ratatui::style::Color;
        match self {
            Self::Blue    => Color::Blue,
            Self::Red     => Color::Red,
            Self::Green   => Color::Green,
            Self::Cyan    => Color::Cyan,
            Self::Magenta => Color::Magenta,
            Self::Yellow  => Color::Yellow,
            Self::Black   => Color::Black,
            Self::White   => Color::White,
            Self::Rgb(r,g,b) => Color::Rgb(*r, *g, *b),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LineStyle { Solid, Dashed }

#[derive(Debug, Clone, PartialEq)]
pub enum PlotKind { Line, Stem, Bar, Scatter }

/// One data series in a subplot.
#[derive(Debug, Clone)]
pub struct Series {
    pub label: String,
    pub x_data: Vec<f64>,
    pub y_data: Vec<f64>,
    pub color:  SeriesColor,
    pub style:  LineStyle,
    pub kind:   PlotKind,
}

/// State for a single subplot panel.
#[derive(Debug, Clone)]
pub struct SubplotState {
    pub title:  String,
    pub xlabel: String,
    pub ylabel: String,
    pub grid:   bool,
    pub series: Vec<Series>,
    pub xlim:   (Option<f64>, Option<f64>),
    pub ylim:   (Option<f64>, Option<f64>),
}
impl SubplotState {
    pub fn new() -> Self {
        Self {
            title: String::new(), xlabel: String::new(), ylabel: String::new(),
            grid: true, series: Vec::new(),
            xlim: (None, None), ylim: (None, None),
        }
    }
}

/// Global per-thread figure state shared by all plot builtins.
#[derive(Debug, Clone)]
pub struct FigureState {
    pub hold: bool,
    pub subplot_rows: usize,
    pub subplot_cols: usize,
    pub current_subplot: usize,
    pub subplots: Vec<SubplotState>,
}
impl FigureState {
    pub fn new() -> Self {
        Self {
            hold: false,
            subplot_rows: 1, subplot_cols: 1,
            current_subplot: 0,
            subplots: vec![SubplotState::new()],
        }
    }
    pub fn reset(&mut self) { *self = Self::new(); }
    pub fn current(&self) -> &SubplotState {
        let i = self.current_subplot.min(self.subplots.len().saturating_sub(1));
        &self.subplots[i]
    }
    pub fn current_mut(&mut self) -> &mut SubplotState {
        let i = self.current_subplot.min(self.subplots.len().saturating_sub(1));
        &mut self.subplots[i]
    }
    /// Switch to subplot (rows×cols, 1-based idx).
    pub fn set_subplot(&mut self, rows: usize, cols: usize, idx: usize) {
        let n = rows * cols;
        if self.subplot_rows != rows || self.subplot_cols != cols {
            self.subplot_rows = rows;
            self.subplot_cols = cols;
            self.subplots = (0..n).map(|_| SubplotState::new()).collect();
        } else {
            while self.subplots.len() < n {
                self.subplots.push(SubplotState::new());
            }
        }
        self.current_subplot = (idx.saturating_sub(1)).min(n.saturating_sub(1));
        self.hold = false;
    }
    /// Color for the next series added to current subplot.
    pub fn next_color(&self) -> SeriesColor {
        SeriesColor::cycle(self.current().series.len())
    }
}

thread_local! {
    pub static FIGURE: RefCell<FigureState> = RefCell::new(FigureState::new());
}

// ─── Colormap ──────────────────────────────────────────────────────────────

/// Interpolate a colormap at normalised position t ∈ [0,1].
/// Supported names: "viridis" (default), "jet", "hot", "gray".
pub fn colormap_rgb(t: f64, name: &str) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    type Pts = &'static [(f64, (u8, u8, u8))];
    let pts: Pts = match name {
        "jet" => &[
            (0.00, (  0,   0, 128)),
            (0.25, (  0, 128, 255)),
            (0.50, (  0, 255, 128)),
            (0.75, (255, 255,   0)),
            (1.00, (128,   0,   0)),
        ],
        "hot" => &[
            (0.00, (  0,   0,   0)),
            (0.33, (255,   0,   0)),
            (0.67, (255, 255,   0)),
            (1.00, (255, 255, 255)),
        ],
        "gray" => &[
            (0.00, (  0,   0,   0)),
            (1.00, (255, 255, 255)),
        ],
        _ => &[  // viridis
            (0.00, ( 68,   1,  84)),
            (0.25, ( 59,  82, 139)),
            (0.50, ( 33, 145, 140)),
            (0.75, ( 94, 201,  98)),
            (1.00, (253, 231,  37)),
        ],
    };
    for w in pts.windows(2) {
        let (t0, c0) = w[0];
        let (t1, c1) = w[1];
        if t >= t0 && t <= t1 {
            let s = (t - t0) / (t1 - t0);
            let lerp = |a: u8, b: u8| (a as f64 * (1.0 - s) + b as f64 * s).round() as u8;
            return (lerp(c0.0, c1.0), lerp(c0.1, c1.1), lerp(c0.2, c1.2));
        }
    }
    pts.last().map(|(_, c)| *c).unwrap_or((0, 0, 0))
}
