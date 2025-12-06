use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use anyhow::{Result, Context, bail};
use chrono::NaiveDateTime;
use crate::rustustc::young::service::YouthService;

// ==================== 基础类型定义 (TimePeriod) ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePeriod {
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
}

impl TimePeriod {
    pub fn new(start: NaiveDateTime, end: NaiveDateTime) -> Result<Self> {
        if start > end {
            bail!("The start time should be earlier than the end time");
        }
        Ok(Self { start, end })
    }

    pub fn parse(s: &str) -> Result<NaiveDateTime> {
        NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")
            .context("Failed to parse datetime")
    }

    pub fn parse_period(start_str: &str, end_str: Option<&str>) -> Result<Self> {
        let start = Self::parse(start_str)?;
        let end = if let Some(e) = end_str {
            Self::parse(e)?
        } else {
            start
        };
        Self::new(start, end)
    }

    pub fn is_contain(&self, other: &TimePeriod) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    pub fn is_overlap(&self, other: &TimePeriod) -> bool {
        self.start <= other.end && self.end >= other.start
    }
}

// ==================== Tag 系统 (Module, Department, Label) ====================

#[async_trait::async_trait]
pub trait Tag: Sized {
    fn get_url() -> &'static str;
    fn from_dict(data: Value) -> Result<Self>;
    
