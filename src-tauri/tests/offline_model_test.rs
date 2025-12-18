use better_ustc_2_lib::rustustc::url::generate_url;
use better_ustc_2_lib::rustustc::young::model::{Label, Module, TimePeriod};
use better_ustc_2_lib::rustustc::young::{SCFilter, SecondClass, Status};
use chrono::NaiveDateTime;
use serde_json::json;

fn sample_activity() -> SecondClass {
    SecondClass {
        id: "1".into(),
        name: "艺术赏析课".into(),
        status_code: Status::Applying.code(),
        valid_hour: Some(2.0),
        apply_num: Some(5),
        apply_limit: Some(20),
        boolean_registration: Some(0),
        need_sign_info_str: Some("0".into()),
        conceive: None,
        base_content: None,
        item_category: Some("0".into()),
        create_time_str: Some("2024-01-01 00:00:00".into()),
        apply_start: Some("2024-02-01 00:00:00".into()),
        apply_end: Some("2024-02-05 00:00:00".into()),
        start_time: Some("2024-02-10 18:00:00".into()),
        end_time: Some("2024-02-10 20:00:00".into()),
        tel: None,
        raw: json!({
            "module": "m",
            "module_dictText": "美",
            "businessDeptId": "dept-1",
            "businessDeptName": "艺术中心",
            "itemLable": "lab1,lab2",
            "lableNames": ["社团", "演出"],
        }),
    }
}

#[test]
fn time_period_overlap_and_contain() {
    let a = TimePeriod::new(
        NaiveDateTime::parse_from_str("2024-03-01 10:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        NaiveDateTime::parse_from_str("2024-03-01 12:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
    )
    .unwrap();

    let b = TimePeriod::new(
        NaiveDateTime::parse_from_str("2024-03-01 11:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        NaiveDateTime::parse_from_str("2024-03-01 11:30:00", "%Y-%m-%d %H:%M:%S").unwrap(),
    )
    .unwrap();

    assert!(a.is_overlap(&b));
    assert!(a.is_contain(&b));

    let c = TimePeriod::new(
        NaiveDateTime::parse_from_str("2024-03-01 12:30:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        NaiveDateTime::parse_from_str("2024-03-01 13:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
    )
    .unwrap();

    assert!(!a.is_overlap(&c));
    assert!(!a.is_contain(&c));
}

#[test]
fn scfilter_check_and_applyable() {
    let activity = sample_activity();

    // 名称精确匹配
    let filter = SCFilter::new().name("艺术赏析课");
    assert!(filter.check(&activity, false));

    // 模块与标签匹配
    let filter = SCFilter {
        fuzzy_name: true,
        ..SCFilter::new()
    }
    .module(Module {
        value: "m".into(),
        text: "美".into(),
    })
    .add_label(Label {
        id: "lab1".into(),
        name: "社团".into(),
    });
    assert!(filter.check(&activity, false));

    // 时间段严格包含：活动在 18-20 点，过滤 17-21 点应通过
    let time_filter = TimePeriod::new(
        NaiveDateTime::parse_from_str("2024-02-10 17:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
        NaiveDateTime::parse_from_str("2024-02-10 21:00:00", "%Y-%m-%d %H:%M:%S").unwrap(),
    )
    .unwrap();
    let filter = SCFilter {
        fuzzy_name: true,
        strict_time: true,
        time_period: Some(time_filter.clone()),
        ..SCFilter::default()
    };
    let ht = activity.hold_time().expect("hold_time should parse");
    assert!(time_filter.is_contain(&ht));
    assert!(filter.check(&activity, true));

    // applyable: 报名中 + 未报名 + 名额未满
    assert!(activity.applyable());

    // 报名结束或已报名都不可报名
    let mut closed = activity.clone();
    closed.status_code = Status::ApplyEnded.code();
    assert!(!closed.applyable());

    let mut already = activity.clone();
    already.boolean_registration = Some(1);
    assert!(!already.applyable());
}

#[test]
fn url_generation() {
    let url = generate_url("young", "item/scItem/enrolmentList");
    assert!(url.starts_with("https://young.ustc.edu.cn/item/scItem/enrolmentList"));
}
