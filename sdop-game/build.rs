use asefile::AsepriteFile;
use image::GenericImageView;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{self},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
    vec,
};

const ASSETS_PATH: &'static str = "../assets";
const IMAGES_MISC_PATH: &str = "../assets/images/misc";
const IMAGES_TILESETS_PATH: &str = "../assets/images/misc/tilesets";
const PETS_RON_PATH: &str = "../assets/pets.ron";
const FOODS_RON_PATH: &str = "../assets/foods.ron";

#[derive(Default)]
struct ContentOut {
    assets: String,
    pet_definitions: String,
    food_definitions: String,
}

impl ContentOut {
    fn merge(&mut self, other: Self) {
        self.assets.push_str(&other.assets);
        self.pet_definitions.push_str(&other.pet_definitions);
        self.food_definitions.push_str(&other.food_definitions);
    }
}

fn write_vec_to_contents<T: ToString>(contents: &mut String, name: T, data: &[u8]) {
    let name = name.to_string();
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join(format!("{}.bin", name));
    std::fs::write(&dest_path, data).unwrap();

    contents.push_str(&format!(
        "const {}: &[u8; {}] = include_bytes!(\"{}\");\n",
        name,
        data.len(),
        dest_path.to_str().unwrap()
    ));
}

#[derive(Clone)]
struct ConvertedImage {
    suffix: Option<String>,
    width: usize,
    height: usize,
    data: Vec<u8>,
}

fn compress_image(img: image::RgbaImage, suffix: Option<String>) -> ConvertedImage {
    let width = img.width() as usize;
    let height = img.height() as usize;
    let bytes_needed = (width * height + 7) / 8;
    let mut compressed_data = vec![0u8; bytes_needed];
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel(x as u32, y as u32);
            if pixel[0] != 0 || pixel[1] != 0 || pixel[2] != 0 {
                let pixel_index = y * width + x;
                let byte_index = pixel_index / 8;
                let bit_index = pixel_index % 8;
                compressed_data[byte_index] |= 1 << bit_index;
            }
        }
    }
    ConvertedImage {
        suffix: suffix,
        width: width,
        height: height,
        data: compressed_data,
    }
}

struct Frame {
    duration_ms: u32,
    image: ConvertedImage,
}

enum ConvertOuput {
    Image(ConvertedImage),
    Anime(Vec<Frame>),
}

fn convert_ase_file<P: AsRef<Path>>(path: P) -> ConvertOuput {
    if !path.as_ref().exists() {
        panic!("{:?}", path.as_ref())
    }
    let ase = AsepriteFile::read_file(path.as_ref()).unwrap();

    if ase.num_frames() > 1 {
        let mut frames = vec![];
        for i in 0..ase.num_frames() {
            let ase_frame = ase.frame(i);
            frames.push(Frame {
                duration_ms: ase_frame.duration(),
                image: compress_image(ase_frame.image(), Some(i.to_string())),
            });
        }
        ConvertOuput::Anime(frames)
    } else {
        let image = ase.frame(0).image();
        // Convert image into a simple u8 array where any non 0,0,0 is converted to 1 and compressed into a u8
        ConvertOuput::Image(compress_image(image, None))
    }
}

#[derive(Serialize, Deserialize)]
struct Tileset {
    path: String,
    width: u32,
    height: u32,
    names: Vec<String>,
}

impl Tileset {
    fn write<T: ToString, P: AsRef<Path>>(target: &mut String, var_name: T, path: P) {
        let file_str = std::fs::read_to_string(path.as_ref()).unwrap();
        let tileset: Self = ron::from_str(&file_str).unwrap();
        let parent = path.as_ref().parent().unwrap();
        let ase_file = parent.join(tileset.path);
        let ase = AsepriteFile::read_file(&ase_file).unwrap();

        let img = ase.frame(0).image();

        let total_width = img.width();
        let num_tiles = total_width / tileset.width;

        let mut result = vec![];

        for i in 0..num_tiles {
            let tile = img
                .view(i * tileset.width, 0, tileset.width, tileset.height)
                .to_image();
            result.push(compress_image(
                tile,
                Some(tileset.names[i as usize].clone()),
            ));
        }

        let var_name_base = var_name.to_string();
        let data_name_base = format!("IMAGE_DATA_{}", var_name_base);
        for converted in result {
            let (var_name, data_name) = if let Some(suffix) = converted.suffix {
                (
                    format!("{}_{}", var_name_base, suffix),
                    format!("{}_{}", data_name_base, suffix),
                )
            } else {
                (var_name_base.clone(), data_name_base.clone())
            };
            write_vec_to_contents(target, &data_name, &converted.data);
            target.push_str(&format!(
                "pub const IMAGE_{}: StaticImage = StaticImage::new({}, {}, {});",
                var_name, converted.width, converted.height, data_name
            ));
        }
    }
}

