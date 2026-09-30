//! todo: flatbuffer module doc

mod flatbuffer {
    include!(concat!(env!("OUT_DIR"), "/footer_generated.rs"));
}

pub(crate) mod footer {
    //! todo: flatbuffer footer module doc
    pub(crate) use self::super::flatbuffer::minknow::reads_format::{
        Footer,
        FooterArgs,
        FooterBuilder,
        FooterOffset
    };
}
pub(crate) mod embedded_file {
    //! todo: flatbuffer embedded_file module doc
    pub(crate) use self::super::flatbuffer::minknow::reads_format::{
        EmbeddedFile,
        EmbeddedFileArgs,
        EmbeddedFileOffset,
        EmbeddedFileBuilder,
    };
}
pub(crate) use self::flatbuffer::minknow::reads_format::{
    ContentType,
    Format,
};
