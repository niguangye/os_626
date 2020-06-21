

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

// VGA字符缓冲区字符单元的抽象结构，共16位
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)] //按C语言约定的顺序布局它的成员变量，让我们能正确地映射内存片段
struct ScreenChar {
    ascii_character: u8, //字符部分 0-7
    color_code: ColorCode, // 颜色部分 8-15
}
// 对应VGA模式25行
const BUFFER_HEIGHT: usize = 25;
// 对应VGA模式80列
const BUFFER_WIDTH: usize = 80;

// 字符缓冲区抽象
// 双层数组，第一层数组的元素为[ScreenChar; BUFFER_WIDTH]，长度为 BUFFER_HEIGHT
// 第二层数组（[ScreenChar; BUFFER_WIDTH]）的元素为ScreenChar(16位），长度为BUFFER_WIDTH
// 我们再次使用repr(transparent)，来确保类型和它的单个成员有相同的内存布局
// 此时Buffer即从某个地址起始的 16x80x25 bit 长度的内存，chars正好映射该段内存
#[repr(transparent)]
struct Buffer {
    chars: [[ScreenChar; BUFFER_WIDTH]; BUFFER_HEIGHT],
}










