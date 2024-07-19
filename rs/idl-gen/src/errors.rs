pub type Result<T, E = Error> = core::result::Result<T, E>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("funcion meta is invalid: {0}")]
    FuncMetaIsInvalid(String),
    #[error("event meta is invalid: {0}")]
    EventMetaIsInvalid(String),
    #[error("event meta is ambiguous: {0}")]
    EventMetaIsAmbiguous(String),
    #[error("type id `{0}` is not found in the type registry")]
    TypeIdIsUnknown(u32),
    #[error("type `{0}` is not supported")]
    TypeIsUnsupported(String),
    #[error(transparent)]
    TemplateIsBroken(#[from] Box<handlebars::TemplateError>),
    #[error(transparent)]
    RenderingFailed(#[from] Box<handlebars::RenderError>),
    #[error(transparent)]
    IoFailed(#[from] std::io::Error),
}
