use hashbag::HashBag;
use image::io::Reader as ImageReader;
use image::{GenericImageView, Rgba};
use std::error::Error;
use wfc::patterns::Pattern;
use wfc::tiles::Tile;
use wfc::Wave;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct Pixel(Rgba<u8>);

impl Default for Pixel {
    fn default() -> Self {
        Self(Rgba(Default::default()))
    }
}

fn save_patterns(
    hashbag: &HashBag<Pattern<Pixel>>,
    n: u32,
    col_count: u32,
) -> Result<(), Box<dyn Error>> {
    let n_padding = n + 2;
    let mut img = image::RgbaImage::new(
        col_count * n_padding,
        ((hashbag.set_len() as u32 / col_count) + 1) * n_padding,
    );
    for (i, (pattern, _)) in hashbag.set_iter().enumerate() {
        let pattern_x = (i as u32 % col_count) * n_padding + 1;
        let pattern_y = (i as u32 / col_count) * n_padding + 1;

        for (j, pixel) in pattern.data().iter().enumerate() {
            let x = pattern_x + j as u32 % n;
            let y = pattern_y + j as u32 / n;

            img.put_pixel(x, y, pixel.0);
        }
    }

    img.save("debug/patterns.png")?;

    Ok(())
}

fn main() -> Result<(), Box<dyn Error>> {
    let image_data = ImageReader::open("flowers.png")?.decode()?;
    let n = 3u32;
    let x_cells = 20;
    let y_cells = 20;

    let mut patterns = HashBag::new();

    for x in 0..image_data.width() {
        for y in 0..image_data.height() {
            let mut pixel_data = Vec::with_capacity((n * n) as usize);
            for i in 0..n {
                for j in 0..n {
                    let pixel = image_data
                        .get_pixel((x + i) % image_data.width(), (y + j) % image_data.height());

                    pixel_data.push(Pixel(pixel));
                }
            }

            let pattern = Pattern::new(pixel_data.into_boxed_slice());
            for permutation in pattern.all_permutations() {
                patterns.insert(permutation);
            }
        }
    }

    save_patterns(&patterns, n, 15)?;

    let pattern_count = patterns.len() as f64;

    let tiles: Box<_> = patterns
        .into_iter()
        .enumerate()
        .map(|(id, (pattern, frequency))| {
            Tile::new(pattern, (frequency as f64) / pattern_count, id)
        })
        .collect();

    let mut wave = Wave::new(&tiles, x_cells, y_cells, n as usize);
    while !wave.collapsed() {
        wave.collapse()?;
    }

    let image = wave.to_image(|remaining_tiles| {
        let mut iter = remaining_tiles.into_iter().cloned();

        // let mut base: Box<_> = iter
        //     .next()
        //     .unwrap()
        //     .into_iter()
        //     .map(|p| {
        //         let raw = p.0 .0;
        //         [
        //             raw[0] as usize,
        //             raw[1] as usize,
        //             raw[2] as usize,
        //             raw[3] as usize,
        //         ]
        //     })
        //     .collect();

        // iter.for_each(|tile| {
        //     tile.into_iter().zip(base.iter_mut()).for_each(|(p, b)| {
        //         let raw_p = p.0 .0;

        //         b[0] += raw_p[0] as usize;
        //         b[1] += raw_p[1] as usize;
        //         b[2] += raw_p[2] as usize;
        //         b[3] += raw_p[3] as usize;
        //     });
        // });

        // base.into_iter()
        //     .map(|p| {
        //         let count = remaining_tiles.len();
        //         Rgba([
        //             (p[0] / count) as u8,
        //             (p[1] / count) as u8,
        //             (p[2] / count) as u8,
        //             (p[3] / count) as u8,
        //         ])
        //     })
        //     .collect()

        iter.next().unwrap().into_iter().map(|p| p.0).collect()
    });

    image.save("debug/debug.png")?;

    Ok(())
}
