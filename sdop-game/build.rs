use asefile::AsepriteFile;
use chrono::{Datelike, Days, NaiveDate};
use convert_case::{Case, Casing};
use image::GenericImageView;
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

#[derive(Default)]
struct ContentOut {
    assets: String,
    pet_definitions: String,
    food_definitions: String,
    item_definitions: String,
    dates_definitions: String,
}

impl ContentOut {
    fn merge(&mut self, other: Self) {
        self.assets.push_str(&other.assets);
        self.pet_definitions.push_str(&other.pet_definitions);
        self.food_definitions.push_str(&other.food_definitions);
        self.item_definitions.push_str(&other.item_definitions);
        self.dates_definitions.push_str(&other.dates_definitions);
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
    #[serde(default)]
    sleep: Option<String>,
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

        if let Some(path) = &template.images.sleep {
            let var_name = format!("{}_SLEEP", pet_var_name);
            write_image(&mut assets, &var_name, asset_path_base.join(&path));
            pet_definitions.push_str(&format!(".with_sleep(&assets::FRAMES_{})", var_name));
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

#[derive(Serialize, Deserialize)]
struct ItemEntry {
    name: String,
    cost: i32,
    rarity: RarityEnum,
    image: String,
    unique: bool,
}

fn generate_item_enum<P: AsRef<Path>>(path: P, food_path: P) -> ContentOut {
    let contents = std::fs::read_to_string(food_path).unwrap();
    let food_tempaltes: Vec<FoodTemplate> = ron::from_str(&contents).unwrap();

    let contents = std::fs::read_to_string(path).unwrap();
    let templates: Vec<ItemEntry> = ron::from_str(&contents).unwrap();

    let mut enum_def = String::new();
    enum_def
        .push_str("#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, EnumCount, FromRepr)]\n");
    enum_def.push_str("#[repr(usize)]\n");
    enum_def.push_str("pub enum Item \n{");
    enum_def.push_str("None = 0,");
    let mut rare_fn_def = String::new();
    rare_fn_def.push_str("pub const fn rarity(&self) -> ItemRarity {\n");
    rare_fn_def.push_str("return match self {\n");
    rare_fn_def.push_str("Item::None => ItemRarity::Common,\n");
    let mut cost_fn_def = String::new();
    cost_fn_def.push_str("pub const fn cost(&self) -> crate::money::Money {\n");
    cost_fn_def.push_str("return match self {\n");
    cost_fn_def.push_str("Item::None => 0,\n");
    let mut name_fn_def = String::new();
    name_fn_def.push_str("pub const fn name(&self) -> &'static str {\n");
    name_fn_def.push_str("return match self {\n");
    name_fn_def.push_str("Item::None => \"none\",\n");
    let mut unique_fn_def = String::new();
    unique_fn_def.push_str("pub const fn unique(&self) -> bool {\n");
    unique_fn_def.push_str("return match self {\n");
    unique_fn_def.push_str("Item::None => false,\n");
    let mut image_fn_def = String::new();
    image_fn_def.push_str("pub const fn image(&self) -> &'static crate::assets::StaticImage {\n");
    image_fn_def.push_str("return match self {\n");
    image_fn_def.push_str("Item::None => &crate::assets::IMAGE_POOP_0,\n");
    let mut from_food_fn = String::new();
    from_food_fn.push_str("pub const fn from_food(food_id: u32) -> Self {\n");
    from_food_fn.push_str("return match food_id {\n");

    let mut item_count = 1;
    for template in templates {
        let enum_name = template.name.to_case(Case::Pascal);
        enum_def.push_str(&format!("{} = {},\n", enum_name, item_count));
        rare_fn_def.push_str(&format!(
            "Item::{} => ItemRarity::{},\n",
            enum_name,
            template.rarity.to_string()
        ));
        cost_fn_def.push_str(&format!("Item::{} => {},\n", enum_name, template.cost));

        name_fn_def.push_str(&format!("Item::{} => \"{}\",\n", enum_name, template.name));

        unique_fn_def.push_str(&format!("Item::{} => {},\n", enum_name, template.unique));

        image_fn_def.push_str(&format!(
            "Item::{} => &crate::assets::IMAGE_{},\n",
            enum_name,
            template.image.to_case(Case::UpperSnake)
        ));
        item_count += 1;
    }

    for (i, template) in food_tempaltes.iter().enumerate() {
        let enum_name = format!("Recipe{}", template.name.to_case(Case::Pascal));
        enum_def.push_str(&format!("{} = {},\n", enum_name, item_count));
        rare_fn_def.push_str(&format!("Item::{} => ItemRarity::Common,\n", enum_name,));

        let price = (template.fill_factor * 50.) as i32;
        cost_fn_def.push_str(&format!("Item::{} => {},\n", enum_name, price));

        name_fn_def.push_str(&format!("Item::{} => \"{}\",\n", enum_name, template.name));

        unique_fn_def.push_str(&format!("Item::{} => true,\n", enum_name));

        let food_var_name = template.name.replace(" ", "_").to_uppercase();
        image_fn_def.push_str(&format!(
            "Item::{} => &crate::assets::IMAGE_FOOD_{},\n",
            enum_name, food_var_name
        ));

        from_food_fn.push_str(&format!("{} => Self::{},\n", i + 1, enum_name));

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

    let mut items_definitions = String::new();

    items_definitions.push_str(&enum_def);
    items_definitions.push_str("impl Item {\n");
    items_definitions.push_str(&rare_fn_def);
    items_definitions.push_str(&cost_fn_def);
    items_definitions.push_str(&name_fn_def);
    items_definitions.push_str(&unique_fn_def);
    items_definitions.push_str(&image_fn_def);
    items_definitions.push_str(&from_food_fn);
    items_definitions.push_str("}");

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

fn main() {
    let mut contents = ContentOut::default();

    contents.merge(generate_image_code(IMAGES_MISC_PATH));
    contents.merge(generate_image_tilesets_code(IMAGES_TILESETS_PATH));
    contents.merge(generate_pet_definitions(PETS_RON_PATH));
    contents.merge(generate_food_definitions(FOODS_RON_PATH));
    contents.merge(generate_item_enum(ITEMS_RON_PATH, FOODS_RON_PATH));
    contents.merge(generate_dates());

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

    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=assets");
}