struct WriteOutput {
    width: u32,
    height: u32,
}

fn write_image<T: ToString, P: AsRef<Path>>(
    target: &mut String,
    var_name: T,
    path: P,
) -> WriteOutput {
    let var_name_base = var_name.to_string();
    let data_name_base = format!("IMAGE_DATA_{}", var_name_base);
    let output = convert_ase_file(path);
    let images = match &output {
        ConvertOuput::Image(converted_image) => vec![converted_image.clone()],
        ConvertOuput::Anime(frames) => frames
            .iter()
            .map(|i| i.image.clone())
            .collect::<Vec<ConvertedImage>>(),
    };
    let width = images[0].width;
    let height = images[0].height;
    let mut image_names = vec![];
    for converted in images {
        let (var_name, data_name) = if let Some(suffix) = converted.suffix {
            (
                format!("{}_{}", var_name_base, suffix),
                format!("{}_{}", data_name_base, suffix),
            )
        } else {
            (var_name_base.clone(), data_name_base.clone())
        };
        write_vec_to_contents(target, &data_name, &converted.data);
        target.push_str(&format!(
            "pub const IMAGE_{}: StaticImage = StaticImage::new({}, {}, {});",
            var_name, converted.width, converted.height, data_name
        ));
        image_names.push(var_name);
    }

    if let ConvertOuput::Anime(frames) = &output {
        target.push_str(&format!(
            "pub const FRAMES_{}: [Frame; {}] = [",
            var_name_base,
            frames.len(),
        ));
        for i in 0..frames.len() {
            target.push_str(&format!(
                "Frame::new(&IMAGE_{}, Duration::from_millis({})), ",
                &image_names[i], frames[i].duration_ms,
            ));
        }
        target.push_str("];");
    }

    WriteOutput {
        width: width as u32,
        height: height as u32,
    }
}

fn generate_image_code<P: AsRef<Path>>(path: P) -> ContentOut {
    let entries = std::fs::read_dir(path).unwrap();

    let mut result = String::new();

    for entry in entries {
        let entry = entry.unwrap();
        let ext = if let Some(ext) = entry.path().extension() {
            ext.to_str().unwrap().to_string()
        } else {
            continue;
        };

        let name = entry
            .path()
            .file_stem()
            .clone()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            .to_uppercase();

        match ext.as_str() {
            "ase" | "aseprite" => {
                write_image(&mut result, name, entry.path());
            }
            _ => panic!(),
        }
    }

    ContentOut {
        assets: result,
        ..Default::default()
    }
}

fn generate_image_tilesets_code<P: AsRef<Path>>(path: P) -> ContentOut {
    let entries = std::fs::read_dir(path).unwrap();

    let mut result = String::new();

    for entry in entries {
        let entry = entry.unwrap();
        let ext = if let Some(ext) = entry.path().extension() {
            ext.to_str().unwrap().to_string()
        } else {
            continue;
        };

        let name = entry
            .path()
            .file_stem()
            .clone()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string()
            .to_uppercase();

        match ext.as_str() {
            "ase" => {}
            "ron" => {
                Tileset::write(&mut result, name, entry.path());
            }
            _ => panic!(),
        }
    }

    ContentOut {
        assets: result,
        ..Default::default()
    }
}

