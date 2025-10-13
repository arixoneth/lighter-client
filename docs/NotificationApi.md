# \NotificationApi

All URIs are relative to *https://mainnet.zklighter.elliot.ai*

Method | HTTP request | Description
------------- | ------------- | -------------
[**notification_ack**](NotificationApi.md#notification_ack) | **POST** /api/v1/notification/ack | notification_ack



## notification_ack

> models::ResultCode notification_ack(notif_id, account_index, authorization, auth)
notification_ack

Ack notification

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**notif_id** | **String** |  | [required] |
**account_index** | **i64** |  | [required] |
**authorization** | Option<**String**> |  make required after integ is done |  |
**auth** | Option<**String**> |  made optional to support header auth clients |  |

### Return type

[**models::ResultCode**](ResultCode.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: multipart/form-data
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