    async fn get_available_tags(service: &YouthService) -> Result<Vec<Self>> {
        let raw_list = service.get_result(Self::get_url(), None).await?;
        let list_val = raw_list.as_array().context("API response is not an array")?;
        
        let mut tags = Vec::new();
        for v in list_val {
            tags.push(Self::from_dict(v.clone())?);
        }
        Ok(tags)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Module {
    pub value: String,
    pub text: String,
}

#[async_trait::async_trait]
impl Tag for Module {
    fn get_url() -> &'static str { "sys/dict/getDictItems/item_module" }
    fn from_dict(data: Value) -> Result<Self> {
        Ok(Self {
            value: data["value"].as_str().unwrap_or_default().to_string(),
            text: data["text"].as_str().unwrap_or_default().to_string(),
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Department {
    pub id: String,
    #[serde(rename = "departName")]
    pub name: String,
    #[serde(skip)]
    pub children: Vec<Department>,
    #[serde(skip)]
    pub level: i32,
}

impl Department {
    fn from_dict_recursive(data: Value, level: i32) -> Result<Self> {
        let id = data["id"].as_str().unwrap_or_default().to_string();
        let name = data["departName"].as_str().unwrap_or_default().to_string();
        
        let mut children = Vec::new();
        if let Some(child_arr) = data.get("children").and_then(|c| c.as_array()) {
            for c in child_arr {
                children.push(Self::from_dict_recursive(c.clone(), level + 1)?);
            }
        }
        Ok(Self { id, name, children, level })
    }

    pub async fn get_root_dept(service: &YouthService) -> Result<Self> {
        let tags = Self::get_available_tags(service).await?;
        tags.into_iter().next().context("No root department found")
    }

    pub fn find<'a>(&'a self, name: &str, max_level: i32) -> Vec<&'a Department> {
        let mut result = Vec::new();
        if max_level != -1 && self.level > max_level {
            return result;
        }
        if self.name.contains(name) {
            result.push(self);
        }
        for child in &self.children {
            result.extend(child.find(name, max_level));
        }
        result
    }

    pub fn find_one<'a>(&'a self, name: &str, max_level: i32) -> Option<&'a Department> {
        self.find(name, max_level).into_iter().next()
    }
}

#[async_trait::async_trait]
impl Tag for Department {
    fn get_url() -> &'static str { "sysdepart/sysDepart/queryTreeList" }
    fn from_dict(data: Value) -> Result<Self> {
        Self::from_dict_recursive(data, 0)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Label {
    pub id: String,
    pub name: String,
}

#[async_trait::async_trait]
impl Tag for Label {
    fn get_url() -> &'static str { "paramdesign/scLabel/queryListLabel" }
    fn from_dict(data: Value) -> Result<Self> {
        Ok(Self {
            id: data["id"].as_str().unwrap_or_default().to_string(),
            name: data["name"].as_str().unwrap_or_default().to_string(),
        })
    }
}

// ==================== 过滤器 SCFilter ====================

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SCFilter {
    pub name: String,
    pub time_period: Option<TimePeriod>,
    pub module: Option<Module>,
    pub department: Option<Department>,
    pub labels: Vec<Label>,
    pub fuzzy_name: bool,
    pub strict_time: bool,
}

impl SCFilter {
    pub fn new() -> Self { Self::default() }
    pub fn name(mut self, name: &str) -> Self { self.name = name.to_string(); self }
    pub fn module(mut self, module: Module) -> Self { self.module = Some(module); self }
    pub fn department(mut self, dept: Department) -> Self { self.department = Some(dept); self }
    pub fn time_period(mut self, period: TimePeriod) -> Self { self.time_period = Some(period); self }
    pub fn strict_time(mut self, strict: bool) -> Self { self.strict_time = strict; self }
    pub fn add_label(mut self, label: Label) -> Self { self.labels.push(label); self }

    pub fn to_params(&self) -> Value {
        let mut params = json!({});
        if !self.name.is_empty() { params["itemName"] = json!(self.name); }
        if let Some(m) = &self.module { params["module"] = json!(m.value); }
        if let Some(d) = &self.department { params["businessDeptId"] = json!(d.id); }
        if !self.labels.is_empty() {
            let ids: Vec<String> = self.labels.iter().map(|l| l.id.clone()).collect();
            params["itemLable"] = json!(ids.join(","));
        }
        params
    }

    pub fn check(&self, sc: &SecondClass, only_strict: bool) -> bool {
        if !only_strict {
            if self.fuzzy_name && !sc.name.to_lowercase().contains(&self.name.to_lowercase()) { return false; }
            if !self.fuzzy_name && sc.name != self.name { return false; }
            
            if let (Some(m), Some(sc_m)) = (&self.module, sc.module()) {
                if sc_m.value != m.value { return false; }
            }
            if let (Some(d), Some(sc_d)) = (&self.department, sc.department()) {
                if sc_d.id != d.id { return false; }
            }
            if !self.labels.is_empty() {
                let sc_labels = sc.labels();
                if !self.labels.iter().any(|t| sc_labels.iter().any(|s| s.id == t.id)) { return false; }
            }
        }

        if !self.fuzzy_name && sc.name != self.name { return false; }
        if let Some(period) = &self.time_period {
            if let Ok(ht) = sc.hold_time() {
                if self.strict_time {
                    if !period.is_contain(&ht) { return false; }
                } else {
                    if !period.is_overlap(&ht) { return false; }
                }
            }
        }
        true
    }
}

// ==================== User & SignInfo ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: String,
    #[serde(rename = "realname")]
    pub name: String,
    #[serde(rename = "sex_dictText")]
    pub gender: String,
    pub avatar: Option<String>,
    pub grade: String,
    pub college: Option<String>,
    pub classes: String,
    #[serde(rename = "scientificqiValue")]
    pub scientific_value: i32,
    pub birthday: Option<String>,
    #[serde(skip)]
    pub phone: Option<String>,
}

impl User {
    pub async fn get_phone(&mut self, service: &YouthService) -> Result<Option<String>> {
        if self.phone.is_some() { return Ok(self.phone.clone()); }
        let url = "sys/user/querySysUser";
        let params = json!({ "username": self.id });
        match service.get_result(url, Some(params)).await {
            Ok(v) => {
                println!("User info response: {:?}", v);
                let p = v["phone"].as_str().map(|s| s.to_string());
                self.phone = p.clone();
                Ok(p)
            },
            Err(e) if e.to_string().contains("验证失败") => { self.phone = None; Ok(None) },
            Err(e) => Err(e),
        }
    }

    pub async fn get_current(service: &YouthService) -> Result<Self> {
        let info = service.get_result("paramdesign/scMyInfo/info", None).await?;
        let id = info["username"].as_str().context("Missing username")?;
        let phone = info["phone"].as_str().map(|s| s.to_string());
        let mut user = Self::find(service, id, 2, 2).await?.into_iter().next().context("Failed to find self user info")?;
        if let Some(p) = phone { user.phone = Some(p); }
        Ok(user)
    }

    pub async fn find(service: &YouthService, name_or_id: &str, max: i32, size: i32) -> Result<Vec<User>> {
        let url = "sys/user/getPersonInChargeUser";
        let params = json!({ "realname": name_or_id });
        let raw = service.page_search(url, params, max, size).await?;
        Ok(raw.into_iter().map(|v| serde_json::from_value(v)).collect::<Result<Vec<_>, _>>()?)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignInfo {
    pub college: String,
    pub classes: String,
    pub phone: String,
    pub email: String,
    pub remarks: String,
}

impl SignInfo {
    pub async fn get_self(service: &YouthService) -> Result<Self> {
        let mut user = User::get_current(service).await?;
        let phone = user.get_phone(service).await?.unwrap_or_default();
        Ok(Self {
            college: user.college.unwrap_or_default(),
            classes: user.classes,
            phone,
            email: String::new(),
            remarks: String::new(),
        })
    }
}

// ==================== Status ====================

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum Status {
    Applying = 26,
    ApplyEnded = 28,
    HourPublic = 30,
    HourAppendPublic = 31,
    PublicEnded = 32,
    HourApplying = 33,
    HourApproved = 34,
    HourRejected = 35,
    Finished = 40,
    AbnormalFinished = -3,
    Unknown = -1,
}

impl Status {
    pub fn code(&self) -> i32 { *self as i32 }
    pub fn text(&self) -> &'static str {
        match self {
            Status::Applying => "报名中",
            Status::ApplyEnded => "报名已结束",
            Status::HourPublic => "学时公示中",
            Status::HourAppendPublic => "追加学时公示",
            Status::PublicEnded => "公示已结束",
            Status::HourApplying => "学时申请中",
            Status::HourApproved => "学时审核通过",
            Status::HourRejected => "学时驳回",
            Status::Finished => "结项",
            Status::AbnormalFinished => "异常结项",
            Status::Unknown => "未知状态",
        }
    }
}

impl From<i32> for Status {
    fn from(code: i32) -> Self {
        match code {
            26 => Status::Applying,
            28 => Status::ApplyEnded,
            30 => Status::HourPublic,
            31 => Status::HourAppendPublic,
            32 => Status::PublicEnded,
            33 => Status::HourApplying,
            34 => Status::HourApproved,
            35 => Status::HourRejected,
            40 => Status::Finished,
            -3 => Status::AbnormalFinished,
            _ => Status::Unknown,
        }
    }
}

// ==================== SecondClass (核心逻辑) ====================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecondClass {
    pub id: String,
    #[serde(rename = "itemName")]
    pub name: String,
    #[serde(rename = "itemStatus")]
    pub status_code: i32,
    #[serde(rename = "validHour")]
    pub valid_hour: Option<f64>,
    #[serde(rename = "applyNum")]
    pub apply_num: Option<i32>,
    #[serde(rename = "peopleNum")]
    pub apply_limit: Option<i32>,
    #[serde(rename = "booleanRegistration")]
    pub boolean_registration: Option<i32>,
    #[serde(rename = "needSignInfo")]
    pub need_sign_info_str: Option<String>,
    #[serde(rename = "conceive")]
    pub conceive: Option<String>,
    #[serde(rename = "baseContent")]
    pub base_content: Option<String>,
    #[serde(rename = "itemCategory")]
    pub item_category: Option<String>, // "1" 为系列活动
    
    // Time
    #[serde(rename = "createTime")]
    pub create_time_str: Option<String>,
    #[serde(rename = "applySt")]
    pub apply_start: Option<String>,
    #[serde(rename = "applyEt")]
    pub apply_end: Option<String>,
    #[serde(rename = "st")]
    pub start_time: Option<String>,
    #[serde(rename = "et")]
    pub end_time: Option<String>,

    #[serde(rename = "tel")]
    pub tel: Option<String>,

    #[serde(flatten)]
    pub raw: Value,
}

// maybe more objects:
// raw.pic : The header image URL: "https://young.ustc.edu.cn/login/{pic}"
// (about department): as follows
// An raw field example:
/*
{
    "publicEndTime": null,
    "examineStatusName": null,
    "applyEt": "2025-12-21 18:45:00",
    "enshrineNum": null,
    "type": 1,
    "examineStatus": 10,
    "marathonDataStatus": null,
    "sumHours": {
        "source": "0.00",
        "parsedValue": 0
    },
    "evaluation": 0,
    "itemName": "“聆冬映雪”首届古琴音乐会",
    "actionStatus": null,
    "historyType": "000",
    "review": "0",
    "tel": "13167733518",
    "stick": "1",
    "id": "7697fe4c3f4a4b35c8f4e553a72fbc7f",
    "evaluated": null,
    "enshrine": null,
    "backOaBudget": 0,
    "itemLimitNum": null,
    "sumPersons": 0,
    "outlayDetail": null,
    "module": "m",
    "outlayMoney": {
        "source": "7006.90",
        "parsedValue": 7006.9
    },
    "createet": null,
    "regRemarks": null,
    "version": null,
    "attaType": "",
    "serviceHour": "2.0",
    "qrSigninClosed": "0",
    "auditTime": "2025-12-01 12:17:44",
    "itemLable": "1296303965593841666",
    "signInType": null,
    "businessDeptName": "学生正则古琴协会",
    "itemCategory": "0",
    "hosting": "1",
    "baseContentCountNum": null,
    "signOutType": null,
    "delFlag": 0,
    "holdingPeriod": null,
    "proposalId": null,
    "examineStatus_dictText": "发布",
    "marathonCampus": null,
    "regPicUrls": null,
    "auditAssistant": "",
    "qdCourseStatus": null,
    "identity": null,
    "budgetList": null,
    "showWininfo": "0",
    "processType": 0,
    "st": "2025-12-21 19:00:00",
    "sponsor_dictText": "学生社团管理指导委员会",
    "needPlaceApply": "1",
    "qdClassId": null,
    "regOptions": null,
    "needSignInfo": "0",
    "updateTime": "2025-12-03 11:15:39",
    "applyNum": 0,
    "delAudit": 1,
    "canSubmitWork": false,
    "uuidKey": null,
    "businessDeptId": "4dcaf351b5c74a06b4503d219acb6280",
    "teamSize": null,
    "createTime": "2025-11-30 16:03:11",
    "organizer": "4dcaf351b5c74a06b4503d219acb6280",
    "isKnot": 0,
    "registrationStatus": null,
    "signInfo": 0,
    "form_dictText": "现场参与",
    "pid": "-1",
    "pic": "group1/M00/2D/EA/wKgUEWkr9-aAbVjdAAFQ0EMsZQE288.jpg",
    "baseContent": "&nbsp; &nbsp; &nbsp; &nbsp;时维亚岁，序属玄英。长至初阳生九地，葭灰始动；深庭素雪映冰弦，梅影将横。正则琴社谨以清商雅意，拟于冬至之夜（12月21日晚19:00），于东区水上报告厅，特备首届「聆冬映雪」古琴音乐会，邀诸君共赴林泉之约。 <br /><br />&nbsp; &nbsp; &nbsp; &nbsp;是夜也，炉暖松烟，窗含玉尘。《飞雪玉花》，启琼英之曼舞；《流水》 汤汤，写寒涧之幽淙。《平沙落雁》，寄遥思于霜浦；《良宵》 泠然，契冰心于月穹。更有《阳关三叠》，诉尽故人之谊；《酒狂》逸兴，抒怀物外之风。七弦吐纳，合天地之呼吸；宫商应和，通古今之消息。 <br /><br />&nbsp; &nbsp; &nbsp; &nbsp;琴社敬备种子门票一份，奉予诸君。愿君携归，植于案头盆内，待得东风吹拂，便可萌叶开花，以此冬日之清音，换彼春朝之烂漫。又设&ldquo;拈喜抽奖&rdquo;之趣，于中场暂歇之时，锦匣藏珍，待有缘者抽取，聊佐清欢，以志雅念。<br /><br />&nbsp; &nbsp; &nbsp; &nbsp;诚邀雅客，各携素心。扫竹径以迎鹤驾，煨芋炉而待清谈。愿借太古遗音，涤尘襟于三叠；且凭钧天妙响，寄幽思于九嶷。",
    "putaway": "0",
    "linkMan": "李孝诚",
    "organizer_dictText": "学生正则古琴协会",
    "totalServiceHour": null,
    "stick_dictText": "已置顶",
    "placeInfo": "东区水上报告厅",
    "activityLevel": "school",
    "processInstanceId": "51837123",
    "applyTeamNum": null,
    "libId": null,
    "rangeDeptIds": null,
    "marathon": null,
    "qdClassIndex": null,
    "peopleNum": 250,
    "canGivenHours": false,
    "modules": null,
    "et": "2025-12-21 21:00:00",
    "needPlaceApply_dictText": "需要",
    "labelTo": null,
    "lastApprovalMan": "P0812",
    "isAuditAssistant": null,
    "ewSponsor": "",
    "itemStatus_dictText": "报名中",
    "nj": "",
    "applyStatus": 26,
    "sponsor": "4f936144c0b84acc8793411121528c0d",
    "workStatus_dictText": null,
    "planningAtta": "group1/M00/2D/EA/wKgUEWkr-4yAbjkFAAB3NBaWmPY71.docx",
    "isShowShare": 0,
    "cancelSign": 0,
    "itemPlaceDTO": {
        "itemId": "7697fe4c3f4a4b35c8f4e553a72fbc7f",
        "places": [
            {
                "itemId": null,
                "placeSt": null,
                "createBy": "PB23051008",
                "createTime": 1764562739000,
                "updateBy": "PB23051008",
                "placeInfo": "东区水上报告厅",
                "updateTime": 1764731737000,
                "id": "39b36923262f251cf5385ba661444616",
                "placeEt": null
            }
        ]
    },
    "conceive": "为&ldquo;跃动青春&rdquo;文艺季专项活动，12.21晚19:00在东区水上报告厅开展古琴演出，主要演出人员为社团成员、校内其他社团成员。",
    "validHour": {
        "source": "2.00",
        "parsedValue": 2
    },
    "itemCategory_dictText": "单次项目",
    "duration": {
        "source": "2.0",
        "parsedValue": 2
    },
    "businessDeptId_dictText": "学生正则古琴协会",
    "updateBy": "PB23051008",
    "itemStatus": 26,
    "applySt": "2025-11-30 17:00:00",
    "module_dictText": "美",
    "teamNum": null,
    "createst": null,
    "qrSignoffClosed": "0",
    "booleanRegistration": 0,
    "workStatus": null,
    "hours": null,
    "conceiveCountNum": null,
    "attaEndTime": null,
    "activityLevel_dictText": "校级",
    "budgetProjectId": null,
    "createBy": "PB23051008",
    "form": "0",
    "applyWay": null,
    "needApply": "1",
    "xq": null,
    "departTo": null,
    "applyRange": "0",
    "partakeNum": 0
}
*/

impl SecondClass {
    pub fn status(&self) -> Status { Status::from(self.status_code) }

    pub fn create_time(&self) -> Result<NaiveDateTime> {
        TimePeriod::parse(self.create_time_str.as_deref().unwrap_or(""))
    }
    
    pub fn apply_time(&self) -> Result<TimePeriod> {
        TimePeriod::parse_period(self.apply_start.as_deref().unwrap_or(""), self.apply_end.as_deref())
    }
    
    pub fn hold_time(&self) -> Result<TimePeriod> {
        TimePeriod::parse_period(self.start_time.as_deref().unwrap_or(""), self.end_time.as_deref())
    }

    pub fn applied(&self) -> bool {
        self.boolean_registration.unwrap_or(0) == 1
    }

    pub fn applyable(&self) -> bool {
        self.status() == Status::Applying 
            && !self.applied() 
            && self.apply_num.unwrap_or(0) < self.apply_limit.unwrap_or(0)
    }

    pub fn need_sign_info(&self) -> bool {
        self.need_sign_info_str.as_deref() == Some("1")
    }

    pub fn is_series(&self) -> bool {
        self.item_category.as_deref() == Some("1")
    }

    pub fn module(&self) -> Option<Module> {
        Some(Module {
            value: self.raw.get("module")?.as_str()?.to_string(),
            text: self.raw.get("module_dictText")?.as_str()?.to_string(),
        })
    }

    pub fn department(&self) -> Option<Department> {
        Some(Department {
            id: self.raw.get("businessDeptId")?.as_str()?.to_string(),
            //businessDeptId_dictText, businessDeptName, bussinessDeptName choose one not none
            // name: self.raw.get("businessDeptName")?.as_str()?.to_string(),
            // how can I implement it?
            // Try businessDeptId_dictText first, then businessDeptName, then bussinessDeptName
            name: ["businessDeptId_dictText", "businessDeptName", "bussinessDeptName"]
                .iter()
                .find_map(|&key| self.raw.get(key).and_then(|v| v.as_str()))
                .unwrap_or_default()
                .to_string(),
            children: vec![],
            level: -1,
        })
    }

    pub fn labels(&self) -> Vec<Label> {
        let mut result = Vec::new();
        if let (Some(ids), Some(names)) = (
            self.raw.get("itemLable").and_then(|v| v.as_str()),
            self.raw.get("lableNames").and_then(|v| v.as_array())
        ) {
            let id_list: Vec<&str> = ids.split(',').collect();
            for (i, id) in id_list.iter().enumerate() {
                if let Some(name) = names.get(i).and_then(|v| v.as_str()) {
                    result.push(Label { id: id.to_string(), name: name.to_string() });
                }
            }
        }
        result
    }

    // === 对应 Python @cached_property children ===
    pub async fn get_children(&self, service: &YouthService) -> Result<Vec<SecondClass>> {
        if !self.is_series() { return Ok(vec![]); }
        
        let url = "item/scItem/selectSignChirdItem";
        let params = json!({ "id": self.id });
        
        // get_result 返回的是 list
        let raw_list = service.get_result(url, Some(params)).await?;
        let list_val = raw_list.as_array().context("Children response is not array")?;
        
        let mut children = Vec::new();
        for v in list_val {
            children.push(serde_json::from_value(v.clone())?);
        }
        Ok(children)
    }

    // === 内部辅助 fetch ===
    async fn fetch(service: &YouthService, filter: &SCFilter, url: &str, size: i32) -> Result<Vec<SecondClass>> {
        let raw_list = service.page_search(url, filter.to_params(), -1, size).await?;
        let mut result = Vec::new();
        for v in raw_list {
            let sc: SecondClass = serde_json::from_value(v)?;
            if filter.check(&sc, true) {
                result.push(sc);
            }
        }
        Ok(result)
    }

    // === 核心 Find 方法 (支持 expand_series) ===
    pub async fn find(
        service: &YouthService, 
        filter: SCFilter, 
        apply_ended: bool,
        expand_series: bool,
        mut max: i32
    ) -> Result<Vec<SecondClass>> {
        if max == 0 { return Ok(vec![]); }
        
        let endpoint = if apply_ended { "item/scItem/endList" } else { "item/scItem/enrolmentList" };
        
        // 1. 获取基础列表
        let base_list = Self::fetch(service, &filter, endpoint, 20).await?;
        
        let mut result = Vec::new();
        
        for sc in base_list {
            if expand_series && sc.is_series() {
                // 如果是系列课程，且要求展开
                let children = sc.get_children(service).await?;
                for child in children {
                    // Python逻辑: (apply_ended ^ (i.status == Status.APPLYING))
                    let is_applying = child.status() == Status::Applying;
                    let status_ok = if apply_ended { !is_applying } else { is_applying };
                    
                    if filter.check(&child, true) && status_ok {
                        result.push(child);
                        max -= 1;
                        if max == 0 { break; }
                    }
                }
            } else {
                // 普通课程，直接添加
                result.push(sc);
                max -= 1;
            }
            
            if max == 0 { break; }
        }
        
        Ok(result)
    }
    
    pub async fn get_participated(service: &YouthService) -> Result<Vec<SecondClass>> {
        let url = "item/scParticipateItem/list";
        let raw = service.page_search(url, json!({}), -1, 20).await?;
        let list = raw.into_iter().map(|v| serde_json::from_value(v)).collect::<Result<Vec<_>,_>>()?;
        Ok(list)
    }
    
    pub async fn apply(&self, service: &YouthService, force: bool, auto_cancel: bool, sign_info: Option<SignInfo>) -> Result<bool> {
        if !force && !self.applyable() { return Ok(false); }
        
        let url = format!("mobile/item/enter/{}", self.id);
        let json_body = if self.need_sign_info() {
            let info = match sign_info {
                Some(s) => s,
                None => SignInfo::get_self(service).await?,
            };
            json!(info)
        } else {
            json!({})
        };

        let res = service.request(&url, "post", None, Some(json_body)).await;
        match res {
            Ok(v) => Ok(v["success"].as_bool().unwrap_or(false)),
            Err(e) => {
                let msg = e.to_string();
                if auto_cancel && msg.contains("时间冲突") {
                    let my_classes = Self::get_participated(service).await?;
                    let my_hold_time = self.hold_time()?;
                    for c in my_classes {
                        if let Ok(ht) = c.hold_time() {
                            if ht.is_overlap(&my_hold_time) {
                                c.cancel_apply(service).await?;
                            }
                        }
                    }
                    Box::pin(self.apply(service, force, false, None)).await
                } else {
                    Err(e)
                }
            }
        }
    }

    pub async fn cancel_apply(&self, service: &YouthService) -> Result<bool> {
        let url = format!("mobile/item/cancellRegistration/{}", self.id);
        let res = service.request(&url, "post", None, None).await?;
        Ok(res["success"].as_bool().unwrap_or(false))
    }
    
    pub async fn update(&mut self, service: &YouthService) -> Result<()> {
        let url = "item/scItem/queryById";
        let params = json!({ "id": self.id });
        let new_data = service.get_result(url, Some(params)).await?;
        let new_sc: SecondClass = serde_json::from_value(new_data)?;
        *self = new_sc;
        Ok(())
    }
}