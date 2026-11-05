//! The crossfeed itself, without Extism, so it is tested natively.
//!
//! Each channel gets the other one, low-passed and lowered by `level`, the
//! way a listener in a room hears both speakers with both ears: the far
//! speaker later, quieter and duller. The sum is scaled so that a sound in
//! the middle (the same on both channels) keeps its level.

/// The user's settings, checked.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Settings {
    /// How far below the direct sound the other channel is mixed in.
    pub level_db: f32,
    /// Where the low-pass on the other channel starts to cut.
    pub cutoff_hz: f32,
}

impl Settings {
    pub const DEFAULT: Settings = Settings { level_db: 4.5, cutoff_hz: 700.0 };

    /// Reads the settings as the user typed them; empty means the default.
    pub fn parse(level: &str, cutoff: &str) -> Result<Settings, String> {
        Ok(Settings {
            level_db: number("Level", level, Settings::DEFAULT.level_db, 1.0..=15.0)?,
            cutoff_hz: number("Cutoff", cutoff, Settings::DEFAULT.cutoff_hz, 300.0..=2000.0)?,
        })
    }
}

fn number(name: &str, text: &str, default: f32, range: std::ops::RangeInclusive<f32>) -> Result<f32, String> {
    let text = text.trim();
    if text.is_empty() {
        return Ok(default);
    }
    match text.parse::<f32>() {
        Ok(v) if range.contains(&v) => Ok(v),
        _ => Err(format!("{name} must be a number from {} to {}, not {text:?}", range.start(), range.end())),
    }
}

/// A crossfeed at one sample rate, with its filters' state.
#[derive(Debug, Clone)]
pub struct Crossfeed {
    /// The other channel's gain.
    feed: f32,
    /// Keeps a sound in the middle at its level.
    norm: f32,
    /// The low-pass coefficient.
    alpha: f32,
    /// Each channel, low-passed.
    low: [f32; 2],
}

impl Crossfeed {
    pub fn new(s: Settings, rate: u32) -> Crossfeed {
        let feed = 10f32.powf(-s.level_db / 20.0);
        let alpha = 1.0 - (-std::f32::consts::TAU * s.cutoff_hz / rate.max(1) as f32).exp();
        Crossfeed { feed, norm: 1.0 / (1.0 + feed), alpha, low: [0.0; 2] }
    }

    /// Changes interleaved stereo samples in place.
    pub fn process(&mut self, block: &mut [f32]) {
        for frame in block.as_chunks_mut::<2>().0 {
            let [l, r] = *frame;
            self.low[0] += self.alpha * (l - self.low[0]);
            self.low[1] += self.alpha * (r - self.low[1]);
            frame[0] = (l + self.feed * self.low[1]) * self.norm;
            frame[1] = (r + self.feed * self.low[0]) * self.norm;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The last frame after `frames` frames of the same `frame`.
    fn settled(frame: [f32; 2], frames: usize) -> [f32; 2] {
        let mut cf = Crossfeed::new(Settings::DEFAULT, 44_100);
        let mut block = frame.repeat(frames);
        cf.process(&mut block);
        [block[block.len() - 2], block[block.len() - 1]]
    }

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-3
    }

    #[test]
    fn a_sound_in_the_middle_keeps_its_level() {
        let [l, r] = settled([0.5, 0.5], 4096);
        assert!(near(l, 0.5) && near(r, 0.5), "{l} {r}");
    }

    #[test]
    fn a_low_sound_on_one_side_reaches_the_other_ear_lowered_by_the_level() {
        let [l, r] = settled([1.0, 0.0], 4096);
        let feed = 10f32.powf(-4.5 / 20.0);
        assert!(near(l, 1.0 / (1.0 + feed)) && near(r, feed / (1.0 + feed)), "{l} {r}");
        assert!(near(20.0 * (l / r).log10(), 4.5));
    }

    #[test]
    fn a_high_sound_on_one_side_hardly_reaches_the_other_ear() {
        // The highest frequency there is: +1, -1, +1, ... on the left.
        let mut cf = Crossfeed::new(Settings::DEFAULT, 44_100);
        let mut block: Vec<f32> = (0..4096).flat_map(|i| [if i % 2 == 0 { 1.0 } else { -1.0 }, 0.0]).collect();
        cf.process(&mut block);
        let right = block[2048..].iter().skip(1).step_by(2).fold(0f32, |m, s| m.max(s.abs()));
        assert!(right < 0.05, "{right}");
    }

    #[test]
    fn a_new_crossfeed_forgets_the_last_track() {
        let mut cf = Crossfeed::new(Settings::DEFAULT, 44_100);
        cf.process(&mut [1.0, 0.0].repeat(4096));
        // Silence after a loud left: the filter still rings into the right.
        let mut kept = [0.0, 0.0];
        cf.process(&mut kept);
        assert!(kept[1] > 0.1, "{kept:?}");
        let mut fresh = [0.0, 0.0];
        Crossfeed::new(Settings::DEFAULT, 44_100).process(&mut fresh);
        assert_eq!(fresh, [0.0, 0.0]);
    }

    #[test]
    fn settings_are_checked_and_empty_means_default() {
        assert_eq!(Settings::parse("", " "), Ok(Settings::DEFAULT));
        assert_eq!(Settings::parse("6", "1000"), Ok(Settings { level_db: 6.0, cutoff_hz: 1000.0 }));
        assert_eq!(Settings::parse("0", ""), Err("Level must be a number from 1 to 15, not \"0\"".into()));
        assert_eq!(Settings::parse("", "loud"), Err("Cutoff must be a number from 300 to 2000, not \"loud\"".into()));
    }
}
