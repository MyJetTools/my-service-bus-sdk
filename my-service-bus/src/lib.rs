#[cfg(feature = "client")]
pub extern crate my_service_bus_tcp_client as client;

pub extern crate my_service_bus_abstractions as abstractions;

#[cfg(feature = "tcp_contracts")]
pub extern crate my_service_bus_tcp_shared as tcp_contracts;

#[cfg(feature = "shared")]
pub extern crate my_service_bus_shared as shared;

#[cfg(feature = "macros")]
pub extern crate my_service_bus_macros as macros;
