pub(crate) fn to_normal_path(path: &str) -> String {
    // 1.. to remove the first \
    path[1..].replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use crate::utils::*;

    #[test]
    fn test_to_normal_path() {
        let path = to_normal_path("\\resource\\uistring\\uistring.xml");

        assert_eq!(path, "resource/uistring/uistring.xml");
    }
}
