pub(crate) fn convert_path(path: &str) -> String {
    // 1.. to remove the first \
    path[1..].replace("\\", "/")
}
