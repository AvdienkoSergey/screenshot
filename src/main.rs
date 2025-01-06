use screenshots::Screen;
use std::{env, thread, time::Duration};
use std::fs::create_dir_all;
use std::time::{SystemTime, UNIX_EPOCH};
use std::path::PathBuf;
use std::fs::File;
use image::codecs::jpeg::JpegEncoder;
use image::{DynamicImage, RgbaImage};
use winapi::um::wincon::FreeConsole;
use chrono::prelude::*;

fn main() {
    unsafe {
        FreeConsole();
    }

    // Чтение временного интервала из аргументов командной строки
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Использование: программа <start_time> <end_time> (формат: HH:MM)");
        return;
    }

    let start_time = NaiveTime::parse_from_str(&args[1], "%H:%M").expect("Неверный формат времени начала");
    let end_time = NaiveTime::parse_from_str(&args[2], "%H:%M").expect("Неверный формат времени окончания");

    let output_dir = "./screenshots"; // Директория для сохранения скриншотов
    create_dir_all(output_dir).expect("Не удалось создать директорию для скриншотов");

    loop {
        let now = Local::now().time();

        // Проверка, находится ли текущее время в заданном диапазоне
        if now < start_time || now > end_time {
            // println!("Вне рабочего времени. Пропуск действия.");
            thread::sleep(Duration::from_secs(60)); // Проверка каждую минуту
            continue;
        }

        let screens = Screen::all().unwrap();

        let mut total_width = 0;
        let mut max_height = 0;

        for screen in &screens {
            total_width += screen.display_info.width;
            max_height = max_height.max(screen.display_info.height);
        }

        let mut combined_image = RgbaImage::new(total_width, max_height);

        let mut current_x = 0;
        for screen in screens {
            let frame: screenshots::image::ImageBuffer<screenshots::image::Rgba<u8>, Vec<u8>> = screen.capture().unwrap();

            let buffer = RgbaImage::from_raw(frame.width(), frame.height(), frame.clone().into_raw())
                .expect("Ошибка при создании ImageBuffer");

            let dynamic_image = DynamicImage::ImageRgba8(buffer);
            overlay(&mut combined_image, &dynamic_image.to_rgba8(), current_x, 0);

            current_x += screen.display_info.width;
        }

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let combined_image_path = format!("{}/{}.jpg", output_dir, timestamp);
        let path = PathBuf::from(&combined_image_path);

        let file = File::create(&path).expect("Не удалось создать файл для сохранения");
        let mut encoder = JpegEncoder::new_with_quality(file, 30);

        encoder
            .encode_image(&DynamicImage::ImageRgba8(combined_image))
            .expect("Не удалось сохранить изображение");

        thread::sleep(Duration::from_secs(1)); // Интервал скриншотов
    }
}

fn overlay(destination: &mut RgbaImage, source: &RgbaImage, x_offset: u32, y_offset: u32) {
    for y in 0..source.height() {
        for x in 0..source.width() {
            let src_pixel = source.get_pixel(x, y);
            if x + x_offset < destination.width() && y + y_offset < destination.height() {
                let dest_pixel = destination.get_pixel_mut(x + x_offset, y + y_offset);
                *dest_pixel = *src_pixel;
            }
        }
    }
}
