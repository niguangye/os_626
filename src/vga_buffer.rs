use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

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
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row-1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize){
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
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

// 支持Rust提供的格式化宏（formatting macros）
impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });

}

#[cfg(test)]
use crate::{serial_print, serial_println};

#[test_case]
fn test_println_simple() {
    serial_print!("test_println... ");
    println!("test_println_simple output");
    serial_println!("[ok]");
}

#[test_case]
fn test_println_many() {
    serial_print!("test_println_many... ");
    for _ in 0..200 {
        println!("test_println_many output");
    }
    serial_println!("[ok]");
}

#[test_case]
fn test_println_output() {

    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    serial_print!("test_println_output... ");

    let s = "Some test string that fits on a single line";
    interrupts::without_interrupts(|| {
        let mut writer = WRITER.lock();
        writeln!(writer, "\n{}", s).expect("writeln failed");
        for (i, c) in s.chars().enumerate() {
            let screen_char = writer.buffer.chars[BUFFER_HEIGHT - 2][i].read();
            assert_eq!(char::from(screen_char.ascii_character), c);
        }
    });

    serial_println!("[ok]");
}








