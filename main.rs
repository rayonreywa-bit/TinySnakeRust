#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[link(name = "user32")]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetStdHandle(n: i32) -> isize;
    fn WriteConsoleA(h: isize, b: *const u8, l: u32, w: *mut u32, r: *mut u8) -> i32;
    fn GetAsyncKeyState(v: i32) -> i16;
    fn Sleep(m: u32);
    fn ExitProcess(c: u32) -> !;
}

#[derive(Copy, Clone, PartialEq)]
struct Point { x: i8, y: i8 }

const W: usize = 20;
const H: usize = 15;

#[unsafe(no_mangle)]
pub unsafe extern "system" fn mainCRTStartup() -> ! {
    let stdout = GetStdHandle(-11);
    let mut snake = [Point { x: 0, y: 0 }; 64];
    let mut len = 1usize;
    snake[0] = Point { x: 10, y: 7 };
    
    let mut food = Point { x: 5, y: 5 };
    let mut seed = 12345u32;
    let (mut dx, mut dy) = (1i8, 0i8);
    let mut dummy = 0u32;

    loop {
        if GetAsyncKeyState(0x51) < 0 { ExitProcess(0); }
        if GetAsyncKeyState(0x57) < 0 && dy == 0 { dx = 0; dy = -1; }
        if GetAsyncKeyState(0x53) < 0 && dy == 0 { dx = 0; dy = 1; }
        if GetAsyncKeyState(0x41) < 0 && dx == 0 { dx = -1; dy = 0; }
        if GetAsyncKeyState(0x44) < 0 && dx == 0 { dx = 1; dy = 0; }

        let nx = snake[0].x + dx;
        let ny = snake[0].y + dy;

        if nx <= 0 || nx >= W as i8 - 1 || ny <= 0 || ny >= H as i8 - 1 { break; }
        
        // تحريك الأفعى باستخدام مؤشرات خام لتجنب Bounds Check
        for i in (1..len).rev() {
            *snake.as_mut_ptr().add(i) = *snake.as_ptr().add(i - 1);
        }
        snake[0] = Point { x: nx, y: ny };

        if nx == food.x && ny == food.y {
            if len < 64 { len += 1; }
            seed = seed.wrapping_mul(214013).wrapping_add(2531011);
            food.x = (seed as i8 % (W as i8 - 2)).abs() + 1;
            food.y = (seed as i8 % (H as i8 - 2)).abs() + 1;
        }

        let mut buf = [b' '; 3 + (W + 1) * H]; 
        buf[0] = 0x1b; buf[1] = b'['; buf[2] = b'H';
        
        for y in 0..H {
            for x in 0..W {
                let p = 3 + y * (W + 1) + x;
                if y == 0 || y == H - 1 || x == 0 || x == W - 1 { 
                    *buf.as_mut_ptr().add(p) = b'#'; 
                }
                else if x as i8 == food.x && y as i8 == food.y { 
                    *buf.as_mut_ptr().add(p) = b'*'; 
                }
            }
            *buf.as_mut_ptr().add(3 + y * (W + 1) + W) = b'\n';
        }
        
        for i in 0..len {
            let s = *snake.as_ptr().add(i);
            let p = 3 + s.y as usize * (W + 1) + s.x as usize;
            if p < buf.len() {
                *buf.as_mut_ptr().add(p) = if i == 0 { b'O' } else { b'o' };
            }
        }

        WriteConsoleA(stdout, buf.as_ptr(), buf.len() as u32, &mut dummy, core::ptr::null_mut());
        Sleep(100);
    }
    ExitProcess(0);
}

// توفير الدوال التي قد يطلبها المترجم بشكل فارغ لتوفير المساحة
#[unsafe(no_mangle)]
pub unsafe extern "C" fn memset(d: *mut u8, c: i32, n: usize) -> *mut u8 {
    for i in 0..n { *d.add(i) = c as u8; }
    d
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! { unsafe { ExitProcess(1) } }