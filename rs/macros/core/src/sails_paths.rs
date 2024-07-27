use std::sync::OnceLock;
use cargo_metadata::MetadataCommand;

pub(crate) struct Sails(pub(crate) &'static str);
impl Sails {
    pub(crate) fn global()->&'static Sails {
        SAILS.get_or_init(||{
            let metadata = MetadataCommand::new()
                .exec()
                .unwrap();
            if let Some(root_package)=metadata.root_package(){
                for dependency in &root_package.dependencies {
                    if dependency.name=="sails-rs"{
                        if let Some(rename)=dependency.rename.clone(){
                            let r=rename.clone();
                            return Sails(Box::leak(r.into_boxed_str()));
                        }
                    }
                }
            }
            Sails("sails_rs")
        })
    }
}
static SAILS:OnceLock<Sails>=OnceLock::new();

pub(crate) fn scale_types_path() -> syn::Path {
    syn::parse_str(Sails::global().0).unwrap()
}

pub(crate) fn scale_codec_path() -> syn::Path {
    syn::parse_str(format!("{}::scale_codec",Sails::global().0).as_str()).unwrap()
}

pub(crate) fn scale_info_path() -> syn::Path {
    syn::parse_str(format!("{}::scale_info",Sails::global().0).as_str()).unwrap()
}
