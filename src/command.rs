//! The command set for the SSD1322.
//!
//! Note 1: The display RAM of the SSD1322 is arranged in 128 rows and 120 columns, where each
//! column is 4 adjacent pixels (segments) in the row for a total max resolution of 128x480. Each
//! pixel is 4 bits/16 levels of intensity, so each column also refers to two adjacent bytes. Thus,
//! anywhere there is a "column" address, these refer to horizontal groups of 2 bytes driving 4
//! pixels.

use interface::DisplayInterface;

/// The address increment orientation when writing image data.
pub enum IncrementAxis {
    /// The column address will increment as image data is written, writing pairs of bytes
    /// (horizontal groups of 4 pixels) from left to right in the range set by `SetColumnAddress`
    /// command, and then top to bottom in the range set by `SetRowAddress` command.
    Horizontal,
    /// The row address will increment as image data is written, writing pairs of bytes
    /// (*horizontal* groups of 4 pixels) from top to bottom in the range set by `SetRowAddress`
    /// command, and then left to right in the range set by `SetColumnAddress` command.
    Vertical,
}

/// Setting of column address remapping.
pub enum ColumnRemap {
    /// Column addresses 0->119 map to segments 0,1,2,3->476,477,478,479.
    Forward,
    /// Column addresses 0->119 map to segments 476,477,478,479->0,1,2,3. Note that the pixels
    /// within each column number in the same order; `NibbleRemap` controls the order of mapping
    /// pixels to nibbles within each column.
    Reverse,
}

/// Setting of data nibble remapping.
pub enum NibbleRemap {
    /// 2-byte sequence 0xABCD maps (in L->R order) to pixels 3,2,1,0.
    Reverse,
    /// 2-byte sequence 0xABCD maps (in L->R order) to pixels 0,1,2,3.
    Forward,
}

/// Setting of the COM line scanning of rows. Changing this setting will flip the image vertically.
pub enum ComScanDirection {
    /// COM lines scan row addresses top to bottom, so that row address 0 is the first row of the
    /// display.
    RowZeroFirst,
    /// COM lines scan row addresses bottom to top, so that row address 0 is the last row of the
    /// display.
    RowZeroLast,
}

/// Setting the layout of the COM lines to the display rows. This setting is dictated by how the
/// display module itself wires the OLED matrix to the driver chip, and changing it to anything
/// other than the correct setting for your module will yield a corrupted image. See the display
/// module datasheet for the correct value to use.
pub enum ComLayout {
    /// COM lines are connected to display rows in a progressive arrangement, so that COM lines
    /// 0->127 map to display rows 0->127.
    Progressive,
    /// COM lines are connected to display rows in an interlaced arrangement, so that COM lines
    /// 0->63 map to *even* display rows 0->126, and COM lines 64->127 map to *odd* display rows
    /// 1->127.
    Interlaced,
    /// COM lines are connected to display rows in a dual-COM progressive arrangement, so that COM
    /// lines 0->63 map to display rows 0->63 for half of the columns, and COM lines 64->127 map to
    /// display rows 0->63 for the other half. The maximum displayable image size for this
    /// configuration is halved to 480x64 because each display row uses two COM lines.
    DualProgressive,
}

/// Setting of the display mode.
pub enum DisplayMode {
    /// The display is blanked with all pixels turned OFF (to grayscale level 0).
    BlankDark,
    /// The display is blanked with all pixels turned ON (to grayscale level 15).
    BlankBright,
    /// The display operates normally, showing the image in the display RAM.
    Normal,
    /// The display operates with inverse brightness, showing the image in the display RAM with the
    /// grayscale levels inverted (level 0->15, 1->14, ..., 15->0).
    Inverse,
}

