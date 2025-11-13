use crate::commands::{
    CMD_8BIT_4LINES_RE0_IS0, CMD_8BIT_4LINES_RE0_IS1, CMD_8BIT_4LINES_RE1_IS0, CMD_BS0_1,
    CMD_BS1_1, CMD_CLEAR_DISPLAY, CMD_CONTRAST_DEFAULT_DOGS164, CMD_DISPLAY,
    CMD_FOLLOWER_CONTROL_DOGS164, CMD_POWER_CONTROL_DOGS164, CMD_RETURN_HOME, COMMAND_2LINES,
    COMMAND_3LINES_BOTTOM, COMMAND_3LINES_MIDDLE, COMMAND_3LINES_TOP, DisplayConfig,
    EntryModeSettings, ExtendedFunctionSet, MODE_COMMAND, MODE_DATA,
};
use crate::commands::{DoubleHeight, ViewMode};
use crate::config::Config;
use embedded_hal::delay::DelayNs;
use embedded_hal::i2c::{Error as I2cErr, I2c};
use heapless::Vec;

#[derive(Debug)]
pub enum LcdError<E: I2cErr> {
    I2c(E),
    InvalidInputData,
}

impl<E: I2cErr> From<E> for LcdError<E> {
    fn from(err: E) -> Self {
        LcdError::I2c(err)
    }
}

/// Trait defining the LCD operations
pub trait Lcd {
    type Error;

    fn init(&mut self, config: Config) -> Result<(), Self::Error>;

    /// Clear display and set cursor to home position
    fn clear(&mut self) -> Result<(), Self::Error>;

    /// Set cursor to home position
    fn home(&mut self) -> Result<(), Self::Error>;

    /// Set cursor position. Row and column are 1-based.
    fn locate(&mut self, row: u8, col: u8) -> Result<(), Self::Error>;

    /// Write a string to the display at the current cursor position
    fn write(&mut self, s: &str) -> Result<(), Self::Error>;

    /// Set display control (display on/off, cursor on/off, blink on/off)
    fn set_display(&mut self, flags: DisplayConfig) -> Result<(), Self::Error>;

    /// Set entry mode (set cursor/blink direction and enables shift for shift-enabled lines)
    fn set_entry_mode(&mut self, mode: EntryModeSettings) -> Result<(), Self::Error>;

    /// Set view mode (top or bottom)
    fn set_view_mode(&mut self, mode: ViewMode) -> Result<(), Self::Error>;

    fn set_cursor_off(&mut self) -> Result<(), Self::Error>;

    fn set_blinking_off(&mut self) -> Result<(), Self::Error>;

    fn extended_function_set(&mut self) -> Result<(), Self::Error>;

    fn set_double_height(&mut self) -> Result<(), Self::Error>;

    fn clear_line(&mut self, line: u8) -> Result<(), Self::Error>;

    fn clear_chars(&mut self, row_col: (u8, u8), chars: u8) -> Result<(), Self::Error>;
}

pub struct SSD18030<'a, B: I2c, D: DelayNs> {
    i2c: B,

    delay: &'a mut D,

    address: u8,

    ddram_start: u8,

    config: Config,
}

impl<'a, B: I2c, D: DelayNs> SSD18030<'a, B, D> {
    pub fn new_i2c(i2c: B, address: u8, delay: &'a mut D) -> Self {
        SSD18030 {
            i2c,
            delay,
            address,
            ddram_start: 0x84, // Top view
            config: Config::default(),
        }
    }

    pub fn send_command(&mut self, command: u8) -> Result<(), B::Error> {
        let bytes = [MODE_COMMAND, command];
        self.i2c.write(self.address, &bytes)?;
        Ok(())
    }

    pub fn send_data_byte(&mut self, data: u8) -> Result<(), B::Error> {
        let bytes = [MODE_DATA, data];
        self.i2c.write(self.address, &bytes)?;
        Ok(())
    }

    pub fn send_data(&mut self, data: &[u8]) -> Result<(), LcdError<B::Error>> {
        if data.len() > 31 {
            return Err(LcdError::InvalidInputData);
        }

        let mut vec: Vec<u8, 32> = Vec::new();
        vec.push(MODE_DATA).unwrap();
        vec.extend_from_slice(&data[..data.len()]).unwrap();
        self.i2c.write(self.address, &vec)?;
        Ok(())
    }

