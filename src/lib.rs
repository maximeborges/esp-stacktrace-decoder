use std::{borrow::Cow};
use addr2line::{Context, fallible_iterator::FallibleIterator, gimli, object::{self, Endianness, Object, ObjectSection}};

use typed_arena::Arena;
use regex::Regex;
#[cfg(target_family = "wasm")]
use wasm_bindgen::prelude::*;
#[cfg(target_family = "wasm")]
use js_sys::Array;

fn load_file_section<'input, 'arena, Endian: gimli::Endianity>(
    id: gimli::SectionId,
    file: &object::File<'input>,
    endian: Endian,
    arena_data: &'arena Arena<Cow<'input, [u8]>>,
) -> Result<gimli::EndianSlice<'arena, Endian>, object::Error> {
    let name = id.name();
    match file.section_by_name(name) {
        Some(section) => match section.uncompressed_data()? {
            Cow::Borrowed(b) => Ok(gimli::EndianSlice::new(b, endian)),
            Cow::Owned(b) => Ok(gimli::EndianSlice::new(arena_data.alloc(b.into()), endian)),
        },
        None => Ok(gimli::EndianSlice::new(&[][..], endian)),
    }
}

#[cfg(not(target_family = "wasm"))]
pub struct DecodedAddress {
    pub address: u64,
    pub function_name: String,
    pub location: String,
}
#[cfg(target_family = "wasm")]
#[wasm_bindgen]
pub struct DecodedAddress {
    pub address: u64,
    function_name: String,
    location: String,
}
#[cfg(target_family = "wasm")]
#[cfg_attr(target_family = "wasm", wasm_bindgen)]
impl DecodedAddress {
    #[cfg_attr(target_family = "wasm", wasm_bindgen(getter))]
    pub fn function_name(&self) -> String {
        self.function_name.clone()
    }
    #[cfg_attr(target_family = "wasm", wasm_bindgen(setter))]
    pub fn set_function_name(&mut self, function_name: String) {
        self.function_name = function_name;
    }

    #[cfg_attr(target_family = "wasm", wasm_bindgen(getter))]
    pub fn location(&self) -> String {
        self.location.clone()
    }
    #[cfg_attr(target_family = "wasm", wasm_bindgen(setter))]
    pub fn set_location(&mut self, location: String) {
        self.location = location;
    }
}

#[cfg(target_family = "wasm")]
type ReturnType = Array;
#[cfg(not(target_family = "wasm"))]
type ReturnType = Vec<DecodedAddress>;

#[cfg_attr(target_family = "wasm", wasm_bindgen)]
pub fn decode(bin: &[u8], dump: &str) -> ReturnType {
    // Prepare vector for output data
    let mut decoded_data = Vec::<DecodedAddress>::new();
    #[cfg(target_family = "wasm")]
    let format_return = |decoded_data: Vec::<DecodedAddress>| {
        decoded_data.into_iter().map(JsValue::from).collect()
    };
    #[cfg(not(target_family = "wasm"))]
    let format_return = |decoded_data| {
        decoded_data
    };

    // Used to keep slices from uncompressed section data
    let arena_data = Arena::new();

    // Parse the binary as an object file
    let object = &object::File::parse(bin);
    let object = match object {
        Ok(object) => object,
        Err(_) => return format_return(decoded_data),
    };
    let endianness = match object.endianness() {
        Endianness::Little => gimli::RunTimeEndian::Little,
        Endianness::Big => gimli::RunTimeEndian::Big
    };

    // Wrapper for the Dwarf loader
    let mut load_section = |id: gimli::SectionId| -> Result<_, _> {
        load_file_section(id, object, endianness, &arena_data)
    };

    // Load the Dwarf sections
    let dwarf = gimli::Dwarf::load(&mut load_section);
    let dwarf = match dwarf {
        Ok(dwarf) => dwarf,
        Err(_) => return format_return(decoded_data),
    };
    let ctx = Context::from_dwarf(dwarf);
    let ctx = match ctx {
        Ok(ctx) => ctx,
        Err(_) => return format_return(decoded_data),
    };

    // Match everything that looks like a program address
    let re = Regex::new(r"([0-9a-fA-F]{8})\b").unwrap();
    for cap in re.captures_iter(dump) {
        let address = u64::from_str_radix(&cap[0], 16).unwrap();
        // Look for frame that contains the address
        let mut frames = match ctx.find_frames(address) {
            Ok(frames) => frames.enumerate(),
            Err(_) => continue,
        };
        while let Some((_, frame)) = frames.next().unwrap() {
            // Skip if it doesn't point to a function
            if frame.function.is_none() {
                continue;
            }
            // Extract function name
            let func = frame.function.unwrap();
            let function_name = match func.raw_name() {
                Ok(function_name) => String::from(addr2line::demangle_auto(function_name, func.language)),
                Err(_) => "unknown_func".to_string(),
            };
            // Extract location
            let location = match frame.location {
                Some(func_location) => {
                    let file = func_location.file.unwrap_or("?");
                    let line = match func_location.line {
                        Some(line) => line.to_string(),
                        None => "?".to_string(),
                    };
                    format!("{}:{}", file, line)
                },
                None => "?:?".to_string(),
            };

            // Append the decoded address
            let decoded = DecodedAddress {
                address,
                function_name,
                location,
            };
            decoded_data.push(decoded);
        }
    }

    format_return(decoded_data)
}