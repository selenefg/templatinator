use image::{DynamicImage, GenericImage, GenericImageView, Pixel, imageops, imageops::overlay_bounds, io::Reader};
use rayon::prelude::*;
use std::{fs, u32};

const OUTPUT_DIR: &'static str = "resultado";
struct Template {
    image: DynamicImage,
    transparency_top_left: (u32, u32),
    transparency_bottom_right: (u32, u32),
}

//struct Kid {
//    image: ImageBuffer,
//    top_left: (u32, u32);
//    bottom_right: (u32, u32);
//}

pub fn inverse_overlay<I, J>(bottom: &mut I, top: &J, x: u32, y: u32)
where
    I: GenericImage,
    J: GenericImageView<Pixel = I::Pixel>,
{
    let bottom_dims = bottom.dimensions();
    let top_dims = top.dimensions();

    // Crop our top image if we're going out of bounds
    let (range_width, range_height) = overlay_bounds(bottom_dims, top_dims, x, y);

    for top_y in 0..range_height {
        for top_x in 0..range_width {
            let mut p = top.get_pixel(top_x, top_y);
            let bottom_pixel = bottom.get_pixel(x + top_x, y + top_y);
            p.blend(&bottom_pixel);

            bottom.put_pixel(x + top_x, y + top_y, p);
        }
    }
}

fn process_kid(kid: &mut DynamicImage, template: &mut Template) {
    *kid = kid.resize_to_fill(
        template.transparency_bottom_right.0.saturating_sub(template.transparency_top_left.0),
        template.transparency_bottom_right.1.saturating_sub(template.transparency_top_left.1),
        imageops::FilterType::Gaussian,
    );

    inverse_overlay(
        &mut template.image,
        kid,
        template.transparency_top_left.0,
        template.transparency_top_left.1,
    );
}

fn calculate_transp(image: &DynamicImage) -> ((u32, u32), (u32, u32)) {
    let (mut min_x, mut min_y) = (99999, 99999);
    let (mut max_x, mut max_y) = (0, 0);
    for (x, y, rgba) in image.pixels()
    {
        let alpha = rgba.channels()[3];
        if alpha != 255 
        {
            if x > max_x { max_x = x; }
            if x < min_x { min_x = x; }
            if y > max_y { max_y = y; }
            if y < min_y { min_y = y; }
        }
    }

    return((min_x, min_y), (max_x, max_y));
}

fn main() {
    fs::create_dir_all(OUTPUT_DIR).unwrap();

    let kid_entries: Vec<_> = fs::read_dir("kids/")
        .unwrap()
        .filter_map(Result::ok)
        .filter(|f| f.path().extension().map(|e| e.to_str().unwrap()) == Some("jpg"))
        .collect();
    
    let template_entries: Vec<_> = fs::read_dir("templates/")
        .unwrap()
        .filter_map(Result::ok)
        .filter(|f| f.path().extension().map(|e| e.to_str().unwrap()) == Some("png"))
        .collect();

    kid_entries.par_iter().for_each(|kid_entry| {
        template_entries.par_iter().for_each(|template_entry| {
            let image = Reader::open(template_entry.path()).expect("Open template image").decode().unwrap();
            let (transparency_top_left, transparency_bottom_right) = calculate_transp(&image);
            let mut template = Template {
                image,
                transparency_top_left,
                transparency_bottom_right,
            };
            let mut kid = Reader::open(kid_entry.path()).expect("Open kid image").decode().unwrap();
            process_kid(&mut kid, &mut template);
            template
                .image
                .save(
                    dbg!(format!(
                        "{}/{}_{}",
                        OUTPUT_DIR,
                        template_entry.path().file_name().unwrap().to_str().unwrap(),
                        kid_entry.path().file_name().unwrap().to_str().unwrap()
                    ))
                    .as_str(),
                )
                .unwrap();
        })
    })
}
