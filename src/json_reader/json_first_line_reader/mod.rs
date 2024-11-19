//mod read_mode;
mod reader;
//mod states;
//pub use read_mode::*;
pub use reader::JsonFirstLineReader;
mod first_line_reader_from_slice;
pub use first_line_reader_from_slice::*;
