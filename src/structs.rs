use crate::magic::Magic;

// pub struct MagicEntry {
//     mp: Magic,
//     cont_count: u32,
//     max_count: u32,
// }
//
// pub struct MagicEntrySet {
//     me: Box<MagicEntry>,
//     count: u32,
//     max: u32,
// }
//
// #[derive(Default)]
// pub enum LoaderType {
//     #[default]
//     User,
//     Malloc,
//     MMap,
// }
//
#[derive(Default)]
pub struct MagicMap {
    pub left: Vec<Magic>,
    pub right: Vec<Magic>,
}
//
// pub struct MagicList {
//     magic: Vec<Magic>,
//     regexps: Vec<Vec<u8>>,
//     nmagic: usize,
//     // void* map - internal resources used by entry
//     // mlist* next
//     // mlist* prev
// }
//
// pub struct LevelInfo {
//     offset: i32,
//     got_match: i32,
//     last_match: i32,
//     last_cond: i32,
// }
//
// pub struct Context {
//     len: usize,
//     level_infos: Vec<LevelInfo>,
// }
//
// pub struct Search {
//     index: usize,
//     search_length: usize,
//     offset: usize,
//     match_length: usize,
// }
//
// pub struct MagicSet {
//     mlist: MagicList,
//     context: Context,
//     output: String,
//     offset: u32,
//     eoffset: u32,
//     error: i32,
//     flags: i32,
//     event_flags: i32,
//     filename: String,
//     line: usize,
//     mode: String, // Probably not a string
//     warnings: u16,
//     search: Search,
//     match_value: Value,
//     indirection_max: u16,
//     name_max: u16,
//     elf_shnum_max: u16,
//     elf_phnum_max: u16,
//     elf_notes_max: u16,
//     regex_max: u16,
//     num_warnings_max: u16,
//     bytes_max: usize,
//     encoding_max: usize,
//     elf_shsize_max: usize,
// }
