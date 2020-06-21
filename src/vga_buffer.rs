

#[allow(dead_code)] //抑制 `dead_code` lint
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)] //repr(u8)注记标注的枚举类型，都会以一个u8的形式存储
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

// VGA字符缓冲区字符单元颜色部分的抽象结构
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);

// 注意此函数没有显示使用return
// (background as u8) << 4 | (foreground as u8) back左移四位与fore组合成新的u8
impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}
