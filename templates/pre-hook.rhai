import "../set-vars";
import "../check-version";

file::rename("Project.toml", "Cargo.toml");
file::rename("app/AppProject.toml", "app/Cargo.toml");
file::rename("client/ClientProject.toml", "client/Cargo.toml");

let with_client = variable::get("with-client");
let with_gtest = variable::get("with-gtest");
if !with_client {
    file::delete("client")
}
if !with_client || !with_gtest {
    file::delete("tests")
}
