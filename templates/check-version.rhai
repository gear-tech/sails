//
// Check min version of 'sails-cli'
//
let sails_cli_min_version_str = "0.4.0";
let sails_cli_min_version = parse_sem_ver(sails_cli_min_version_str);

let sails_cli_version_str = if variable::is_set("sails-cli-version") {
    variable::get("sails-cli-version")
} else {
    "0"
};
let sails_cli_version = parse_sem_ver(sails_cli_version_str);

if sails_cli_version[0] < sails_cli_min_version[0]
    || sails_cli_version[1] < sails_cli_min_version[1]
    || sails_cli_version[2] < sails_cli_min_version[2] {
    abort("Sails CLI version '" + sails_cli_version_str + "' is not supported. Please upgrade to version '" + sails_cli_min_version_str + "' or higher");
}

//
// Check min version of 'sails-rs'
//
let sails_min_version_str = "0.6.3";
let sails_min_version = parse_sem_ver(sails_min_version_str);

let sails_max_version_str = variable::get("sails-rs-version");
let sails_max_version = parse_sem_ver(sails_max_version_str);

let sails_version_str = if variable::is_set("sails-version") {
    variable::get("sails-version")
} else {
    ""
};
let sails_version = if sails_version_str == "" {
    sails_max_version
} else {
    parse_sem_ver(sails_version_str)
};

if is_sem_ver_less(sails_version, sails_min_version) {
    abort(`Sails version '${sails_version_str}' is not supported. Please specify version equal to or higher than '${sails_min_version_str}'`);
}

if is_sem_ver_less(sails_max_version, sails_version) {
    abort(`Sails version '${sails_version_str}' is not supported. Please specify version equal to or lower than '${sails_max_version_str}'`);
}

variable::set("sails-rs-version", `${sails_version[0]}.${sails_version[1]}.${sails_version[2]}`);

println("Sails version: " + variable::get("sails-rs-version"));

fn parse_sem_ver(version) {
    let dot_idx = 0;
    let dot_idx_next = index_of(version, ".", dot_idx);
    let major = if dot_idx_next == -1 {
        version.sub_string(dot_idx)
    } else {
        version.sub_string(dot_idx..dot_idx_next)
    };
    major = parse_int(major);

    dot_idx = dot_idx_next + 1;
    dot_idx_next = index_of(version, ".", dot_idx);
    let minor = if dot_idx == 0 {
        "0"
    } else if dot_idx_next == -1 {
        version.sub_string(dot_idx)
    } else {
        version.sub_string(dot_idx..dot_idx_next)
    };
    minor = parse_int(minor);

    dot_idx = dot_idx_next + 1;
    dot_idx_next = index_of(version, ".", dot_idx);
    let patch = if dot_idx == 0 {
        "0"
    } else if dot_idx_next == -1 {
        version.sub_string(dot_idx)
    } else {
        abort("Version '" + version + "' does not conform to semantic versioning");
    };
    patch = parse_int(patch);

    [major, minor, patch]
}

fn is_sem_ver_less(version1, version2) {
    if version1[0] < version2[0] {
        return true;
    } else if version1[0] == version2[0] && version1[1] < version2[1] {
        return true;
    } else if version1[0] == version2[0] && version1[1] == version2[1] && version1[2] < version2[2] {
        return true;
    }
    false
}