pub enum Command<'buf> {
    /// Enable the gray scale gamma table (see SetGrayScaleTable).
    EnableGrayScaleTable,
    /// Set the column start and end address range when writing to the display RAM. The column
    /// address pointer is reset to the start column address such that `WriteImageData` will begin
    /// writing there. Range is 0-119. (Note 1)
    SetColumnAddress(u8, u8),
    /// Set the row start and end address range when writing to the display RAM. The row address
    /// pointer is reset to the start row address such that `WriteImageData` will begin writing
    /// there. Range is 0-127.
    SetRowAddress(u8, u8),
    /// Set the direction of display address increment, column address remapping, data nibble
    /// remapping, COM scan direction, and COM line layout. See documentation for each enum for
    /// details.
    SetRemapping(
        IncrementAxis,
        ColumnRemap,
        NibbleRemap,
        ComScanDirection,
        ComLayout,
    ),
    /// Write image data into display RAM. The image data will be written to the display RAM in the
    /// order specified by `SetRemapping` `IncrementAxis` setting. The data, once written, will be
    /// mapped onto the display pixels in a manner determined by `SetRemapping` `ColumnRemap`,
    /// `NibbleRemap`, `ComScanDirection`, and `ComLayout` settings.
    WriteImageData(&'buf [u8]),
    /// Set the display start line. Setting this to e.g. 40 will cause the first row of pixels on
    /// the display to display row 40 or the display RAM, and rows 0-39 of the display RAM will be
    /// wrapped to the bottom, "rolling" the displayed image upwards.  This transformation is
    /// applied *before* the MUX ratio setting, meaning if the MUX ratio is set to 90, display rows
    /// 0->89 will always be active, and the "rolled" image will be rendered within those display
    /// rows. Range is 0-127.
    SetStartLine(u8),
    /// Set the display COM line offset. This has a similar effect to `SetStartLine`, rolling the
    /// displayed image upwards as the values increase, except that it is applied *after* the MUX
    /// ratio setting. This means both the image *and* the display rows seleced by the MUX ratio
    /// setting will be rolled upwards. Range is 0-127.
    SetDisplayOffset(u8),
    /// Set the display operating mode. See enum for details.
    SetDisplayMode(DisplayMode),
    /// Enable partial display mode. This selects a range of rows in the display area which will be
    /// active, while all others remain inactive. Range is 0-127, where start must be <= end.
    EnablePartialDisplay(u8, u8),
    /// Disable partial display mode.
    DisablePartialDisplay,
    /// Control sleep mode.
    SetSleepMode(bool),
    /// Set the refresh phase lengths. The first phase (reset) can be set from 5-31 DCLKs, and the
    /// second (first pre-charge) can be set from 3-15 DCLKs. The display module datasheet should
    /// have appropriate values.
    SetPhaseLengths(u8, u8),
    /// Set the oscillator frequency Fosc and the display clock divider. The relationship between
    /// the frequency settings 0-15 and the actual Fosc value is not documented, except that higher
    /// values increase the frequency. The divider DIVSET is a value n from 0-10, where DCLK is
    /// produced by dividing Fosc by 2^n. The resulting DCLK rate indirectly determines the refresh
    /// rate of the display (the exact rate depends on the MUX ratio and some other things).
    SetClockFoscDivset(u8, u8),
    /// Enable or disable display enhancements "external VSL" and "Enhanced low GS display
    /// quality".
    SetDisplayEnhancements(bool, bool),
    /// Set the second pre-charge period. Range 0-15 DCLKs.
    SetSecondPrechargePeriod(u8),
    /// Set the gray scale gamma table. Each byte 0-14 can range from 0-180 and sets the pixel
    /// drive pulse width in DCLKs. Bytes 0->14 adjust the gamma setting for grayscale levels
    /// 1->15; grayscale level 0 cannot be modified. The gamma settings must monotonically
    /// increase.
    SetGrayScaleTable([u8; 15]),
    /// Set the gray scale gamma table to the factory default.
    SetDefaultGrayScaleTable,
    /// Set the pre-charge voltage level, from 0.2*Vcc to 0.6*Vcc. Range 0-31.
    SetPreChargeVoltage(u8),
    /// Set the COM deselect voltage level, from 0.72*Vcc to 0.86*Vcc. Range 0-7.
    SetComDeselectVoltage(u8),
    /// Set the contrast current. Range 0-255.
    SetContrastCurrent(u8),
    /// Set the master contrast control, uniformly reducing all grayscale levels by 0-15
    /// sixteenths. Range 0 (maximum dimming) to 15 (normal contrast).
    SetMasterContrast(u8),
    /// Set the MUX ratio, which controls the number of COM lines that are active and thus the
    /// number of display pixel rows which are active. Which COM lines are active is controlled by
    /// `SetDisplayOffset`, and how the COM lines map to display RAM row addresses is controlled by
    /// `SetStartLine`. Range 16-128.
    SetMuxRatio(u8),
    /// Set whether the command lock is enabled or disabled. Enabling the command lock blocks all
    /// commands except `SetCommandLock`.
    SetCommandLock(bool),
}
