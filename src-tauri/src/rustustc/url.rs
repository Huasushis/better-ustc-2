use url::Url;

pub fn generate_url(website: &str, path: &str) -> String {
    let base = match website {
        "id" => "https://id.ustc.edu.cn",
        "edu_system" => "https://jw.ustc.edu.cn",
        "young" => "https://young.ustc.edu.cn",
        _ => panic!("Unknown website key: {}", website),
    };

    let base_url = Url::parse(base).expect("Invalid base URL");
    base_url.join(path).expect("Invalid URL path").to_string()
}
