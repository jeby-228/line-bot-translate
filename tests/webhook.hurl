# 這些測試不會觸發真實的 Groq / LINE API 呼叫，
# 僅驗證 webhook 端點的 HTTP 層行為（解析、錯誤處理）。
# 測試容器未設定 LINE_CHANNEL_SECRET，因此簽章驗證會自動略過。

# 1. 健康檢查應回傳 200
GET {{host}}/
HTTP 200
[Asserts]
jsonpath "$.status" == "ok"
jsonpath "$.service" == "webhook-translate"

# 2. 不合法的 JSON body 應回傳 400
POST {{host}}/webhook
Content-Type: application/json
```
{ this is not valid json
```
HTTP 400

# 3. 空的 events 陣列應回傳 200（不呼叫任何外部 API）
POST {{host}}/webhook
Content-Type: application/json
{
  "events": []
}
HTTP 200
[Asserts]
jsonpath "$.status" == "ok"

# 4. 非 message 類型事件應被略過並回傳 200
POST {{host}}/webhook
Content-Type: application/json
{
  "events": [
    { "type": "follow", "replyToken": "dummy-token" }
  ]
}
HTTP 200
[Asserts]
jsonpath "$.status" == "ok"

# 5. message 事件但非文字類型，應被略過並回傳 200
POST {{host}}/webhook
Content-Type: application/json
{
  "events": [
    { "type": "message", "replyToken": "dummy-token", "message": { "type": "sticker" } }
  ]
}
HTTP 200
[Asserts]
jsonpath "$.status" == "ok"
