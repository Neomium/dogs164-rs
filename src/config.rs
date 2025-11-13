use crate::commands::{
    DisplayConfig, DisplaySettings, DoubleHeight, EntryModeSettings, FontWidth, HorizontalDir,
    LineDisplayMode, OscillatorSettings, Rom, SegCommControl, VerticalDir, ViewMode,
};

/// Configuration structure holding current settings
pub struct Config {
    pub display_control: DisplayConfig,

    pub entry_mode: EntryModeSettings,

    pub seg_comm: SegCommControl,

    pub view_mode: ViewMode,

    pub double_height: Option<DoubleHeight>,

    pub charset: Rom,

    pub display_settings: DisplaySettings,

    pub osc_freq_cmd: OscillatorSettings,

    pub bw_inversion: bool,

    pub font_width: FontWidth,

    pub four_line_enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            four_line_enabled: true,
            bw_inversion: false,
            font_width: FontWidth::FiveDot,
            display_control: DisplayConfig::DISPLAY_ON,
            entry_mode: EntryModeSettings::new(HorizontalDir::LeftToRight, false),
            seg_comm: SegCommControl {
                seg_dir: HorizontalDir::LeftToRight,
                com_dir: VerticalDir::TopToBottom,
            },
            view_mode: ViewMode::Top,
            charset: Rom::A,
            display_settings: DisplaySettings::new(
                LineDisplayMode::TwoOrFourLines,
                false,
                false,
                false,
            ),
            osc_freq_cmd: OscillatorSettings::default(),
            double_height: None,
        }
    }
}
