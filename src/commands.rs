use bitflags::bitflags;

pub const MODE_COMMAND: u8 = 0x00;
pub const MODE_DATA: u8 = 0x40;

pub const CMD_CLEAR_DISPLAY: u8 = 0x01;
pub const CMD_RETURN_HOME: u8 = 0x02;
pub const CMD_ENTRY_MODE_SET: u8 = 0x04;
pub const CMD_DISPLAY_SHIFT_LEFT: u8 = 0x08;
pub const CMD_DISPLAY_SHIFT_RIGHT: u8 = 0x0C;
pub const CMD_CURSOR_SHIFT_LEFT: u8 = 0x00;
pub const CMD_CURSOR_SHIFT_RIGHT: u8 = 0x04;
pub const CMD_SHIFT: u8 = 0x10;

pub const ADDR_CGRAM: u8 = 0x40;
pub const ADDR_DDRAM: u8 = 0x80;
pub const ADDR_DDRAM_TOP_OFFSET: u8 = 0x04;

/// RE = 1, IS = 0
pub const CMD_8BIT_4LINES_RE1_IS0: u8 = 0x3A;
/// RE = 0, IS = 1
pub const CMD_8BIT_4LINES_RE0_IS1: u8 = 0x39;
/// RE = 0, IS = 0 , DH = 1 (Double Height)
pub const CMD_8BIT_4LINES_RE0_IS0_DH1: u8 = 0x3C;
/// RE = 0, IS = 1 , DH = 1 (Double Height)
pub const CMD_8BIT_4LINES_RE0_IS1_DH1: u8 = 0x3D;
/// RE = 0, IS = 0
pub const CMD_8BIT_4LINES_RE0_IS0: u8 = 0x38;

// Commands from extended set (RE = 1, IS = 0)
pub const CMD_BS1_1: u8 = 0x1E;
pub const CMD_POWER_DOWN_DISABLE: u8 = 0x02;

// Commands from extended set (RE = 0, IS = 1)
pub const CMD_DISPLAY: u8 = 0x08;

// Other commands / defaults
pub const CMD_BS0_1: u8 = 0x1B;
pub const CMD_INTERNAL_DIVIDER: u8 = 0x13;
pub const CMD_CONTRAST_DEFAULT_DOGS164: u8 = 0x6B;
pub const CMD_POWER_CONTROL_DOGS164: u8 = 0x56;
pub const CMD_POWER_ICON_CONTRAST: u8 = 0x5C;
pub const CMD_FOLLOWER_CONTROL_DOGS164: u8 = 0x6C;
pub const CMD_FOLLOWER_CONTROL: u8 = 0x60;
pub const CMD_ROM_SELECT: u8 = 0x72;

pub const COMMAND_3LINES_TOP: u8 = 0x1F;
pub const COMMAND_3LINES_MIDDLE: u8 = 0x17;
pub const COMMAND_3LINES_BOTTOM: u8 = 0x13;
pub const COMMAND_2LINES: u8 = 0x1B;

bitflags! {
    pub struct DisplayConfig: u8 {
        const DISPLAY_ON = 0x04;
        const CURSOR_ON  = 0x02;
        const BLINK_ON   = 0x01;
    }
}

pub struct EntryModeSettings {
    pub direction: HorizontalDir,
    pub shift_incr: bool,
}

impl EntryModeSettings {
    pub fn new(direction: HorizontalDir, shift_incr: bool) -> Self {
        Self {
            direction,
            shift_incr,
        }
    }

    pub fn cmd(&self) -> u8 {
        let shift_bit = if self.shift_incr { 0x01 } else { 0x00 };
        let direction_bit = match self.direction {
            HorizontalDir::LeftToRight => 0x02,
            HorizontalDir::RightToLeft => 0x00,
        };

        CMD_ENTRY_MODE_SET | direction_bit | shift_bit
    }
}

pub struct SegCommControl {
    pub seg_dir: HorizontalDir,
    pub com_dir: VerticalDir,
}

impl SegCommControl {
    pub fn new(seg_dir: HorizontalDir, com_dir: VerticalDir) -> Self {
        Self { seg_dir, com_dir }
    }

