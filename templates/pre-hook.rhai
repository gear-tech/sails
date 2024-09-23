import "../set-vars";
import "../check-version";

file::rename("CargoTemplate.toml", "Cargo.toml");
if file::exists("app/CargoTemplate.toml") {
    file::rename("app/CargoTemplate.toml", "app/Cargo.toml");
}
file::rename("client/CargoTemplate.toml", "client/Cargo.toml");

let with_client = variable::get("with-client");
let with_gtest = variable::get("with-gtest");
if !with_client {
    file::delete("client")
}
if !with_client || !with_gtest {
    file::delete("tests")
}
