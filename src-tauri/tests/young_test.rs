use better_ustc_2_lib::rustustc::cas::client::CASClient;
use better_ustc_2_lib::rustustc::young::{YouthService, SCFilter, SecondClass};
use better_ustc_2_lib::rustustc::young::model::{Department};
use dotenv::dotenv;

#[tokio::test]
async fn test_young_full_flow() {
    dotenv().ok();

    // 1. 登录 CAS
    let client = CASClient::new();
    println!("Logging in to CAS...");
    client.login_by_pwd(None, None).await.expect("CAS login failed");

    // 2. 初始化 YouthService
    println!("Initializing Youth Service...");
    let young = YouthService::new(&client).await.expect("Youth login failed");

    // 3. 获取根部门 (测试 Tag 系统)
    println!("Fetching root department...");
    let root_dept: Department = Department::get_root_dept(&young).await.expect("Failed to get root dept");
    println!("Root Dept: {} (ID: {})", root_dept.name, root_dept.id);

    // 4. 搜索课程 (SecondClass.find)
    println!("\nSearching for courses (Limit 5)...");
    // 创建一个空的过滤器，搜索所有
    let filter = SCFilter::new(); 
    
    let courses = SecondClass::find(
        &young, 
        filter, 
        false, // apply_ended
        false, // expand_series
        5      // max
    ).await.expect("Search failed");

    println!("Found {} courses:", courses.len());
    for course in &courses {
        println!(" - [{}] {} (Status: {})", course.id, course.name, course.status().text());
        
        // 测试详细属性
        if let Some(dept) = course.department() {
             println!("   Dept: {}", dept.name);
        }
        if let Ok(time) = course.hold_time() {
             println!("   Time: {} ~ {}", time.start, time.end);
        }
    }

    // 5. 获取已参与的课程
    println!("\nFetching participated courses...");
    let my_courses = SecondClass::get_participated(&young).await.expect("Fetch participated failed");
    
    if my_courses.is_empty() {
        println!("No participated courses found.");
    } else {
        println!("Found {} participated courses:", my_courses.len());
        for course in my_courses.iter().take(3) {
            println!(" - {} (Hours: {})", course.name, course.valid_hour.unwrap_or(0.0));
        }
    }
}