    pub fn cmd(&self) -> u8 {
        let seg_bit = match self.seg_dir {
            HorizontalDir::LeftToRight => 0x01,
            HorizontalDir::RightToLeft => 0x00,
        };
        let com_bit = match self.com_dir {
            VerticalDir::TopToBottom => 0x20,
            VerticalDir::BottomToTop => 0x00,
        };

        0x40 | seg_bit | com_bit // SEG/COM direction control
    }
}

pub enum HorizontalDir {
    RightToLeft,
    LeftToRight,
}

pub enum VerticalDir {
    TopToBottom,
    BottomToTop,
}

pub enum ShiftType {
    Display,
    Cursor,
}

pub struct ShiftSettings {
    pub mode: HorizontalDir,
    pub shift_type: ShiftType,
}

impl ShiftSettings {
    pub fn new(mode: HorizontalDir, shift_type: ShiftType) -> Self {
        Self { mode, shift_type }
    }
    pub fn cmd(&self) -> u8 {
        match (&self.mode, &self.shift_type) {
            (HorizontalDir::RightToLeft, ShiftType::Display) => CMD_SHIFT | CMD_DISPLAY_SHIFT_LEFT,
            (HorizontalDir::RightToLeft, ShiftType::Cursor) => CMD_SHIFT | CMD_CURSOR_SHIFT_LEFT,
            (HorizontalDir::LeftToRight, ShiftType::Display) => CMD_SHIFT | CMD_DISPLAY_SHIFT_RIGHT,
            (HorizontalDir::LeftToRight, ShiftType::Cursor) => CMD_SHIFT | CMD_CURSOR_SHIFT_RIGHT,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ViewMode {
    Top = 0x05,
    Bottom = 0x06,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DoubleHeight {
    Lines2 = 2,
    Lines3Top = 3,
    Lines3Middle = 1,
    Lines3Bottom = 0,
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum Rom {
    A = 0x00,
    B = 0x04,
    C = 0x08,
}

pub struct PowerIconContrast {
    pub booster: bool,
    pub icon: bool,   // 0-7
    pub contrast: u8, // 0-15
}

impl Default for PowerIconContrast {
    fn default() -> Self {
        Self {
            booster: true,
            icon: false,
            contrast: 42,
        }
    }
}

impl PowerIconContrast {
    pub fn new(booster: bool, icon: bool, contrast: u8) -> Self {
        let contrast_checked = if contrast > 63 { 63 } else { contrast };

        Self {
            booster,
            icon,
            contrast: contrast_checked,
        }
    }

    pub fn cmd_byte1(&self) -> u8 {
        let mut byte = 0x50;

        if self.booster {
            byte |= 1 << 2;
        }

        if self.icon {
            byte |= 1 << 3;
        }

        byte | ((self.contrast >> 4) & 0x3) // Only C5, C4 bits for contrast
    }

    pub fn cmd_byte2(&self) -> u8 {
        0x70 | ((self.contrast) & 0x0F) // C3-C0 bits for contrast
    }
}

pub struct FollowerControl {
    pub rab: Rab,
    pub d_on: bool,
}

impl FollowerControl {
    pub fn new(rab: Rab, d_on: bool) -> Self {
        Self { rab, d_on }
    }

    pub fn cmd(&self) -> u8 {
        let mut cmd = 0x60; // Base command for follower control

        // Set RAB bits
        cmd |= self.rab as u8;

        // Set D bit
        if self.d_on {
            cmd |= 1 << 3;
        }

        cmd
    }
}

impl Default for FollowerControl {
    fn default() -> Self {
        Self {
            rab: Rab::IR4_3p6,
            d_on: true,
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, Eq, PartialEq, Default)]
pub enum Rab {
    IR0_1p9 = 0b000,
    IR1_2p2 = 0b001,
    IR2_2p6 = 0b010,
    IR3_3p0 = 0b011,
    #[default]
    IR4_3p6 = 0b100,
    IR5_4p4 = 0b101,
    IR6_5p3 = 0b110,
    IR7_6p5 = 0b111,
}

pub struct ExtendedFunctionSet {
    /// FW bit
    font_width: FontWidth,

    /// B/W bit
    bw_inversion: bool,

    /// NW bit
    four_line_enabled: bool,
}

impl ExtendedFunctionSet {
    pub fn new(font_width: FontWidth, bw_inversion: bool, four_line_enabled: bool) -> Self {
        Self {
            font_width,
            bw_inversion,
            four_line_enabled,
        }
    }

    pub fn cmd(&self) -> u8 {
        let mut cmd = 0x08; // Base command for 8-bit interface

        // Set font width
        cmd |= (self.font_width as u8) << 2;

        // Set black/white inversion
        if self.bw_inversion {
            cmd |= 1 << 1;
        }

        // Set 4-line mode
        if self.four_line_enabled {
            cmd |= 1;
        }

        cmd
    }
}

impl Default for ExtendedFunctionSet {
    fn default() -> Self {
        Self {
            font_width: FontWidth::FiveDot,
            bw_inversion: false,
            four_line_enabled: true,
        }
    }
}

/// Function set command
pub struct DisplaySettings {
    /// N Bit
    pub line_number_control: LineDisplayMode,

    /// DH bit (RE=0 only)
    pub double_height: bool,

    /// BE bit (RE=1 only)
    pub data_blink_enable: bool,

    /// REV bit (RE=1 only)
    pub reverse_enable: bool,
}

impl DisplaySettings {
    pub fn new(
        line_number_control: LineDisplayMode,
        double_height: bool,
        data_blink: bool,
        reverse_enabled: bool,
    ) -> Self {
        Self {
            line_number_control,
            double_height,
            data_blink_enable: data_blink,
            reverse_enable: reverse_enabled,
        }
    }

    pub fn cmd_re0_is0(&self) -> u8 {
        let mut cmd = 0x30; // Base command for RE=0

        // Set line number control
        cmd |= (self.line_number_control as u8) << 3;

        // Set double height
        if self.double_height {
            cmd |= 1 << 2;
        }

        cmd
    }

    pub fn cmd_re0_is1(&self) -> u8 {
        let mut cmd = self.cmd_re0_is0();

        cmd |= 1;

        cmd
    }

    pub fn cmd_re1_is0(&self) -> u8 {
        let mut cmd = 0x32; // Base command for RE=1

        // Set line number control
        cmd |= (self.line_number_control as u8) << 3;

        // Set data blink enable
        if self.data_blink_enable {
            cmd |= 1 << 2;
        }

        if self.reverse_enable {
            cmd |= 1;
        }

        cmd
    }

    pub fn cmds_re1_is1(&self) -> [u8; 2] {
        let cmd1 = self.cmd_re0_is1();
        let cmd2 = self.cmd_re1_is0();

        [cmd1, cmd2]
    }
}

pub struct DoubleHeightBiasDisplayShift {
    pub double_height_mode: DoubleHeight,

    pub display_dot_shift: bool,

    pub bs1: bool,
}

impl DoubleHeightBiasDisplayShift {
    pub fn new(double_height_mode: DoubleHeight, display_dot_shift: bool, bs1: bool) -> Self {
        Self {
            double_height_mode,
            display_dot_shift,
            bs1,
        }
    }

    pub fn cmd(&self) -> u8 {
        let mut cmd = 0x10; // Base command for line mode

        // Set line mode bits
        cmd |= (self.double_height_mode as u8) << 2;

        // Set display dot shift
        if self.display_dot_shift {
            cmd |= 1;
        }

        // Set BS1 bit
        if self.bs1 {
            cmd |= 1 << 1;
        }

        cmd
    }
}

pub struct OscillatorSettings {
    pub freq: OscillatorFreq,
    pub bs0: bool,
}

impl Default for OscillatorSettings {
    fn default() -> Self {
        Self {
            freq: OscillatorFreq::Freq540kHz,
            bs0: true,
        }
    }
}

impl OscillatorSettings {
    pub fn new(freq: OscillatorFreq, bs0: bool) -> Self {
        Self { freq, bs0 }
    }

    pub fn cmd(&self) -> u8 {
        let mut cmd = 0x10; // Base command for oscillator frequency

        // Set frequency bits
        cmd |= self.freq as u8;

        // Set BS0 bit
        if self.bs0 {
            cmd |= 1 << 3;
        }

        cmd
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum FontWidth {
    #[default]
    FiveDot = 0x00,
    SixDot = 0x01,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineDisplayMode {
    OneOrThreeLines = 0b0,
    TwoOrFourLines = 0b1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OscillatorFreq {
    Freq680kHz = 0b111,
    Freq640kHz = 0b110,
    Freq620kHz = 0b101,
    Freq580kHz = 0b100,
    Freq540kHz = 0b011,
    Freq500kHz = 0b010,
    Freq460kHz = 0b001,
    Freq420kHz = 0b000,
}

mod tests {
    use super::*;

    #[test]
    fn test_entry_mode_settings() {
        let ems = EntryModeSettings::new(HorizontalDir::LeftToRight, true);
        assert_eq!(ems.cmd(), 0x07);

        let ems = EntryModeSettings::new(HorizontalDir::RightToLeft, false);
        assert_eq!(ems.cmd(), 0x04);

        let ems = EntryModeSettings::new(HorizontalDir::LeftToRight, false);
        assert_eq!(ems.cmd(), 0x06);
    }

    #[test]
    fn test_seg_comm_control() {
        let scc = SegCommControl::new(HorizontalDir::LeftToRight, VerticalDir::TopToBottom);
        assert_eq!(scc.cmd(), 0x61);

        let scc = SegCommControl::new(HorizontalDir::RightToLeft, VerticalDir::BottomToTop);
        assert_eq!(scc.cmd(), 0x40);
    }

    #[test]
    fn test_shift_settings() {
        let ss = ShiftSettings::new(HorizontalDir::LeftToRight, ShiftType::Display);
        assert_eq!(ss.cmd(), 0x1C);

        let ss = ShiftSettings::new(HorizontalDir::RightToLeft, ShiftType::Cursor);
        assert_eq!(ss.cmd(), 0x10);
    }

    #[test]
    fn test_power_icon_contrast() {
        let pic = PowerIconContrast::new(true, true, 45);
        assert_eq!(pic.cmd_byte1(), 0x5E);
        assert_eq!(pic.cmd_byte2(), 0x7D);

        let pic = PowerIconContrast::new(false, false, 15);
        assert_eq!(pic.cmd_byte1(), 0x50);
        assert_eq!(pic.cmd_byte2(), 0x7F);
    }

    #[test]
    fn test_function_set_cmd() {
        let fsc = DisplaySettings::new(LineDisplayMode::TwoOrFourLines, false, false, false);

        assert_eq!(fsc.cmd_re0_is0(), 0x38); // RE=0, IS=0
        assert_eq!(fsc.cmd_re0_is1(), 0x39); // RE=0, IS=1
        assert_eq!(fsc.cmd_re1_is0(), 0x3A); // RE=1, IS=0
        assert_eq!(fsc.cmds_re1_is1(), [0x39, 0x3A]); // RE=1, IS=1

        let fsc_dh = DisplaySettings::new(LineDisplayMode::TwoOrFourLines, true, false, false);
        assert_eq!(fsc_dh.cmd_re0_is0(), 0x3C);
        assert_eq!(fsc_dh.cmd_re0_is1(), 0x3D);
    }

    #[test]
    fn test_extended_function_set() {
        let efs = ExtendedFunctionSet::new(FontWidth::SixDot, true, true);
        assert_eq!(efs.cmd(), 0x0F);
        let efs = ExtendedFunctionSet::new(FontWidth::FiveDot, false, false);
        assert_eq!(efs.cmd(), 0x08);
    }

    #[test]
    fn test_follower_control() {
        let fc = FollowerControl::new(Rab::IR3_3p0, true);
        assert_eq!(fc.cmd(), 0x6B);
        let fc = FollowerControl::new(Rab::IR0_1p9, false);
        assert_eq!(fc.cmd(), 0x60);
    }

    #[test]
    fn test_osc_freq_cmd() {
        let ofc = OscillatorSettings::new(OscillatorFreq::Freq540kHz, true);
        assert_eq!(ofc.cmd(), 0x1B);
        let ofc = OscillatorSettings::new(OscillatorFreq::Freq420kHz, false);
        assert_eq!(ofc.cmd(), 0x10);
    }
}
