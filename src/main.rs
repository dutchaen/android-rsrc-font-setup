
use std::fs;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use convert_case::{Casing, Case};

use font_kit::font::Font;
use font_kit::properties::{Style, Weight};


const COMMON_FONT_EXTENSIONS: &[&str; 4] = &["otf", "ttf", "woff", "woff2"];


#[derive(Debug, Clone)]
struct FontInfo {
    inner: Font,
    resource_name: String,
}


fn main() {
    println!("Android Resource Font Setup");
    println!();

    let wd = std::env::current_dir()
        .expect("get working directory");

    let mut dir: PathBuf = wd.clone();


    loop {
        if folder_contains_fonts(&dir) {
            break;
        }

        match dir == wd {
            true => println!("Sorry, the current working directory does not contain any fonts."),
            _ => println!("Sorry, the directory selected does not contain any fonts.")
        }

        eprint!("Enter the path of where your fonts are located: ");
        let input: String = read_input()
            .expect("valid input was given from the user");

        dir = input.into();
    }


    println!("Fonts have been found!");
    println!("Directory: {:?}", dir);
    println!();


    eprint!("Rename fonts for Android Resource? y/n: ");
    let input: String = read_input()
        .expect("valid input was given from the user");

    if !input.starts_with('y') && !input.starts_with('Y') {
        println!("Exiting...");
        return;
    }

    rename_fonts_for_android(&dir);
    println!("Renamed fonts for Android.");

    let fonts = get_fonts_in_resource(&dir);

    let code = template_fonts_for_jetpack_compose(&fonts);

    println!();
    println!();
    println!("#jetpack_compose:");
    println!();

    println!("{}", code);
    println!();

}


fn template_fonts_for_jetpack_compose(fonts: &HashMap<String, Vec<FontInfo>>) -> String {
    let mut code = String::new();

    for (font_family_name, font_versions) in fonts {

        let variable_name = font_family_name_to_variable_name(&font_family_name);

        code += &format!("val {} = FontFamily(\r\n", variable_name);

        for (index, version) in font_versions.iter().enumerate() {

            let properties = version.inner.properties();
            
            let font_weight_variable = match properties.weight {
                Weight::THIN => "FontWeight.Thin",
                Weight::EXTRA_LIGHT => "FontWeight.ExtraLight",
                Weight::LIGHT => "FontWeight.Light",
                Weight::NORMAL => "FontWeight.Normal",
                Weight::MEDIUM => "FontWeight.Medium",
                Weight::SEMIBOLD => "FontWeight.SemiBold",
                Weight::BOLD => "FontWeight.Bold",
                Weight::EXTRA_BOLD => "FontWeight.ExtraBold",
                Weight::BLACK => "FontWeight.Black",
                _ => panic!("font weight type was not recognized")
            };


            // assuming in resources folder for template code
            let next_line = match properties.style {
                Style::Italic => format!("\tFont(R.font.{}, {}, {})", version.resource_name, font_weight_variable, "FontStyle.Italic"),
                _ => format!("\tFont(R.font.{}, {})", version.resource_name, font_weight_variable),
            };

            code += &next_line;

            if index != font_versions.len() - 1 {
                code.push(',');
            }
            code.push_str("\r\n");

        }

        code.push(')');


        code.push_str("\r\n\r\n\r\n");
    }

    return code;
}


fn folder_contains_fonts(dir: &Path) -> bool {
    if !dir.is_dir() {
        return false;
    }
    
    if let Ok(dir) = fs::read_dir(dir) {

        let filtered_dir = dir.filter_map(|x| x.ok());
        for entry in filtered_dir {

            let file_type = entry.file_type()
                .expect("valid file type from DirEntry");


            if !file_type.is_file() {
                continue;
            }

            let path = entry.path();

            if let Some(extension) = path.extension() {

                let extension = extension.to_str()
                    .expect("OsStr to convert to &str");

                if COMMON_FONT_EXTENSIONS.contains(&extension) {
                    return true;
                }
            }
        }
    }
    
    return false;
}


fn rename_fonts_for_android(rsrc_font_dir: &Path) {

    if let Ok(dir) = fs::read_dir(rsrc_font_dir) {

        let filtered_dir = dir.filter_map(|x| x.ok());
        for entry in filtered_dir {

            let file_type = entry.file_type()
                .expect("valid file type from DirEntry");


            if !file_type.is_file() {
                continue;
            }

            let path = entry.path();
            let mut filename = path.file_name()
                .expect("filename as &OsStr")
                .to_owned()
                .into_string()
                .expect("OsString to String conversion");

            if let Some(extension) = path.extension() {

                let extension = extension.to_str()
                    .expect("OsStr to convert to &str");

                if COMMON_FONT_EXTENSIONS.contains(&extension) {


                    let old_path = rsrc_font_dir.join(&filename);

                    filename = filename.to_lowercase();
                    filename = filename.replace("-", "_");
                    filename = filename.replace(" ", "_");

                    let new_path = rsrc_font_dir.join(&filename);

                    fs::rename(old_path, new_path)
                        .expect("font path is renamed to fixed name");
                }
            }
        }
    }
}


fn get_fonts_in_resource(rsrc_font_dir: &Path) -> HashMap<String, Vec<FontInfo>> {

    let mut font_map: HashMap<String, Vec<FontInfo>> = HashMap::new();

    if let Ok(dir) = fs::read_dir(rsrc_font_dir) {

        let filtered_dir = dir.filter_map(|x| x.ok());
        for entry in filtered_dir {

            let file_type = entry.file_type()
                .expect("valid file type from DirEntry");


            if !file_type.is_file() {
                continue;
            }

            let path = entry.path();
            let filename = path.file_name()
                .expect("filename as &OsStr")
                .to_owned()
                .into_string()
                .expect("OsString to String conversion");

            if let Some(extension) = path.extension() {

                let extension = extension.to_str()
                    .expect("OsStr to convert to &str");

                let extension_start_index = filename.len()-extension.len()-1;
                let resource_name = filename[..extension_start_index].to_owned();

                if COMMON_FONT_EXTENSIONS.contains(&extension) {


                    let font = Font::from_path(&path, 0)
                        .expect("load font from path");

                    let font_name = font.family_name();
                    let info = FontInfo {
                        inner: font,
                        resource_name
                    };

                    font_map.entry(font_name)
                        .and_modify(|x| x.push(info.clone()))
                        .or_insert(vec![info]);

                    continue;
                }
            }
        }
    }

    return font_map;
}


fn font_family_name_to_variable_name(name: &str) -> String {
    return name.to_case(Case::Pascal) + "FontFamily";
}


fn read_input<T: std::str::FromStr>
    () -> Result<T, Box<dyn std::error::Error>> {

    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;
    input = input.trim().to_string();

    match input.parse() {
        Ok(value) => Ok(value),
        Err(_) => Err("could not be parsed".into())
    }
}
