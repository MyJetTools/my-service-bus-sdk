use proc_macro::TokenStream;
use quote::quote;
use types_reader::TokensObject;

pub fn generate(attr: TokenStream, input: TokenStream) -> Result<proc_macro::TokenStream, syn::Error> {
    let ast: syn::DeriveInput = syn::parse(input).unwrap();

    let attr:proc_macro2::TokenStream = attr.into();
    let attrs = TokensObject::new(attr.into(), &||None)?;

    let topic_id:&str = attrs.get_from_single_or_named("topic_id")?.try_into()?;

    let struct_name = &ast.ident;

    let result = quote!{
        #ast
        
        impl #struct_name{
            pub fn as_protobuf_bytes(&self) -> Result<Vec<u8>, prost::EncodeError> {
                let mut result = Vec::new();
                prost::Message::encode(self, &mut result)?;
                Ok(result)
            }
        
            pub fn from_protobuf_bytes(bytes: &[u8]) -> Result<Self, prost::DecodeError> {
                prost::Message::decode(bytes)
            }

        }

        impl my_service_bus::abstractions::MySbMessageSerializer for #struct_name{

            fn serialize(
                &self,
                headers: my_service_bus::abstractions::SbMessageHeaders,
            ) -> Result<(Vec<u8>, my_service_bus::abstractions::SbMessageHeaders), String> {
                match self.as_protobuf_bytes() {
                    Ok(result) => Ok((result, headers)),
                    Err(err) => Err(format!("Error serializing protobuf: {}", err)),
                }
            }

        }

        impl my_service_bus::abstractions::subscriber::MySbMessageDeserializer for #struct_name{
            type Item = Self;

            fn deserialize(bytes: &[u8], _: &my_service_bus::abstractions::SbMessageHeaders) -> Result<Self, my_service_bus::abstractions::SubscriberError> {
                match prost::Message::decode(bytes) {
                    Ok(ok) => Ok(ok),
                    Err(err) => Err(
                        my_service_bus::abstractions::SubscriberError::CanNotDeserializeMessage(format!(
                            "Error deserializing protobuf: {}",
                            err
                        )),
                    ),
                }
            }
        }

        impl my_service_bus::abstractions::GetMySbModelTopicId for #struct_name{
            fn get_topic_id() -> &'static str {
                #topic_id
            }
        }

    }.into();

    Ok(result)
}
