# 爬虫实现文档

本文档描述了 Better-USTC-2 项目中用于爬取中国科学技术大学“第二课堂”平台数据的爬虫实现细节。

## 请求头 (Headers)

所有请求均使用以下标准请求头：

```http
Accept: application/json, text/plain, */*
Accept-Language: en-US,en;q=0.5
Accept-Encoding: gzip, deflate, br, zstd
X-Access-Token: xxxx
Content-Type: application/json;charset=utf-8
Connection: keep-alive
Referer: https://young.ustc.edu.cn/login/sc-wisdom-group-learning/myproject/SignUp
Cookie: SameSite=None; JSESSIONID=xxxx
Sec-Fetch-Dest: empty
Sec-Fetch-Mode: cors
Sec-Fetch-Site: same-origin
Priority: u=0
TE: trailers
```

## 获取项目列表 (Get Project List)

### 请求 URL

```
https://young.ustc.edu.cn/login/wisdom-group-learning-bg/item/scItem/enrolmentList?_t=<timestamp>&requestParams=<encrypted_params>
```

### 请求参数

- `_t`: 毫秒级时间戳 (timestamp in milliseconds)
- `requestParams`: 使用 AES CBC 模式加密的 JSON 数据 (key 和 iv 需要动态获取)

### 加密参数示例

原始 JSON 数据：

```json
{
  "_t": 1763620205686,
  "column": "createTime",
  "order": "desc",
  "field": "id,,action",
  "pageNo": 1,
  "pageSize": 10
}
```

### 响应格式

```json
{
  "success": true,
  "message": "操作成功！",
  "code": 200,
  "result": {
    "records": [
      {
        "publicEndTime": null,
        "examineStatusName": null,
        "applyEt": "2025-11-26 10:00:00",
        "enshrineNum": null,
        "type": 0,
        "examineStatus": 10,
        "marathonDataStatus": null,
        "sumHours": 0.00,
        "evaluation": 0,
        "itemName": "空间量子科学",
        "actionStatus": null,
        "historyType": "000",
        "review": "0",
        "tel": "0551-63606540",
        "stick": "0",
        "id": "ff3af5caafe0b3aab5f8e373b65f61e1",
        "evaluated": null,
        "enshrine": null,
        "backOaBudget": 0,
        "itemLimitNum": null,
        "sumPersons": 0,
        "outlayDetail": null,
        "module": "z",
        "outlayMoney": 0.00,
        "createet": null,
        "regRemarks": null,
        "version": null,
        "attaType": null,
        "serviceHour": "1.0",
        "qrSigninClosed": "0",
        "auditTime": "2025-11-20 11:26:41",
        "itemLable": "1296304142622830594",
        "signInType": null,
        "businessDeptName": "合肥微尺度物质科学国家研究中心",
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
        "st": "2025-11-26 16:00:00",
        "sponsor_dictText": "中国科学技术大学",
        "needPlaceApply": "1",
        "qdClassId": null,
        "regOptions": null,
        "needSignInfo": "0",
        "updateTime": "2025-11-20 11:32:35",
        "applyNum": 0,
        "delAudit": 1,
        "canSubmitWork": false,
        "uuidKey": null,
        "businessDeptId": "234",
        "teamSize": null,
        "createTime": "2025-11-20 09:32:54",
        "organizer": "211134",
        "isKnot": 0,
        "registrationStatus": null,
        "signInfo": 0,
        "form_dictText": "现场参与",
        "pid": "-1",
        "pic": "group1/M00/2D/A0/wKgUEWkeb4eAAtNbAAGt0RkoU6M596.jpg",
        "baseContent": "活动简介...",
        "putaway": "0",
        "linkMan": "怀楠",
        "organizer_dictText": "中国科学技术大学",
        "totalServiceHour": null,
        "stick_dictText": "未置顶",
        "placeInfo": "物质科研楼3楼会议室",
        "activityLevel": "college",
        "processInstanceId": "51429025",
        "applyTeamNum": null,
        "libId": "",
        "rangeDeptIds": null,
        "marathon": "0",
        "qdClassIndex": null,
        "peopleNum": 100,
        "canGivenHours": false,
        "modules": null,
        "et": "2025-11-26 17:00:00",
        "needPlaceApply_dictText": "需要",
        "labelTo": null,
        "lastApprovalMan": "10527",
        "isAuditAssistant": null,
        "ewSponsor": null,
        "itemStatus_dictText": "报名中",
        "nj": null,
        "applyStatus": 26,
        "sponsor": "211134",
        "workStatus_dictText": null,
        "planningAtta": null,
        "isShowShare": 0,
        "cancelSign": 0,
        "itemPlaceDTO": {
          "itemId": "ff3af5caafe0b3aab5f8e373b65f61e1",
          "places": [
            {
              "itemId": null,
              "placeSt": null,
              "createBy": "P0813",
              "createTime": 1763609555000,
              "updateBy": null,
              "placeInfo": "物质科研楼3楼会议室",
              "updateTime": null,
              "id": "c3f9646c9c6c5dbb14b6be19b6ce26b1",
              "placeEt": null
            }
          ]
        },
        "conceive": "现场参与，扫码签到",
        "validHour": 1.00,
        "itemCategory_dictText": "单次项目",
        "duration": 1.0,
        "businessDeptId_dictText": "合肥微尺度物质科学国家研究中心",
        "updateBy": "P0813",
        "itemStatus": 26,
        "applySt": "2025-11-20 09:06:00",
        "module_dictText": "智",
        "teamNum": null,
        "createst": null,
        "qrSignoffClosed": "0",
        "booleanRegistration": 0,
        "workStatus": null,
        "hours": null,
        "conceiveCountNum": null,
        "attaEndTime": null,
        "activityLevel_dictText": "院级",
        "budgetProjectId": null,
        "createBy": "P0813",
        "form": "0",
        "applyWay": "0",
        "needApply": "1",
        "xq": null,
        "departTo": null,
        "applyRange": "0",
        "partakeNum": 0
      }
      // ... 更多记录
    ],
    "total": 112,
    "size": 10,
    "current": 1,
    "orders": [],
    "searchCount": true,
    "pages": 12
  },
  "timestamp": 1763620201775
}
```

## 注意事项

- 所有请求需要有效的 SESSION Cookie 和 X-Access-Token。
- 时间戳和加密参数需要动态生成。
- 响应格式可能根据实际 API 而变化。
- AES 加密使用 CBC 模式和零填充，生成 Base64 编码。