    fn finish_cmd(&mut self) -> Result<(), B::Error> {
        self.send_command(self.config.display_settings.cmd_re0_is0())?;
        Ok(())
    }

    fn re0_is0_cmd(&mut self) -> Result<(), B::Error> {
        self.send_command(self.config.display_settings.cmd_re0_is0())?;
        Ok(())
    }

    fn re0_is1_cmd(&mut self) -> Result<(), B::Error> {
        self.send_command(self.config.display_settings.cmd_re0_is1())?;
        Ok(())
    }

    fn re1_is0_cmd(&mut self) -> Result<(), B::Error> {
        self.send_command(self.config.display_settings.cmd_re1_is0())?;
        Ok(())
    }

    fn re1_is1_cmd(&mut self) -> Result<(), B::Error> {
        self.send_command(self.config.display_settings.cmds_re1_is1()[0])?;
        self.send_command(self.config.display_settings.cmds_re1_is1()[1])?;
        Ok(())
    }

    fn set_bias(&mut self) -> Result<(), B::Error> {
        self.send_command(self.config.display_settings.cmd_re1_is0())?;
        self.send_command(CMD_BS1_1)?;
        self.send_command(self.config.display_settings.cmd_re0_is1())?;
        self.send_command(CMD_BS0_1)?;
        Ok(())
    }

    pub fn setup(&mut self) -> Result<(), B::Error> {
        self.send_command(CMD_8BIT_4LINES_RE0_IS0)?;
        self.send_command(0x06)?;
        self.send_command(CMD_8BIT_4LINES_RE1_IS0)?;
        self.send_command(0x09)?;
        self.send_command(ViewMode::Top as u8)?;
        self.send_command(CMD_BS1_1)?;
        self.send_command(CMD_8BIT_4LINES_RE0_IS1)?;
        self.send_command(CMD_BS0_1)?;
        self.send_command(CMD_FOLLOWER_CONTROL_DOGS164)?;
        self.send_command(CMD_POWER_CONTROL_DOGS164)?;
        self.send_command(CMD_CONTRAST_DEFAULT_DOGS164)?;
        self.send_command(CMD_8BIT_4LINES_RE0_IS0)?;
        let display_cfg =
            DisplayConfig::DISPLAY_ON | DisplayConfig::CURSOR_ON | DisplayConfig::BLINK_ON;
        self.send_command(CMD_DISPLAY | display_cfg.bits())?;
        self.send_command(CMD_8BIT_4LINES_RE0_IS0)?;
        // self.send_command(0x84)?;
        self.delay.delay_ms(100);
        self.send_command(CMD_CLEAR_DISPLAY)?;
        Ok(())
    }
}

