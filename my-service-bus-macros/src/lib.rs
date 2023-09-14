mod fn_impl_protobuf_model;

use proc_macro::TokenStream;

#[proc_macro_attribute]
pub fn my_sb_entity_protobuf_model(attr: TokenStream, item: TokenStream) -> TokenStream {
    match crate::fn_impl_protobuf_model::generate(attr, item) {
        Ok(result) => result,
        Err(err) => err.into_compile_error().into(),
    }
}
