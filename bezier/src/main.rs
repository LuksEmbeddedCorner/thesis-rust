#![cfg_attr(not(test), no_std)]
#![no_main]
#![feature(alloc_error_handler)]
#![feature(asm)]

extern crate alloc;

use alloc::vec::Vec;
use core::{alloc::Layout, ops::*};
use cstr_core::cstr;
use semihosting_files::{File, FileOpenMode};
use stopwatch::StopWatch;

use alloc_cortex_m::CortexMHeap;
use cortex_m::{asm, peripheral::DWT};
use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use panic_halt as _;

// Needed for linking
#[cfg(not(feature = "use_fpu"))]
#[allow(unused_imports)]
use stm32f2::stm32f217 as _;
#[cfg(feature = "use_fpu")]
#[allow(unused_imports)]
use stm32f4::stm32f429 as _;

#[cfg(not(feature = "use_fpu"))]
mod fixed_point;
#[cfg(not(feature = "use_fpu"))]
use fixed_point::FixedPoint as Scalar;
#[cfg(feature = "use_fpu")]
mod floating_point;
#[cfg(feature = "use_fpu")]
use f32 as Scalar;
#[cfg(feature = "use_fpu")]
use floating_point::FloatExt;

mod stackpaint;
mod stopwatch;

// this is the allocator the application will use
#[global_allocator]
static ALLOCATOR: CortexMHeap = CortexMHeap::empty();

#[derive(Debug, Default, PartialEq, Clone)]
struct Point {
    x: Scalar,
    y: Scalar,
}

const PARM_STEPS: i32 = 10;

#[derive(Debug, Default, PartialEq, Clone)]
struct Curve {
    // First endpoint
    p1: Point,
    // First control point
    c1: Point,
    // Second control point
    c2: Point,
    // Second endpoint
    p2: Point,
}

trait ScalarLike: Add + Mul + Sub + Div + Sized {
    fn from_decimal(sign: i32, integer: u32, fraction: u32, dec: u32) -> Self;

    fn from_i32(value: i32) -> Self;

    fn from_step(step: i32, total: i32) -> Self;

    fn parametric(t: Self, a: Self, b: Self, c: Self, d: Self) -> Self;

    type Primitive;

    fn into_primitive(self) -> Self::Primitive;
}

impl Curve {
    // This function gets inlined
    pub fn parametric(&self, t: Scalar) -> Point {
        hprintln!("Hallo").unwrap();
        Point {
            x: Scalar::parametric(t, self.p1.x, self.c1.x, self.c2.x, self.p2.x),
            y: Scalar::parametric(t, self.p1.y, self.c1.y, self.c2.y, self.p2.y),
        }
    }
}

