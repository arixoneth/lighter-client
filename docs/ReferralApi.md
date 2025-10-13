# \ReferralApi

All URIs are relative to *https://mainnet.zklighter.elliot.ai*

Method | HTTP request | Description
------------- | ------------- | -------------
[**referral_points**](ReferralApi.md#referral_points) | **GET** /api/v1/referral/points | referral_points



## referral_points

> models::ReferralPoints referral_points(account_index, authorization, auth)
referral_points

Get referral points

### Parameters


Name | Type | Description  | Required | Notes
------------- | ------------- | ------------- | ------------- | -------------
**account_index** | **i64** |  | [required] |
**authorization** | Option<**String**> |  make required after integ is done |  |
**auth** | Option<**String**> |  made optional to support header auth clients |  |

### Return type

[**models::ReferralPoints**](ReferralPoints.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

