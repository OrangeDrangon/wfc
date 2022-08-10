use hashbag::HashBag;
use image::io::Reader as ImageReader;
use image::GenericImageView;
use std::error::Error;
use wfc::patterns::Pattern;
use wfc::tiles::Tile;
use wfc::Wave;

fn main() -> Result<(), Box<dyn Error>> {
    let image_data = ImageReader::open("flowers.png")?.decode()?;
    let n = 3u32;
    let x_cells = 100;
    let y_cells = 100;

    let mut patterns = HashBag::new();

    for x in 0..image_data.width() {
        for y in 0..image_data.height() {
            let mut pixel_data = Vec::with_capacity((n * n) as usize);
            for i in 0..n {
                for j in 0..n {
                    pixel_data.push(
                        image_data
                            .get_pixel((x + i) % image_data.width(), (y + j) % image_data.height())
                            .0,
                    );
                }
            }

            let pattern = Pattern::new(pixel_data);
            for permutation in pattern.all_permutations() {
                patterns.insert(permutation);
            }
        }
    }

    let pattern_count = patterns.len() as f64;

    let tiles: Vec<_> = patterns
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

    Ok(())
}
