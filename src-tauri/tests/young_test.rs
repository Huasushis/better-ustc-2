use better_ustc_2_lib::recommend::Recommender;
use better_ustc_2_lib::rustustc::cas::client::CASClient;
use better_ustc_2_lib::rustustc::young::model::Department;
use better_ustc_2_lib::rustustc::young::{SCFilter, SecondClass, YouthService};
use dotenv::dotenv;
use serde_json::json;

#[tokio::test]
#[ignore = "requires real USTC CAS credentials and network access"]
async fn test_young_full_flow() {
    dotenv().ok();

    // 1. 登录 CAS
    let client = CASClient::new();
    println!("Logging in to CAS...");
    client
        .login_by_pwd(None, None)
        .await
        .expect("CAS login failed");

    // 2. 初始化 YouthService
    println!("Initializing Youth Service...");
    let young = YouthService::new(&client)
        .await
        .expect("Youth login failed");

    let user = better_ustc_2_lib::rustustc::young::model::User::get_current(&young)
        .await
        .expect("Failed to get user info");

    println!("Logged in as: {} (ID: {})", user.name, user.id);
    let sign_info = better_ustc_2_lib::rustustc::young::model::SignInfo::get_self(&young)
        .await
        .expect("Failed to get sign info");
    let json_info = json!(sign_info);
    println!("Sign Info: {}", json_info);

    // 3. 获取根部门 (测试 Tag 系统)
    println!("Fetching root department...");
    let root_dept: Department = Department::get_root_dept(&young)
        .await
        .expect("Failed to get root dept");
    println!("Root Dept: {} (ID: {})", root_dept.name, root_dept.id);

    // 4. 搜索课程 (SecondClass.find)
    println!("\nSearching for courses (Limit 5)...");
    // 创建一个空的过滤器，搜索所有
    let filter = SCFilter::new();

    let courses = SecondClass::find(
        &young, filter, false, // apply_ended
        false, // expand_series
        5,     // max
    )
    .await
    .expect("Search failed");

    println!("Found {} courses:", courses.len());
    for mut course in courses {
        course.update(&young).await.expect("Update failed");
        println!(
            " - [{}] {} (Status: {})",
            course.id,
            course.name,
            course.status().text()
        );

        // 测试详细属性
        if let Some(dept) = course.department() {
            println!("   Dept: {}", dept.name);
        }
        if let Ok(time) = course.hold_time() {
            println!("   Time: {} ~ {}", time.start, time.end);
        }
        if let Some(module) = course.module() {
            println!("   Module: {}", module.value);
        }
        if let Some(popularity) = course.apply_num {
            println!("   Applied by {} users", popularity);
        }
        //print raw
        // println!("   Raw: {}", course.raw);
    }

    // 5. 获取已参与的课程
    println!("\nFetching participated courses...");
    let my_courses = SecondClass::get_participated(&young)
        .await
        .expect("Fetch participated failed");

    if my_courses.is_empty() {
        println!("No participated courses found.");
    } else {
        println!("Found {} participated courses:", my_courses.len());
        for course in my_courses.iter().take(3) {
            println!(
                " - {} (Hours: {})",
                course.name,
                course.valid_hour.unwrap_or(0.0)
            );
        }
    }

    //6. Get recommended courses
    println!("\nGetting recommended courses (Limit 5)...");
    let recommended = Recommender::recommend(&young, 5)
        .await
        .expect("Recommendation failed");
    println!("Recommended {} courses:", recommended.len());
    for course in &recommended {
        println!(" - [{}] {}", course.id, course.name);
    }
}
