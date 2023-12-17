fn main() {
    #[cfg(feature = "protocol_v4")]
    let protocol_v4 = true;
    #[cfg(not(feature = "protocol_v4"))]
    let protocol_v4 = false;

    #[cfg(feature = "protocol_v6")]
    let protocol_v6 = true;
    #[cfg(not(feature = "protocol_v6"))]
    let protocol_v6 = false;

    if protocol_v4 && protocol_v6 {
        panic!("Features 'protocol_v4' and 'protocol_v6' cannot be enabled at the same time");
    }

    if !protocol_v4 && !protocol_v6 {
        panic!("One of 'protocol_v4' or 'protocol_v6' must be enabled");
    }
}
