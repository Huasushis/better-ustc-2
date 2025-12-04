use better_ustc_2_lib::rustustc::cas::client::CASClient;
use dotenv::dotenv;

#[tokio::test]
async fn test_login_and_get_info() {
    // 1. 加载 .env 文件中的环境变量
    dotenv().ok();

    // 2. 初始化客户端
    let client = CASClient::new();

    // 3. 尝试登录 (如果环境变量没设，这里会报错)
    println!("Starting login...");
    let login_result = client.login_by_pwd(None, None).await;

    match login_result {
        Ok(_) => println!("Login successfully!"),
        Err(e) => panic!("Login failed: {:?}", e),
    }

    // 4. 检查登录状态
    let is_login = client.is_login().await;
    assert!(is_login, "Should be logged in");

    // 5. 获取用户信息
    println!("Fetching user info...");
    let info = client.get_info().await;
    match info {
        Ok(user_info) => {
            println!("User Info Retrieved:");
            println!("ID: {}", user_info.id);
            println!("Name: {}", user_info.name);
            println!("GID: {}", user_info.gid);
            println!("Email: {:?}", user_info.email);
        }
        Err(e) => panic!("Failed to get info: {:?}", e),
    }
}
