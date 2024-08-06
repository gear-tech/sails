let project_name = variable::get("project-name");

variable::set("program-name", project_name.to_pascal_case());

variable::set("program-name-snake", project_name.to_snake_case());
