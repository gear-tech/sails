fn main() {
    csbindgen::Builder::default()
        .input_extern_file("src/lib.rs")
        .csharp_dll_name("sails_net_client_gen")
        .csharp_namespace("Sails.ClientGenerator")
        .generate_csharp_file("../../src/Sails.ClientGenerator/NativeMethods.g.cs")
        .unwrap();
}