impl<B: I2c, D: DelayNs> Lcd for SSD18030<'_, B, D> {
    type Error = LcdError<B::Error>;

    fn init(&mut self, config: Config) -> Result<(), Self::Error> {
        self.delay.delay_ms(15);
        self.set_entry_mode(config.entry_mode)?;

        self.delay.delay_ms(100);
        self.set_view_mode(config.view_mode)?;

        self.delay.delay_ms(100);
        self.set_double_height()?;

        self.delay.delay_ms(100);
        self.extended_function_set()?;

        self.delay.delay_ms(100);
        self.set_bias()?;

        self.delay.delay_ms(100);
        self.send_command(self.config.display_settings.cmd_re0_is1())?;

        self.delay.delay_ms(100);
        self.send_command(CMD_FOLLOWER_CONTROL_DOGS164)?;

        self.delay.delay_ms(100);
        self.send_command(CMD_POWER_CONTROL_DOGS164)?;

        self.delay.delay_ms(100);
        self.send_command(CMD_CONTRAST_DEFAULT_DOGS164)?;

        self.delay.delay_ms(100);
        self.set_display(config.display_control)?;

        self.delay.delay_ms(100);
        self.locate(1, 1)?;

        self.delay.delay_ms(100);
        self.clear()?;
        Ok(())
    }

    fn clear(&mut self) -> Result<(), LcdError<B::Error>> {
        self.send_command(CMD_CLEAR_DISPLAY)?;
        Ok(())
    }

    fn home(&mut self) -> Result<(), LcdError<B::Error>> {
        self.send_command(CMD_RETURN_HOME)?;
        Ok(())
    }

    fn locate(&mut self, row: u8, col: u8) -> Result<(), LcdError<B::Error>> {
        if col > 16 || col == 0 || row == 0 || row > 4 {
            return Err(LcdError::InvalidInputData);
        }

        let col = col - 1; // Convert to 0-based index

        let addr = match row {
            1 => col,
            2 => 0x20 + col,
            3 => 0x40 + col,
            4 => 0x60 + col,
            _ => return Err(LcdError::InvalidInputData),
        };

        let mut start = 0x80;
        if self.config.view_mode == ViewMode::Top {
            start = start + 0x04;
        }

        self.send_command(start + addr)?;
        Ok(())
    }

    fn write(&mut self, s: &str) -> Result<(), LcdError<B::Error>> {
        let bytes = s.as_bytes();
        self.send_data(bytes)?;
        Ok(())
    }

    fn set_display(&mut self, flags: DisplayConfig) -> Result<(), LcdError<B::Error>> {
        self.re0_is0_cmd()?;
        self.send_command(CMD_DISPLAY | flags.bits())?;
        Ok(())
    }

    fn set_entry_mode(&mut self, mode: EntryModeSettings) -> Result<(), LcdError<B::Error>> {
        self.re0_is0_cmd()?;
        self.send_command(mode.cmd())?;
        Ok(())
    }

    fn set_view_mode(&mut self, mode: ViewMode) -> Result<(), LcdError<B::Error>> {
        match mode {
            ViewMode::Top => self.ddram_start = 0x84,
            ViewMode::Bottom => self.ddram_start = 0x80,
        }
        self.re1_is0_cmd()?;
        self.send_command(mode as u8)?;
        Ok(())
    }

    fn set_cursor_off(&mut self) -> Result<(), Self::Error> {
        self.config.display_control.remove(DisplayConfig::CURSOR_ON);
        self.send_command(CMD_DISPLAY | self.config.display_control.bits())?;
        Ok(())
    }

    fn set_blinking_off(&mut self) -> Result<(), Self::Error> {
        self.config.display_control.remove(DisplayConfig::BLINK_ON);
        self.send_command(CMD_DISPLAY | self.config.display_control.bits())?;
        Ok(())
    }

    fn extended_function_set(&mut self) -> Result<(), Self::Error> {
        self.re1_is0_cmd()?;
        let cmd = ExtendedFunctionSet::new(
            self.config.font_width,
            self.config.bw_inversion,
            self.config.four_line_enabled,
        )
        .cmd();
        self.send_command(cmd)?;
        Ok(())
    }

    fn set_double_height(&mut self) -> Result<(), Self::Error> {
        if let Some(dh) = self.config.double_height {
            let cmd = match dh {
                DoubleHeight::Lines2 => COMMAND_2LINES,
                DoubleHeight::Lines3Middle => COMMAND_3LINES_MIDDLE,
                DoubleHeight::Lines3Top => COMMAND_3LINES_TOP,
                DoubleHeight::Lines3Bottom => COMMAND_3LINES_BOTTOM,
            };
            self.send_command(0x3A)?;
            self.send_command(cmd)?;
            self.send_command(0x3C)?;
        }
        Ok(())
    }

    fn clear_line(&mut self, line: u8) -> Result<(), Self::Error> {
        if line == 0 || line > 4 {
            return Err(LcdError::InvalidInputData);
        }

        self.locate(line, 1)?;
        for _ in 0..16 {
            self.send_data_byte(b' ')?;
        }
        self.locate(line, 1)?;
        Ok(())
    }

    fn clear_chars(&mut self, row_col: (u8, u8), chars: u8) -> Result<(), Self::Error> {
        let (row, col) = row_col;
        if col == 0 || col > 16 || row == 0 || row > 4 || chars == 0 || chars > 16 {
            return Err(LcdError::InvalidInputData);
        }

        self.locate(row, col)?;
        for _ in 0..chars {
            self.send_data_byte(b' ')?;
        }

        self.locate(row, col)?;
        Ok(())
    }
}
