# \InfoApi

All URIs are relative to *https://mainnet.zklighter.elliot.ai*

Method | HTTP request | Description
------------- | ------------- | -------------
[**transfer_fee_info**](InfoApi.md#transfer_fee_info) | **GET** /api/v1/transferFeeInfo | transferFeeInfo
[**withdrawal_delay**](InfoApi.md#withdrawal_delay) | **GET** /api/v1/withdrawalDelay | withdrawalDelay



## transfer_fee_info

> models::TransferFeeInfo transfer_fee_info(account_index, authorization, auth, to_account_index)
transferFeeInfo

Transfer fee info

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_index** | **i64** |  | [required] |
**authorization** | Option<**String**> |  |  |
**auth** | Option<**String**> |  |  |
**to_account_index** | Option<**i64**> |  |  |[default to -1]

### Return type

[**models::TransferFeeInfo**](TransferFeeInfo.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## withdrawal_delay

> models::RespWithdrawalDelay withdrawal_delay()
withdrawalDelay

Withdrawal delay in seconds

### Parameters

This endpoint does not need any parameter.

### Return type

[**models::RespWithdrawalDelay**](RespWithdrawalDelay.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

