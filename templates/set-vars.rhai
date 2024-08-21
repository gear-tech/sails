let project_name = variable::get("project-name");

let client_project_name = if project_name.to_snake_case() == project_name {
    project_name + "_client"
} else {
    project_name + "-client"
};

variable::set("client-project-name", client_project_name);

variable::set("client_crate_name", client_project_name.to_snake_case());

variable::set("program-struct-name", project_name.to_pascal_case() + "Program");

variable::set("service-struct-name", project_name.to_pascal_case() + "Service");

variable::set("service-name", project_name.to_pascal_case());

variable::set("service-name-snake", project_name.to_snake_case());

variable::set("mocks-feature-name", "mocks");

// Set versions of used crates
variable::set("mockall-version", "0.12");
variable::set("sails-rs-version", "0.3.0");
variable::set("tokio-version", "1.39");

let with_client = variable::get("with-client");
let with_gtest = variable::get("with-gtest");
if !with_client {
    file::delete("client")
}
if !with_client || !with_gtest {
    file::delete("tests")
}