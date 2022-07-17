use hashbag::HashBag;
use image::io::Reader as ImageReader;
use image::GenericImageView;
use std::error::Error;
use wfc::{Pattern, PatternAdjacencyMap, Tile, Wave};

fn main() -> Result<(), Box<dyn Error>> {
    let image = ImageReader::open("flowers.png")?.decode()?;
    let n = 3u32;
    let x_cells = 3usize;
    let y_cells = 3usize;

    let mut patterns = HashBag::new();

    for x in 0..image.width() {
        for y in 0..image.height() {
            let mut pixel_data = Vec::with_capacity((n * n) as usize);
            for i in 0..n {
                for j in 0..n {
                    pixel_data.push(
                        image
                            .get_pixel((x + i) % image.width(), (y + j) % image.height())
                            .0,
                    );
                }
            }

            let pattern = Pattern::new(pixel_data);
            let reflected = pattern.reflect();
            let rotated = pattern.rotate();
            let rotated_reflected = rotated.reflect();
            let rotated_rotated = rotated.rotate();
            let rotated_rotated_reflected = rotated_rotated.reflect();
            let rotated_rotated_rotated = rotated_rotated.rotate();
            let rotated_rotated_rotated_reflected = rotated_rotated_rotated.reflect();

            patterns.insert(pattern);
            patterns.insert(reflected);
            patterns.insert(rotated);
            patterns.insert(rotated_reflected);
            patterns.insert(rotated_rotated);
            patterns.insert(rotated_rotated_reflected);
            patterns.insert(rotated_rotated_rotated);
            patterns.insert(rotated_rotated_rotated_reflected);
        }
    }

    let patterns: Vec<_> = patterns
        .set_iter()
        .map(|(pattern, frequency)| (pattern, (frequency as f64) / (patterns.len() as f64)))
        .collect();

    let adjacency = PatternAdjacencyMap::new(patterns.iter().cloned().map(|(pattern, _)| pattern));

    let tiles: Vec<_> = patterns
        .iter()
        .map(|(pattern, probability)| Tile::new(pattern, &adjacency[pattern], probability.clone()))
        .collect();

    let mut wave = Wave::new(&tiles, x_cells, y_cells, &adjacency);
    wave.collapse();


    Ok(())
}
