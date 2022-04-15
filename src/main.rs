#![no_std]
#![no_main]

use arduino_hal::prelude::*;
use arduino_hal::spi;
use embedded_graphics::geometry::Point;
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::DrawTarget;
use embedded_graphics::prelude::OriginDimensions;
use embedded_graphics::prelude::Primitive;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::prelude::Size;
use embedded_graphics::primitives::Circle;
use embedded_graphics::primitives::PrimitiveStyleBuilder;
use embedded_graphics::primitives::StrokeAlignment;
use embedded_graphics::Drawable;
use embedded_graphics::Pixel;

use panic_halt as _;
use smart_leds::SmartLedsWrite;
use smart_leds::RGB8;

/*
 16x16  - WS2812 (neopixels)
*/

struct LedDisplay<Writer>
where
    Writer: SmartLedsWrite<Color = RGB8>,
{
    writer: Writer,
    frame_buffer: [RGB8; 16 * 16],
}

impl<Writer> LedDisplay<Writer>
where
    Writer: SmartLedsWrite<Color = RGB8>,
{
    fn new(writer: Writer) -> Self {
        Self {
            writer,
            frame_buffer: [RGB8::default(); 16 * 16],
        }
    }

    fn flush(&mut self) {
        self.writer.write(self.frame_buffer.iter().cloned()).ok();
    }

    fn index_top_left(&self, point: Point) -> Result<usize, DrawError> {
        if !(0..=15).contains(&point.x) || !(0..=15).contains(&point.y) {
            return Err(DrawError::OutOfBounds);
        }

        let x = point.x as usize;
        let y = point.y as usize;

        let idx = (15 - y) * 16 + if y % 2 == 1 { x } else { 15 - x };
        Ok(idx)
    }
}

impl<Writer> OriginDimensions for LedDisplay<Writer>
where
    Writer: SmartLedsWrite<Color = RGB8>,
{
    fn size(&self) -> Size {
        Size::new(16, 16)
    }
}

enum DrawError {
    OutOfBounds,
}

impl<Writer> DrawTarget for LedDisplay<Writer>
where
    Writer: SmartLedsWrite<Color = RGB8>,
{
    type Color = Rgb888;
    type Error = DrawError;

    fn draw_iter<I>(&mut self, data: I) -> Result<(), DrawError>
    where
        I: IntoIterator<Item = Pixel<Rgb888>>,
    {
        for Pixel(point, color) in data {
            let idx = self.index_top_left(point)?;
            self.frame_buffer[idx] = [color.r(), color.g(), color.b()].into();
        }

        Ok(())
    }
}

#[arduino_hal::entry]
fn main() -> ! {
    let dp = arduino_hal::Peripherals::take().unwrap();
    let pins = arduino_hal::pins!(dp);
    let mut serial = arduino_hal::default_serial!(dp, pins, 57600);

    ufmt::uwriteln!(&mut serial, "Hello from Arduino!\r").void_unwrap();

    let (spi, _) = arduino_hal::Spi::new(
        dp.SPI,
        pins.d13.into_output(),
        pins.d11.into_output(),
        pins.d12.into_pull_up_input(),
        pins.d10.into_output(),
        spi::Settings::default(),
    );

    //let mut leds = ws2812_spi::Ws2812::new(spi);
    let leds = ws2812_blocking_spi::Ws2812BlockingWriter::new(spi);

    let mut display = LedDisplay::new(leds);

    let circle_style = PrimitiveStyleBuilder::new()
        .fill_color(Rgb888::new(0, 10, 10))
        .stroke_color(Rgb888::new(5, 0, 0))
        .stroke_width(1)
        .stroke_alignment(StrokeAlignment::Inside)
        .build();

    let mut circle_x = 1;
    let mut circle_move = 1;

    loop {
        if circle_x <= 1 {
            circle_move = 1;
        } else if circle_x >= 10 {
            circle_move = -1;
        }
        circle_x += circle_move;

        // display something
        display.clear(Rgb888::BLACK).ok();

        Circle::new(Point::new(circle_x, 4), 5)
            .into_styled(circle_style)
            .draw(&mut display)
            .ok();

        display.flush();

        arduino_hal::delay_ms(100);
    }
}
