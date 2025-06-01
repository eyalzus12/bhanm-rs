use bhanm::AnmFile;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;

fn main() -> Result<(), Box<dyn Error>> {
    const ANM_PATH: &str =
        "C:/Program Files (x86)/Steam/steamapps/common/Brawlhalla/anims/Animation_Bow.anm";

    let file = File::open(ANM_PATH)?;
    let reader = BufReader::new(file);
    let anm_file = AnmFile::read(reader)?;

    for (key, class) in anm_file.classes {
        println!("Anm class {key}, with the animations:");
        for animation in class.animations.iter() {
            println!("{}", animation.name);
        }
    }

    Ok(())
}
