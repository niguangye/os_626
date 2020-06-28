use volatile::Volatile;

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
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

// 负责将字符写入屏幕的最后一行，并在一行写满或收到换行符\n的时候，将所有字符上移一行
pub struct Writer {
    column_position: usize, // 跟踪光标在最后一行的位置
    color_code: ColorCode, // 颜色模式：backgroundColor+foregroundColor
    buffer: &'static mut Buffer, //VGA字符缓冲区的可变借用, 'static在整个运行期间有效，保证该可变借用内存安全
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        // 注意此处的模式匹配
        match byte {
            // 收到‘\n’时，另起一行
            b'\n' => self.new_line(),
            byte => {
                // 该行写满时另起一行
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_position;
                //利用chars二维数组，向制定位置写入字符byte
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code: self.color_code,
                });
                self.column_position += 1;
            }
        }
    }

    fn new_line(&mut self){
        // TODO
    }

}

impl Writer {
    pub fn write_string(&mut self, s: &str){
        for byte in s.bytes() {
            // 依然是模式匹配
            match byte {
                // 从空格（0x20）到波浪号（0x7e）
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // rust默认支持UTF-8，所以存在VGA缓冲区不支持的字节，此时打印■（0xfe）
                _ => self.write_byte(0xfe),
            }
        }
    }
}

pub fn print_something() {
    let mut writer = Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    };

    writer.write_byte(b'H');
    writer.write_string("ello ");
    writer.write_string("Wörld!");
}










