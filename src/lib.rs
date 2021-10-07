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
) -> Result<gimli::EndianSlice<'arena, Endian>, ()> {
    let name = id.name();
    match file.section_by_name(name) {
        Some(section) => match section.uncompressed_data().unwrap() {
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
    // Used to keep slices from uncompressed section data
    let arena_data = Arena::new();

    // Parse the binary as an object file
    let object = &object::File::parse(bin).unwrap();
    let endianness = match object.endianness() {
        Endianness::Little => gimli::RunTimeEndian::Little,
        Endianness::Big => gimli::RunTimeEndian::Big
    };

    // Wrapper for the Dwarf loader
    let mut load_section = |id: gimli::SectionId| -> Result<_, _> {
        load_file_section(id, object, endianness, &arena_data)
    };

    // Load the Dwarf sections
    let dwarf = gimli::Dwarf::load(&mut load_section).unwrap();
    let ctx = Context::from_dwarf(dwarf).unwrap();

    // Prepare vector for output data
    let mut decoded_data = Vec::<DecodedAddress>::new();

    let re = Regex::new(r"(40[0-9a-fA-F]{6})\b").unwrap();
    for cap in re.captures_iter(dump) {
        let addr = u64::from_str_radix(&cap[0], 16).unwrap();
        let mut frames = ctx.find_frames(addr).unwrap().enumerate();
        while let Some((_, frame)) = frames.next().unwrap() {
            if let Some(func) = frame.function {
                let location = match frame.location {
                    Some(location) => format!("{}:{}", location.file.unwrap(), location.line.unwrap()),
                    None => "?:?".to_string(),
                };

                let decoded = DecodedAddress {
                    address: addr,
                    function_name: String::from(addr2line::demangle_auto(func.raw_name().unwrap(), func.language)),
                    location: location,
                };
                decoded_data.push(decoded);
            }
        }
    }
    #[cfg(target_family = "wasm")]
    {decoded_data.into_iter().map(JsValue::from).collect()}
    
    #[cfg(not(target_family = "wasm"))]
    {decoded_data}
}