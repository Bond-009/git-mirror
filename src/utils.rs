use std::{
    fs::File,
    io::{Result, Write},
    path::Path
};

pub fn get_repo_name(url: &str) -> &str {
    let start = url.rfind('/').unwrap_or_default() + 1;
    if url.ends_with(".git")
    {
        return &url[start..url.len() - 4];
    }

    &url[start..]
}

pub fn set_description(path: &Path, description: &str) -> Result<()> {
    let mut path_buf = path.to_path_buf();
    path_buf.push(".git/description");
    let mut file = File::create(&path_buf)?;
    file.write_all(description.as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_repo_name_test() {
        assert_eq!(get_repo_name("https://github.com/Bond-009/git-mirror.git"), "git-mirror");
    }
}