#[derive(Serialize, Deserialize)]
struct PetImageSet {
    normal: String,
    #[serde(default)]
    eat: Option<String>,
    #[serde(default)]
    happy: Option<String>,
    #[serde(default)]
    sad: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct PetTemplate {
    name: String,
    images: PetImageSet,
    stomach_size: f32,
    base_weight: f32,
}

fn generate_pet_definitions<P: AsRef<Path>>(path: P) -> ContentOut {
    let contents = std::fs::read_to_string(path).unwrap();

    let templates: Vec<PetTemplate> = ron::from_str(&contents).unwrap();

    let mut pet_definitions = String::new();
    let mut assets = String::new();

    let pets_length = templates.len();
    pet_definitions.push_str(&format!("pub const PET_COUNT: usize = {};", pets_length));

    // Define IDS
    let mut pet_vars = Vec::new();
    for (i, template) in templates.iter().enumerate() {
        let pet_var_name = template.name.replace(" ", "_").to_uppercase();
        let pet_var_name = format!("PET_{}", pet_var_name);
        // Normal
        let asset_path_base = PathBuf::from_str(ASSETS_PATH)
            .unwrap()
            .canonicalize()
            .unwrap();
        let image_normal_var_name = format!("{}_NORMAL", pet_var_name);
        let write_output = write_image(
            &mut assets,
            &image_normal_var_name,
            asset_path_base.join(&template.images.normal),
        );

        pet_definitions.push_str(&format!(
            "pub const {}_ID: PetDefinitionId = {};",
            pet_var_name, i
        ));

        pet_definitions.push_str(&format!(
            "pub const {}: PetDefinition = PetDefinition::new({}_ID, \"{}\", {:.2}, {:.2}, PetImageSet::new(&assets::FRAMES_{}, {}, {})",
            pet_var_name, pet_var_name, template.name, template.stomach_size, template.base_weight, image_normal_var_name, write_output.width, write_output.height
        ));

        if let Some(path) = &template.images.eat {
            let var_name = format!("{}_EAT", pet_var_name);
            write_image(&mut assets, &var_name, asset_path_base.join(&path));
            pet_definitions.push_str(&format!(".with_eat(&assets::FRAMES_{})", var_name));
        }

        if let Some(path) = &template.images.happy {
            let var_name = format!("{}_HAPPY", pet_var_name);
            write_image(&mut assets, &var_name, asset_path_base.join(&path));
            pet_definitions.push_str(&format!(".with_happy(&assets::FRAMES_{})", var_name));
        }

        if let Some(path) = &template.images.sad {
            let var_name = format!("{}_SAD", pet_var_name);
            write_image(&mut assets, &var_name, asset_path_base.join(&path));
            pet_definitions.push_str(&format!(".with_sad(&assets::FRAMES_{})", var_name));
        }

        pet_definitions.push_str(");");

        pet_vars.push(pet_var_name);
    }

    pet_definitions.push_str(&format!(
        "const PET_DEFINITIONS: [&'static PetDefinition; PET_COUNT] = ["
    ));
    for var in &pet_vars {
        pet_definitions.push_str(&format!("&{}, ", var));
    }
    pet_definitions.push_str(&format!("];"));

    ContentOut {
        assets: assets,
        pet_definitions,
        ..Default::default()
    }
}

#[derive(Serialize, Deserialize)]
struct FoodTemplate {
    name: String,
    fill_factor: f32,
    path: String,
}

fn generate_food_definitions<P: AsRef<Path>>(path: P) -> ContentOut {
    let contents = std::fs::read_to_string(path).unwrap();

    let templates: Vec<FoodTemplate> = ron::from_str(&contents).unwrap();

    let mut food_definitions = String::new();
    let mut assets = String::new();

    let food_length = templates.len();
    food_definitions.push_str(&format!("pub const FOOD_COUNT: usize = {};", food_length));

    let mut food_vars = Vec::new();
    for (i, template) in templates.into_iter().enumerate() {
        let food_var_name = template.name.replace(" ", "_").to_uppercase();
        // Normal
        let image_path = PathBuf::from_str(&format!("{}/{}", ASSETS_PATH, template.path)).unwrap();
        let image_normal_var_name = format!("FOOD_{}", food_var_name);
        write_image(&mut assets, &image_normal_var_name, image_path);

        food_definitions.push_str(&format!(
            "pub const FOOD_{}: Food = Food::new({}, \"{}\", &assets::IMAGE_{}, {:.2});",
            food_var_name, i, template.name, image_normal_var_name, template.fill_factor
        ));

        food_vars.push(image_normal_var_name);
    }

    food_definitions.push_str(&format!("pub const FOODS: [&'static Food; FOOD_COUNT] = ["));
    for var in &food_vars {
        food_definitions.push_str(&format!("&{}, ", var));
    }
    food_definitions.push_str("];");

    ContentOut {
        assets: assets,
        food_definitions: food_definitions,
        ..Default::default()
    }
}

fn main() {
    let mut contents = ContentOut::default();

    contents.merge(generate_image_code(IMAGES_MISC_PATH));
    contents.merge(generate_image_tilesets_code(IMAGES_TILESETS_PATH));
    contents.merge(generate_pet_definitions(PETS_RON_PATH));
    contents.merge(generate_food_definitions(FOODS_RON_PATH));

    let out_dir = env::var_os("OUT_DIR").unwrap();

    let dist_assets_path = Path::new(&out_dir).join("dist_assets.rs");
    fs::write(&dist_assets_path, contents.assets).unwrap();
    Command::new("rustfmt")
        .arg(dist_assets_path)
        .spawn()
        .expect("Unable to format")
        .wait()
        .unwrap();

    let dist_pets_path = Path::new(&out_dir).join("dist_pets.rs");
    fs::write(&dist_pets_path, contents.pet_definitions).unwrap();
    Command::new("rustfmt")
        .arg(dist_pets_path)
        .spawn()
        .expect("Unable to format")
        .wait()
        .unwrap();

    let dist_foods_path = Path::new(&out_dir).join("dist_foods.rs");
    fs::write(&dist_foods_path, contents.food_definitions).unwrap();
    Command::new("rustfmt")
        .arg(dist_foods_path)
        .spawn()
        .expect("Unable to format")
        .wait()
        .unwrap();

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=assets");
}