impl AddAssign for Point {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

// the slices are expected to have the same length
// interpolate_test is an output parameter
// It assures that the operation is not just optimized away entirely
fn interpolate_points(curves: &[Curve], interpolate_test: &mut [Point]) {
    for (curve, interpolate_test) in curves.iter().zip(interpolate_test) {
        *interpolate_test = Point::default();

        // inner loop - gets unrolled (PARM_STEPS is const)
        for step in 0..PARM_STEPS {
            // t is calculated at compile time
            let t = Scalar::from_step(step, PARM_STEPS);
            let point = curve.parametric(t);

            // dbg!(t.into_primitive(), point.x.into_primitive(), point.y.into_primitive());

            *interpolate_test += point;
        }
    }
}

#[inline(never)] // to stop buffer from messing up the stack
fn read_data_points() -> (Vec<Curve>, usize, Vec<Point>) {
    let mut buffer = [0; 55000];

    hprintln!("Data Set loading").unwrap();

    let mut file = File::open(
        cstr!("./examples/bezier/test-files/bezdata1.txt"),
        FileOpenMode::Read,
    )
    .expect("Could not open file");

    let filelength = file.read(&mut buffer).expect("Could not read from file");
    let data = core::str::from_utf8(&buffer[..filelength]).expect("file content is not valid utf8");

    let mut points = data
        .lines()
        .take_while(|line| !line.starts_with('!'))
        .map(parse_line);

    let mut result = Vec::new();

    while let Some(p1) = points.next() {
        let c1 = points
            .next()
            .expect("Number of points cannot be divided by 4");
        let c2 = points
            .next()
            .expect("Number of points cannot be divided by 4");
        let p2 = points
            .next()
            .expect("Number of points cannot be divided by 4");

        //hprintln!("{:?} {:?} {:?} {:?}", p1, c1, c2, p2).unwrap();

        result.push(Curve { p1, c1, c2, p2 })
    }

    let test_index = data
        .lines()
        .find(|line| line.starts_with('!'))
        .expect("No line with !")
        .chars()
        .nth(1)
        .expect("No char after !")
        .to_digit(10)
        .expect("Character not a digit");

    let golden_points: Vec<_> = data
        .lines()
        .skip_while(|line| !line.starts_with('!'))
        .skip(1) // skip the line with the !
        .map(parse_line)
        .collect();

    hprintln!("Data Set loaded").unwrap();

    (result, test_index as usize, golden_points)
}

fn parse_line(line: &str) -> Point {
    let (x_string, y_string) = line.split_once(',').expect("Line must be a valid point");

    let x = parse_scalar(x_string);
    let y = parse_scalar(y_string);

    Point { x, y }
}

fn parse_scalar(chars: &str) -> Scalar {
    let mut chars = chars.trim_start().chars().peekable();

    let sign = if chars.next_if_eq(&'-').is_some() {
        -1
    } else {
        1
    };

    let mut found_dec = false;
    let mut integer = 0;
    let mut fraction = 0;
    let mut dec = 1;

    for char in chars {
        if let Some(digit) = char.to_digit(10) {
            if found_dec {
                fraction = fraction * 10 + digit;
                dec *= 10;
            } else {
                integer = integer * 10 + digit;
            }
        } else if char == '.' {
            found_dec = true;
        } else {
            break;
        }
    }

    Scalar::from_decimal(sign, integer, fraction, dec)
}

fn run_test(iterations: usize, dwt: &DWT) {
    #[allow(unused_variables)]
    let (curves, test_index, golden_points) = read_data_points();
    let mut test_points = alloc::vec![Point::default(); curves.len()];

    let mut dummy_sum = Scalar::from_i32(0);

    hprintln!("Starting test").unwrap();
    let stack_measurer = stackpaint::paint();
    let watch = StopWatch::start(dwt);

    // Do the calculations
    for _ in 0..iterations {
        interpolate_points(&curves, &mut test_points);

        let test_point = &test_points[test_index];
        dummy_sum += test_point.x + test_point.y;
    }

    // stop time measurement and print result
    let result = watch.stop(dwt);

    let used_stack = stack_measurer.get();

    hprintln!(
        "Result: cycles {} dummy {}",
        result,
        dummy_sum.into_primitive(),
    )
    .unwrap();

    hprintln!("Used Stack: {}", used_stack).unwrap();

    #[cfg(not(feature = "use_fpu"))]
    {
        let mut crc = 0;

        for test_point in test_points {
            crc = calc_crc32(test_point.x.into_primitive() as u32, crc);
            crc = calc_crc32(test_point.y.into_primitive() as u32, crc);
        }

        hprintln!("crc: {:#06x}", crc).unwrap();
    }

    #[cfg(feature = "use_fpu")]
    {
        let mut max_error = 0f32;
        const ERROR_EPSILON: f32 = 0.000002;

        for (test_point, golden_point) in test_points.iter().zip(&golden_points) {
            let error = (test_point.x - golden_point.x).abs();
            if error > max_error {
                max_error = error;
            }
            let error = (test_point.y - golden_point.y).abs();

            if error > max_error {
                max_error = error;
            }
        }

        if max_error > ERROR_EPSILON {
            hprintln!("ERROR: Self test failed! Error of {} detected.", max_error).unwrap();
        } else {
            hprintln!("Error small enough").unwrap();
        }
    }
}

#[cfg(not(feature = "use_fpu"))]
fn calc_crc32(data: u32, mut crc: u16) -> u16 {
    crc = calc_crc8(data as u8, crc);
    crc = calc_crc8((data >> 8) as u8, crc);
    crc = calc_crc8((data >> 16) as u8, crc);
    crc = calc_crc8((data >> 24) as u8, crc);

    // data.to_ne_bytes().iter().cloned().fold(crc, |a, b| calc_crc8(b, a));

    crc
}

#[cfg(not(feature = "use_fpu"))]
fn calc_crc8(mut data: u8, mut crc: u16) -> u16 {
    let mut x16: u8;
    let mut carry: u8;

    for _i in 0..8 {
        x16 = (data & 1) ^ ((crc & 1) as u8);
        data >>= 1;

        if x16 == 1 {
            crc ^= 0x4002;
            carry = 1;
        } else {
            carry = 0;
        }

        crc >>= 1;
        if carry != 0 {
            crc |= 0x8000;
        } else {
            crc &= 0x7fff;
        }
    }

    crc
}

const HEAP_SIZE: usize = 1 << 16; // in bytes

#[entry]
fn main() -> ! {
    // Initialize the allocator
    unsafe { ALLOCATOR.init(cortex_m_rt::heap_start() as usize, HEAP_SIZE) }

    let mut peripheral = cortex_m::Peripherals::take().unwrap();
    let dwt = &mut peripheral.DWT;

    // Make sure the stopwatches work
    StopWatch::init(dwt);

    run_test(10, dwt);

    loop {
        asm::nop()
    }
}

#[alloc_error_handler]
fn alloc_error(_layout: Layout) -> ! {
    asm::bkpt();

    loop {
        asm::nop()
    }
}
