#![feature(trim_prefix_suffix)]
use asefile::AsepriteFile;
use chrono::{Datelike, Days, NaiveDate};
use convert_case::{Case, Casing};
use image::{GenericImageView, Rgba};
use sdop_common::MelodyEntry;
use serde::{Deserialize, Serialize};
use solar_calendar_events::AnnualSolarEvent;
use std::{
    env,
    fs::{self},
    path::{Path, PathBuf},
    process::Command,
    str::FromStr,
    vec,
};
use strum_macros::{Display, EnumString};

const ASSETS_PATH: &'static str = "../assets";
const IMAGES_MISC_PATH: &str = "../assets/images/misc";
const IMAGES_TILESETS_PATH: &str = "../assets/images/misc/tilesets";
const PETS_RON_PATH: &str = "../assets/pets.ron";
const FOODS_RON_PATH: &str = "../assets/foods.ron";
const ITEMS_RON_PATH: &str = "../assets/items.ron";
const SOUNDS_PATH: &str = "../assets/sounds";

#[derive(Default)]
struct ContentOut {
    assets: String,
    pet_definitions: String,
    food_definitions: String,
    item_definitions: String,
    dates_definitions: String,
    sounds_definitions: String,
}

impl ContentOut {
    fn merge(&mut self, other: Self) {
        self.assets.push_str(&other.assets);
        self.pet_definitions.push_str(&other.pet_definitions);
        self.food_definitions.push_str(&other.food_definitions);
        self.item_definitions.push_str(&other.item_definitions);
        self.dates_definitions.push_str(&other.dates_definitions);
        self.sounds_definitions.push_str(&other.sounds_definitions);
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

fn replace_alpha(mut img: image::RgbaImage, mask: bool) -> image::RgbaImage {
    for pixel in img.pixels_mut() {
        if pixel[3] == 0 {
            *pixel = if mask {
                Rgba([255, 255, 255, 255])
            } else {
                Rgba([0, 0, 0, 255])
            };
        }
    }

    img
}

fn convert_ase_file(ase: &AsepriteFile, mask: bool) -> ConvertOuput {
    if ase.num_frames() > 1 {
        let mut frames = vec![];
        for i in 0..ase.num_frames() {
            let ase_frame = ase.frame(i);
            let img = replace_alpha(ase_frame.image(), mask);
            let suffix = i.to_string();
            frames.push(Frame {
                duration_ms: ase_frame.duration(),
                image: compress_image(img, Some(suffix)),
            });
        }
        ConvertOuput::Anime(frames)
    } else {
        let image = replace_alpha(ase.frame(0).image(), mask);
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

fn write_image_base<T: ToString>(
    target: &mut String,
    var_name: T,
    ase: &AsepriteFile,
    mask: bool,
) -> WriteOutput {
    let mut var_name_base = var_name.to_string();
    if mask {
        var_name_base += "_MASK"
    }
    let data_name_base = format!("IMAGE_DATA_{}", var_name_base);

    let output = convert_ase_file(ase, mask);
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

struct WriteOutput {
    width: u32,
    height: u32,
}

fn write_image<T: ToString, P: AsRef<Path>>(
    target: &mut String,
    var_name: T,
    path: P,
) -> WriteOutput {
    if !path.as_ref().exists() {
        panic!("{:?}", path.as_ref())
    }
    let ase = AsepriteFile::read_file(path.as_ref()).unwrap();
    let var_name = var_name.to_string();
    // Check if first frame has alpha
    if ase.frame(0).image().pixels().any(|pixel| pixel[3] == 0) {
        write_image_base(target, &var_name, &ase, true);
    }

    write_image_base(target, var_name, &ase, false)
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
    #[serde(default)]
    sleep: Option<String>,
}

#[derive(Serialize, Deserialize, Display)]
enum LifeStage {
    Baby,
    Child,
    Adult,
    Elder,
}

#[derive(Serialize, Deserialize)]
struct PetTemplate {
    name: String,
    life_stage: LifeStage,
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
            "pub const {}: PetDefinition = PetDefinition::new({}_ID, \"{}\", LifeStage::{}, {:.2}, {:.2}, PetImageSet::new(MaskedFramesSet::new(&assets::FRAMES_{}, &assets::FRAMES_{}_MASK), {}, {})",
            pet_var_name, pet_var_name, template.name, template.life_stage, template.stomach_size, template.base_weight, image_normal_var_name, image_normal_var_name, write_output.width, write_output.height
        ));

        if let Some(path) = &template.images.eat {
            let var_name = format!("{}_EAT", pet_var_name);
            write_image(&mut assets, &var_name, asset_path_base.join(&path));
            pet_definitions.push_str(&format!(
                ".with_eat(MaskedFramesSet::new(&assets::FRAMES_{}, &assets::FRAMES_{}_MASK))",
                var_name, var_name
            ));
        }

        if let Some(path) = &template.images.happy {
            let var_name = format!("{}_HAPPY", pet_var_name);
            write_image(&mut assets, &var_name, asset_path_base.join(&path));
            pet_definitions.push_str(&format!(
                ".with_happy(MaskedFramesSet::new(&assets::FRAMES_{}, &assets::FRAMES_{}_MASK))",
                var_name, var_name
            ));
        }

        if let Some(path) = &template.images.sad {
            let var_name = format!("{}_SAD", pet_var_name);
            write_image(&mut assets, &var_name, asset_path_base.join(&path));
            pet_definitions.push_str(&format!(
                ".with_sad(MaskedFramesSet::new(&assets::FRAMES_{}, &assets::FRAMES_{}_MASK))",
                var_name, var_name
            ));
        }

        if let Some(path) = &template.images.sleep {
            let var_name = format!("{}_SLEEP", pet_var_name);
            write_image(&mut assets, &var_name, asset_path_base.join(&path));
            pet_definitions.push_str(&format!(
                ".with_sleep(MaskedFramesSet::new(&assets::FRAMES_{}, &assets::FRAMES_{}_MASK))",
                var_name, var_name
            ));
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

#[derive(Serialize, Deserialize, EnumString, Display)]
enum RarityEnum {
    Common,
    Rare,
}

#[derive(Serialize, Deserialize, Display)]
pub enum ItemCategory {
    Misc,
    Furniture,
    PlayThing,
    Usable,
    Book,
    Software,
    Food,
}

#[derive(Serialize, Deserialize)]
struct ItemEntry {
    name: String,
    cost: i32,
    rarity: RarityEnum,
    image: String,
    unique: bool,
    desc: String,
    #[serde(default)]
    fishing_odds: f32,
    category: ItemCategory,
}

fn generate_item_enum<P: AsRef<Path>>(path: P, food_path: P) -> ContentOut {
    let contents = std::fs::read_to_string(food_path).unwrap();
    let food_tempaltes: Vec<FoodTemplate> = ron::from_str(&contents).unwrap();

    let contents = std::fs::read_to_string(path).unwrap();
    let templates: Vec<ItemEntry> = ron::from_str(&contents).unwrap();

    let mut enum_def = String::new();

    enum_def.push_str(
        "#[cfg_attr(feature = \"serde\", derive(serde::Serialize, serde::Deserialize))]\n",
    );
    enum_def
        .push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount, FromRepr, bincode::Encode, bincode::Decode)]\n");
    enum_def.push_str("#[repr(usize)]\n");
    enum_def.push_str("pub enum ItemKind \n{");
    enum_def.push_str("None = 0,");
    let mut rare_fn_def = String::new();
    rare_fn_def.push_str("pub const fn rarity(&self) -> ItemRarity {\n");
    rare_fn_def.push_str("return match self {\n");
    rare_fn_def.push_str("Self::None => ItemRarity::Common,\n");
    let mut cost_fn_def = String::new();
    cost_fn_def.push_str("pub const fn cost(&self) -> crate::money::Money {\n");
    cost_fn_def.push_str("return match self {\n");
    cost_fn_def.push_str("Self::None => 0,\n");
    let mut name_fn_def = String::new();
    name_fn_def.push_str("pub const fn name(&self) -> &'static str {\n");
    name_fn_def.push_str("return match self {\n");
    name_fn_def.push_str("Self::None => \"none\",\n");
    let mut unique_fn_def = String::new();
    unique_fn_def.push_str("pub const fn unique(&self) -> bool {\n");
    unique_fn_def.push_str("return match self {\n");
    unique_fn_def.push_str("Self::None => false,\n");
    let mut image_fn_def = String::new();
    image_fn_def.push_str("pub const fn image(&self) -> &'static crate::assets::StaticImage {\n");
    image_fn_def.push_str("return match self {\n");
    image_fn_def.push_str("Self::None => &crate::assets::IMAGE_POOP_0,\n");
    let mut from_food_fn = String::new();
    from_food_fn.push_str("pub const fn from_food(food_id: u32) -> Self {\n");
    from_food_fn.push_str("return match food_id {\n");
    let mut desc_fn = String::new();
    desc_fn.push_str("pub const fn desc(&self) -> &'static str {\n");
    desc_fn.push_str("return match self {\n");
    desc_fn.push_str("Self::None => \"?????\",\n");
    let mut category_fn = String::new();
    category_fn.push_str("pub const fn category(&self) -> ItemCategory {\n");
    category_fn.push_str("return match self {\n");
    category_fn.push_str("Self::None => ItemCategory::Misc,\n");
    let mut fishing_chance_def = String::new();
    fishing_chance_def.push_str("pub const FISHING_ITEM_ODDS: &[ItemChance] = &[\n");

    let fishing_sum: f32 = templates.iter().map(|i| i.fishing_odds).sum();
    let mut fishing_current: f32 = 0.;

    let mut item_count = 1;
    for template in templates {
        let enum_name = template.name.to_case(Case::Pascal);
        enum_def.push_str(&format!("{} = {},\n", enum_name, item_count));
        rare_fn_def.push_str(&format!(
            "Self::{} => ItemRarity::{},\n",
            enum_name,
            template.rarity.to_string()
        ));
        cost_fn_def.push_str(&format!("Self::{} => {},\n", enum_name, template.cost));

        name_fn_def.push_str(&format!("Self::{} => \"{}\",\n", enum_name, template.name));

        desc_fn.push_str(&format!("Self::{} => \"{}\",\n", enum_name, template.desc));

        unique_fn_def.push_str(&format!("Self::{} => {},\n", enum_name, template.unique));

        category_fn.push_str(&format!(
            "Self::{} => ItemCategory::{},\n",
            enum_name, template.category
        ));

        image_fn_def.push_str(&format!(
            "Self::{} => &crate::assets::IMAGE_{},\n",
            enum_name,
            template.image.to_case(Case::UpperSnake)
        ));

        if template.fishing_odds > 0. {
            fishing_current += template.fishing_odds / fishing_sum;
            fishing_chance_def.push_str(&format!(
                "ItemChance::new(ItemKind::{}, {:.8}),",
                enum_name, fishing_current
            ));
        }

        item_count += 1;
    }

    for (i, template) in food_tempaltes.iter().enumerate() {
        let enum_name = format!("Recipe{}", template.name.to_case(Case::Pascal));
        enum_def.push_str(&format!("{} = {},\n", enum_name, item_count));
        rare_fn_def.push_str(&format!("Self::{} => ItemRarity::Common,\n", enum_name,));

        let price = (template.fill_factor * 50.) as i32;
        cost_fn_def.push_str(&format!("Self::{} => {},\n", enum_name, price));

        name_fn_def.push_str(&format!("Self::{} => \"{}\",\n", enum_name, template.name));

        desc_fn.push_str(&format!(
            "Self::{} => \"Allows to make {}\",\n",
            enum_name, template.name
        ));

        unique_fn_def.push_str(&format!("Self::{} => true,\n", enum_name));

        let food_var_name = template.name.replace(" ", "_").to_uppercase();
        image_fn_def.push_str(&format!(
            "Self::{} => &crate::assets::IMAGE_FOOD_{},\n",
            enum_name, food_var_name
        ));

        from_food_fn.push_str(&format!("{} => Self::{},\n", i, enum_name));

        category_fn.push_str(&format!("Self::{} => ItemCategory::Food,\n", enum_name,));

        item_count += 1;
    }

    enum_def.push_str("}\n");
    rare_fn_def.push_str("}\n}\n");
    cost_fn_def.push_str("}\n}\n");
    name_fn_def.push_str("}\n}\n");
    unique_fn_def.push_str("}\n}\n");
    image_fn_def.push_str("}\n}\n");
    from_food_fn.push_str("_ => Self::None\n");
    from_food_fn.push_str("}\n}\n");
    category_fn.push_str("}}");
    desc_fn.push_str("}\n}\n");
    fishing_chance_def.push_str("];");

    let mut items_definitions = String::new();

    items_definitions.push_str(&enum_def);
    items_definitions.push_str("impl ItemKind {\n");
    items_definitions.push_str(&rare_fn_def);
    items_definitions.push_str(&cost_fn_def);
    items_definitions.push_str(&name_fn_def);
    items_definitions.push_str(&unique_fn_def);
    items_definitions.push_str(&image_fn_def);
    items_definitions.push_str(&from_food_fn);
    items_definitions.push_str(&desc_fn);
    items_definitions.push_str(&category_fn);
    items_definitions.push_str("}");
    items_definitions.push_str(&fishing_chance_def);

    ContentOut {
        item_definitions: items_definitions,
        ..Default::default()
    }
}
pub struct SpecialDay {
    kind: String,
    month: u32,
    day: u32,
}

fn get_date_of_n_day_in_month(
    year: i32,
    month: u32,
    mut n: u32,
    weekday: chrono::Weekday,
) -> NaiveDate {
    let mut date = NaiveDate::from_ymd_opt(year, month, 1).unwrap();

    while date.month() == month {
        if date.weekday() == weekday {
            n -= 1;
        }
        if n <= 0 {
            return date;
        }
        date = date.checked_add_days(chrono::Days::new(1)).unwrap();
    }

    panic!("Impossible {} {} {}", year, month, n)
}

fn generate_dates() -> ContentOut {
    let mut dates_definitions = String::new();

    // So lets start at 1970 and go until 2370
    dates_definitions.push_str("pub const DYNAMIC_SPECIAL_DAYS: &[&[SpecialDay]] = &[");

    for year in 2025..=2100 {
        let mut dates = vec![];
        let easter = computus::gregorian(year).unwrap();
        let easter = NaiveDate::from_ymd_opt(year, easter.month, easter.day).unwrap();

        dates.push(SpecialDay {
            kind: "EasterSunday".to_owned(),
            month: easter.month(),
            day: easter.day(),
        });

        let good_friday = easter.checked_sub_days(Days::new(2)).unwrap();
        dates.push(SpecialDay {
            kind: "GoodFriday".to_owned(),
            month: good_friday.month(),
            day: good_friday.day(),
        });

        let monday = easter.checked_add_days(Days::new(1)).unwrap();

        dates.push(SpecialDay {
            kind: "EasterMonday".to_owned(),
            month: monday.month(),
            day: monday.day(),
        });

        let grand_final_eve = get_date_of_n_day_in_month(year, 9, 4, chrono::Weekday::Fri);

        dates.push(SpecialDay {
            kind: "GrandFinalEve".to_owned(),
            month: grand_final_eve.month(),
            day: grand_final_eve.day(),
        });

        let fathers_day = get_date_of_n_day_in_month(year, 9, 1, chrono::Weekday::Sun);
        dates.push(SpecialDay {
            kind: "FathersDay".to_owned(),
            month: fathers_day.month(),
            day: fathers_day.day(),
        });

        let mothers_day = get_date_of_n_day_in_month(year, 5, 2, chrono::Weekday::Sun);
        dates.push(SpecialDay {
            kind: "MothersDay".to_owned(),
            month: mothers_day.month(),
            day: mothers_day.day(),
        });

        let melbourne_cup = get_date_of_n_day_in_month(year, 11, 1, chrono::Weekday::Tue);
        dates.push(SpecialDay {
            kind: "MelbourneCup".to_owned(),
            month: melbourne_cup.month(),
            day: melbourne_cup.day(),
        });

        let solar = solar_calendar_events::AnnualSolarEvents::for_year(year).unwrap();
        dates.push(SpecialDay {
            kind: "SeptemberEquinox".to_owned(),
            month: solar.september_equinox().date_time().month(),
            day: solar.september_equinox().date_time().day(),
        });
        dates.push(SpecialDay {
            kind: "MarchEquinox".to_owned(),
            month: solar.march_equinox().date_time().month(),
            day: solar.march_equinox().date_time().day(),
        });
        dates.push(SpecialDay {
            kind: "WinterSolstice".to_owned(),
            month: solar.june_solstice().date_time().month(),
            day: solar.june_solstice().date_time().day(),
        });
        dates.push(SpecialDay {
            kind: "SummerSolstice".to_owned(),
            month: solar.december_solstice().date_time().month(),
            day: solar.december_solstice().date_time().day(),
        });

        dates.sort_by(|a, b| {
            if a.month == b.month {
                a.day.cmp(&b.day)
            } else {
                a.month.cmp(&b.month)
            }
        });

        dates_definitions.push_str("&[");
        for date in dates {
            dates_definitions.push_str(&format!(
                "SpecialDay::new(SpecialDayKind::{}, {}, {}),",
                date.kind, date.month, date.day
            ));
        }
        dates_definitions.push_str("],");
    }

    dates_definitions.push_str("];");

    ContentOut {
        dates_definitions: dates_definitions,
        ..Default::default()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Song {
    name: String,
    melody: Vec<MelodyEntry>,
    tempo: i16,
}

fn generate_sounds() -> ContentOut {
    let root_path = PathBuf::from_str(SOUNDS_PATH).unwrap();

    let entries = std::fs::read_dir(&root_path).unwrap();

    let mut sounds_def = String::new();

    for entry in entries.into_iter().filter_map(|i| i.ok()) {
        let ron_str = fs::read_to_string(entry.path()).unwrap();

        let song: Song = ron::from_str(&ron_str).unwrap();
        let mut melody_def = String::new();

        for entry in &song.melody {
            melody_def.push_str(&format!(
                "MelodyEntry::new(sdop_common::Note::{:?}, {}), ",
                entry.note, entry.duration
            ));
        }

        sounds_def.push_str(&format!(
            "pub const SONG_{}: Song = Song::new(&[{}], {});",
            song.name.to_case(Case::UpperSnake),
            melody_def,
            song.tempo
        ));
    }

    ContentOut {
        sounds_definitions: sounds_def,
        ..Default::default()
    }
}

fn main() {
    let mut contents = ContentOut::default();

    contents.merge(generate_image_code(IMAGES_MISC_PATH));
    contents.merge(generate_image_tilesets_code(IMAGES_TILESETS_PATH));
    contents.merge(generate_pet_definitions(PETS_RON_PATH));
    contents.merge(generate_food_definitions(FOODS_RON_PATH));
    contents.merge(generate_item_enum(ITEMS_RON_PATH, FOODS_RON_PATH));
    contents.merge(generate_dates());
    contents.merge(generate_sounds());

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

    let dist_items_path = Path::new(&out_dir).join("dist_items.rs");
    fs::write(&dist_items_path, contents.item_definitions).unwrap();
    Command::new("rustfmt")
        .arg(dist_items_path)
        .spawn()
        .expect("Unable to format")
        .wait()
        .unwrap();

    let dist_dates_path = Path::new(&out_dir).join("dist_dates.rs");
    fs::write(&dist_dates_path, contents.dates_definitions).unwrap();
    Command::new("rustfmt")
        .arg(dist_dates_path)
        .spawn()
        .expect("Unable to format")
        .wait()
        .unwrap();

    let dist_sounds_path = Path::new(&out_dir).join("dist_sounds.rs");
    fs::write(&dist_sounds_path, contents.sounds_definitions).unwrap();
    Command::new("rustfmt")
        .arg(dist_sounds_path)
        .spawn()
        .expect("Unable to format")
        .wait()
        .unwrap();

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=assets");
}
