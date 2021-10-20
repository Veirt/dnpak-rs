pub(crate) fn to_normal_path(path: &str) -> String {
    // 1.. to remove the first \
    path[1..].replace("\\", "/")
}
